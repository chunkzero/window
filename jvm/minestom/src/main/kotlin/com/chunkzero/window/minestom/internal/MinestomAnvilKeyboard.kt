package com.chunkzero.window.minestom.internal

import net.kyori.adventure.text.Component
import net.minestom.server.component.DataComponents
import net.minestom.server.entity.Player
import net.minestom.server.event.player.PlayerPacketEvent
import net.minestom.server.inventory.type.AnvilInventory
import net.minestom.server.item.ItemStack
import net.minestom.server.network.packet.client.common.ClientPongPacket
import net.minestom.server.network.packet.client.play.ClientClickWindowPacket
import net.minestom.server.network.packet.client.play.ClientCloseWindowPacket
import net.minestom.server.network.packet.client.play.ClientNameItemPacket
import net.minestom.server.network.packet.server.common.PingPacket
import net.minestom.server.network.packet.server.play.BundlePacket
import net.minestom.server.network.packet.server.play.SetExperiencePacket
import net.minestom.server.network.packet.server.play.WindowItemsPacket
import net.minestom.server.network.packet.server.play.WindowPropertyPacket
import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.atomic.AtomicInteger

/**
 * The keyboard text prompt's anvil: untitled, and used only to capture typing. While it is open, [sendBand] sends the
 * prompt's world age band, which the pack's GUI shaders read as "hide what the anvil draws", and every slot it shows
 * holds [blank], which draws nothing, shows no tooltip, and, in the hotbar, hides the arm. Closing it puts back the band, the player's
 * slots through [restoreSlots], and their XP.
 *
 * The input paper is named after the field's text, which vanilla copies into the rename field, and has no
 * enchantments, so the client computes no repair cost. The client resets its field only when the input slot changes,
 * so each seed carries a fresh lore line. A seeding send is bundled with a ping; rename packets before its pong come
 * from an earlier anvil or are the seed's echo, and are dropped. The field holds at most [MAX_LENGTH] characters, and a
 * longer initial text is truncated to fit.
 *
 * The server cannot see the client's caret, so Escape, which closes the anvil, finishes the edit with the text so far.
 * Clicking a slot or outside the anvil's window also finishes it, but the client sends nothing for a click on the
 * rename field or elsewhere inside the window, so a click is only a best-effort way to finish. Another inventory
 * replacing the anvil displaces it. Each is reported to [events] after the anvil's state is reset and the player's
 * band, slots, and XP are put back.
 */
internal class MinestomAnvilKeyboard(
    private val player: Player,
    private val blank: ItemStack,
    private val sendBand: () -> Unit,
    private val restoreSlots: () -> Unit,
    private val events: Events,
) {
    /** What the player did with the open anvil. */
    interface Events {
        /** The field changed to [text]. */
        fun onEdit(text: String)

        /** The player pressed Escape, or clicked a slot or outside the window; the anvil is closed. */
        fun onFinish(text: String)

        /** Another inventory replaced the anvil, or the server closed it. */
        fun onDisplaced()
    }

    private val unenchanted = blank.without(DataComponents.ENCHANTMENTS)
    private var anvil: PromptAnvil? = null

    /** The rename field's text as of the latest edit, or the text it was seeded with. */
    private var field = ""

    /** Distinguishes each seed, so a reseed always changes the input slot and resets the client's field. */
    private var seeds = 0

    /** The ping that settles the latest seeding send, until its pong arrives. */
    private var awaiting: Int? = null

    /** The window id of the open anvil, or [NO_WINDOW]; read by the connection's writer thread. */
    @Volatile
    private var window = NO_WINDOW

    /** The window ids of anvils whose open packet the writer has not written yet, oldest first. */
    private val unwritten = ConcurrentLinkedQueue<Int>()

    /** Whether the anvil is open, so the world age belongs in the prompt band. */
    val isOpen: Boolean get() = window != NO_WINDOW

    /**
     * Whether the open packet the writer is writing for [windowId] is an anvil's rather than a server container's. Call
     * once per open packet, on the writer thread.
     */
    fun ownsOpen(windowId: Int): Boolean = unwritten.peek() == windowId && unwritten.poll() != null

    /** Opens the anvil holding [initial], or reseeds the open one. False when an inventory listener refused it. */
    fun open(initial: String): Boolean {
        val reused = anvil?.takeIf { player.openInventory === it }
        val target = reused ?: PromptAnvil()
        field = initial.take(MAX_LENGTH)
        target.setItemStack(INPUT_SLOT, seed(field), false)
        anvil = target
        window = target.windowId.toInt()
        if (reused == null) unwritten += window
        val opened =
            synced(seeding = true) {
                sendBand()
                val shown = reused != null || (player.openInventory(target) && player.openInventory === target)
                if (shown) sendSlots(target)
                shown
            }
        if (!opened) {
            unwritten.remove(target.windowId.toInt())
            reset()
            restore()
        }
        return opened
    }

    /** Closes the anvil, if one is open, without reporting it. */
    fun close() {
        val current = anvil ?: return
        reset()
        synced(seeding = false) {
            if (player.openInventory === current) player.closeInventory()
            restore()
        }
    }

    fun onPacket(event: PlayerPacketEvent) {
        val current = anvil ?: return
        when (val packet = event.packet) {
            is ClientPongPacket -> {
                if (packet.id() == awaiting) awaiting = null
            }

            is ClientNameItemPacket -> {
                event.isCancelled = true
                if (awaiting == null) edit(packet.itemName())
            }

            is ClientClickWindowPacket -> {
                if (packet.windowId() != window) return
                event.isCancelled = true
                finish(current, closing = true)
            }

            // Minestom closes the anvil once this event returns, and ignores the close if another window opened since.
            is ClientCloseWindowPacket -> {
                if (packet.windowId() == window) finish(current, closing = false)
            }

            else -> {
                Unit
            }
        }
    }

    private fun edit(name: String) {
        val typed = name.filter { it >= ' ' && it != '\u007f' && it != '§' }.take(MAX_LENGTH)
        if (typed == field) return
        field = typed
        events.onEdit(field)
    }

    /** Ends the edit with its text; with [closing], the anvil is still open on the client, so the server closes it. */
    private fun finish(
        current: PromptAnvil,
        closing: Boolean,
    ) {
        val text = field
        reset()
        synced(seeding = false) {
            if (closing && player.openInventory === current) player.closeInventory()
            restore()
        }
        events.onFinish(text)
    }

    /** The anvil lost its viewer while still open: another inventory replaced it, or the server closed it. */
    private fun displaced(closing: PromptAnvil) {
        if (closing !== anvil) return
        reset()
        restore()
        events.onDisplaced()
    }

    private fun reset() {
        anvil = null
        window = NO_WINDOW
        awaiting = null
    }

    /** Puts back the HUD band, the player's slots, and their XP, which the client may have predicted paying. */
    private fun restore() {
        sendBand()
        restoreSlots()
        player.sendPacket(SetExperiencePacket(player.exp, player.level, 0))
    }

    /** Sends the anvil's slots with the player's slots blank, no cursor item, and no repair cost. */
    private fun sendSlots(target: PromptAnvil) {
        val items = List(SLOTS) { slot -> if (slot < ANVIL_SLOTS) target.getItemStack(slot) else blank }
        player.sendPacket(WindowItemsPacket(target.windowId.toInt(), 0, items, ItemStack.AIR))
        player.sendPacket(WindowPropertyPacket(target.windowId.toInt(), REPAIR_COST, 0))
    }

    /** Runs [send] in a bundle, ending in the ping that settles it when [seeding]. */
    private inline fun <T> synced(
        seeding: Boolean,
        send: () -> T,
    ): T {
        val id = if (seeding) PINGS.getAndIncrement() else null
        if (id != null) awaiting = id
        player.sendPacket(BundlePacket())
        try {
            return send().also { if (id != null) player.sendPacket(PingPacket(id)) }
        } finally {
            player.sendPacket(BundlePacket())
        }
    }

    private fun seed(text: String): ItemStack =
        unenchanted
            .with(DataComponents.CUSTOM_NAME, Component.text(text))
            .with(DataComponents.LORE, listOf(Component.text(seeds++)))

    /** An anvil that reports when its viewer is removed, whether it closes or another inventory replaces it. */
    private inner class PromptAnvil : AnvilInventory(Component.empty()) {
        override fun removeViewer(player: Player): Boolean {
            if (player in viewers) displaced(this)
            return super.removeViewer(player)
        }
    }

    private companion object {
        const val INPUT_SLOT = 0

        /** The anvil's own slots: two inputs and the result. */
        const val ANVIL_SLOTS = 3

        /** The anvil's slots and the player's 36. */
        const val SLOTS = ANVIL_SLOTS + 36

        /** The most characters the anvil's rename field holds. */
        const val MAX_LENGTH = 50

        /** The anvil window property carrying the repair cost. */
        const val REPAIR_COST: Short = 0
        const val NO_WINDOW = Int.MIN_VALUE

        /** Ping ids start in a range unlikely to collide with other users of the ping packet. */
        val PINGS = AtomicInteger(0x57_46_00_00)
    }
}
