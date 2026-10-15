package com.chunkzero.window.minestom.internal

import com.chunkzero.window.host.FullscreenTextListener
import net.kyori.adventure.text.Component
import net.minestom.server.component.DataComponents
import net.minestom.server.entity.Player
import net.minestom.server.event.player.PlayerPacketEvent
import net.minestom.server.inventory.type.AnvilInventory
import net.minestom.server.item.ItemStack
import net.minestom.server.network.packet.client.common.ClientPongPacket
import net.minestom.server.network.packet.client.play.ClientClickWindowPacket
import net.minestom.server.network.packet.client.play.ClientNameItemPacket
import net.minestom.server.network.packet.server.common.PingPacket
import net.minestom.server.network.packet.server.play.BundlePacket
import net.minestom.server.network.packet.server.play.SetExperiencePacket
import net.minestom.server.network.packet.server.play.WindowPropertyPacket
import java.util.concurrent.atomic.AtomicInteger

/**
 * The anvil a full-screen window opens over its canvas for [player] to type into, reporting its rename field.
 *
 * The anvil holds a paper named after the field's text, which vanilla copies into the field. The paper is [blank] with
 * its enchantments removed, so it draws nothing and the client computes no result and no repair cost; the cost is
 * sent as 0 anyway.
 *
 * The client resets its field only when the stack in the input slot changes, and the paper is never updated as the
 * player types. A seeding send, which is an open (including a reopen over the current anvil) or a resync after a click
 * that changed the input slot, is bundled with a ping. The client answers it after echoing the paper's name, so rename
 * packets before the latest pong come from an earlier anvil or are the echo, and are dropped. Every click on the anvil
 * is refused and the slots are resent; a click that leaves the input slot alone resends the same paper, which does not
 * touch the field, so renames keep flowing. One on the result slot submits. [defer] runs a task on the next tick while
 * the window is open, since an anvil can be replaced while Minestom is still closing it.
 */
internal class MinestomTextInput(
    private val player: Player,
    private val blank: ItemStack,
    private val defer: (() -> Unit) -> Unit,
) {
    private var anvil: PromptAnvil? = null
    private var listener: FullscreenTextListener? = null

    /** The rename field's text as of the latest edit reported, or the text the anvil opened with. */
    private var field = ""

    /** The ping that settles the latest seeding send, until its pong arrives. */
    private var awaiting: Int? = null

    /** The window id of the open anvil; read by the connection's writer thread. */
    @Volatile
    private var window = NO_WINDOW

    /** Whether [windowId] is the window of the open anvil. */
    fun owns(windowId: Int) = windowId == window

    fun open(
        title: Component,
        text: String,
        listener: FullscreenTextListener,
    ) {
        val current = anvil?.takeIf { player.openInventory === it }
        check(current != null || player.openInventory == null) {
            "${player.username} has another container open, so no text input can open over the full-screen window"
        }
        val target = current ?: PromptAnvil(title)
        target.setItemStack(0, seed(text), false)
        this.listener = listener
        field = text
        anvil = target
        window = target.windowId.toInt()
        try {
            synced(seeding = true) {
                if (current != null) {
                    target.setTitle(title)
                } else {
                    check(player.openInventory(target) && player.openInventory === target) {
                        "Opening a text input for ${player.username} was cancelled or redirected by an inventory open listener"
                    }
                }
                sendCost(target)
            }
        } catch (failure: Throwable) {
            reset()
            throw failure
        }
    }

    /** Closes the anvil, if one is open, without reporting it. */
    fun close() {
        val current = anvil ?: return
        reset()
        if (player.openInventory === current) player.closeInventory()
    }

    /** Handles a packet from the client while the full-screen window is open. */
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
                if (packet.windowId() != current.windowId.toInt()) return
                event.isCancelled = true
                if (packet.slot() == RESULT_SLOT) {
                    // The client may have predicted paying the result's cost before sending this click.
                    player.sendPacket(SetExperiencePacket(player.exp, player.level, 0))
                    val submitted = listener
                    close()
                    submitted?.onSubmit()
                } else {
                    resync(current, reseed = INPUT_SLOT in packet.changedSlots())
                }
            }

            else -> {
                Unit
            }
        }
    }

    private fun edit(text: String) {
        if (text == field) return
        field = text
        listener?.onEdit(text)
    }

    /**
     * Restores the anvil's slots and the cursor after a refused click. The input slot is reseeded, with the paper named
     * after the field, only when the click changed it on the client, which clears the field.
     */
    private fun resync(
        current: PromptAnvil,
        reseed: Boolean,
    ) {
        if (reseed) current.setItemStack(0, seed(field), false)
        synced(seeding = reseed) {
            current.update(player)
            player.inventory.update(player)
            sendCost(current)
        }
    }

    /** Runs [send] in a bundle, ending in the ping that settles it when [seeding]. */
    private inline fun synced(
        seeding: Boolean,
        send: () -> Unit,
    ) {
        val id = if (seeding) PINGS.getAndIncrement() else null
        if (id != null) awaiting = id
        player.sendPacket(BundlePacket())
        try {
            send()
            if (id != null) player.sendPacket(PingPacket(id))
        } finally {
            player.sendPacket(BundlePacket())
        }
    }

    private fun sendCost(target: PromptAnvil) {
        player.sendPacket(WindowPropertyPacket(target.windowId.toInt(), REPAIR_COST, 0))
    }

    private fun ended(closing: PromptAnvil) {
        if (closing !== anvil) return
        val cancelled = listener
        reset()
        defer { cancelled?.onCancel() }
    }

    private fun reset() {
        anvil = null
        listener = null
        awaiting = null
        window = NO_WINDOW
    }

    private fun seed(text: String): ItemStack =
        blank
            .with { it.set(DataComponents.CUSTOM_NAME, Component.text(text)) }
            .without(DataComponents.ENCHANTMENTS)

    /** An anvil that reports when its viewer is removed, whether it closes or another inventory replaces it. */
    private inner class PromptAnvil(
        title: Component,
    ) : AnvilInventory(title) {
        override fun removeViewer(player: Player): Boolean {
            if (player in viewers) ended(this)
            return super.removeViewer(player)
        }
    }

    private companion object {
        const val INPUT_SLOT: Short = 0
        const val RESULT_SLOT: Short = 2

        /** The anvil window property carrying the repair cost. */
        const val REPAIR_COST: Short = 0
        const val NO_WINDOW = Int.MIN_VALUE

        /** Ping ids start in a range unlikely to collide with other users of the ping packet. */
        val PINGS = AtomicInteger(0x57_46_00_00)
    }
}
