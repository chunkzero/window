package com.chunkzero.window.minestom.internal

import com.chunkzero.window.host.FullscreenTextListener
import net.kyori.adventure.text.Component
import net.minestom.server.component.DataComponents
import net.minestom.server.entity.Player
import net.minestom.server.event.player.PlayerPacketEvent
import net.minestom.server.inventory.type.AnvilInventory
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material
import net.minestom.server.network.packet.client.common.ClientPongPacket
import net.minestom.server.network.packet.client.play.ClientClickWindowPacket
import net.minestom.server.network.packet.client.play.ClientNameItemPacket
import net.minestom.server.network.packet.server.common.PingPacket
import net.minestom.server.network.packet.server.play.BundlePacket
import java.util.concurrent.atomic.AtomicInteger

/**
 * The anvil a full-screen window opens over its canvas for [player] to type into, reporting its rename field.
 *
 * The anvil holds a paper named after the field's text, which vanilla copies into the field. Every open, including a
 * reopen over the current anvil, is bundled with a ping: the client answers it after echoing the paper's name, so
 * rename packets before the pong come from an earlier anvil or are the echo, and are dropped. Every click on the anvil
 * is refused and resynced; one on the result slot submits. [defer] runs a task on the next tick while the window is
 * open, since an anvil can be replaced while Minestom is still closing it.
 */
internal class MinestomTextInput(
    private val player: Player,
    private val defer: (() -> Unit) -> Unit,
) {
    private var anvil: PromptAnvil? = null
    private var listener: FullscreenTextListener? = null

    /** The rename field's text as of the latest edit reported, or the text the anvil opened with. */
    private var field = ""

    /** The ping that settles the latest open, until its pong arrives. */
    private var awaiting: Int? = null

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
        val id = PINGS.getAndIncrement()
        awaiting = id
        player.sendPacket(BundlePacket())
        try {
            if (current != null) {
                target.setTitle(title)
            } else {
                check(player.openInventory(target) && player.openInventory === target) {
                    "Opening a text input for ${player.username} was cancelled or redirected by an inventory open listener"
                }
            }
            player.sendPacket(PingPacket(id))
        } catch (failure: Throwable) {
            reset()
            throw failure
        } finally {
            player.sendPacket(BundlePacket())
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
                if (awaiting != null || packet.itemName() == field) return
                field = packet.itemName()
                listener?.onEdit(field)
            }

            is ClientClickWindowPacket -> {
                if (packet.windowId() != current.windowId.toInt()) return
                event.isCancelled = true
                if (packet.slot() == RESULT_SLOT) {
                    val submitted = listener
                    close()
                    submitted?.onSubmit()
                } else {
                    // Renaming the paper to the field keeps the resync from resetting what the player typed.
                    current.setItemStack(0, seed(field), false)
                    current.update(player)
                    player.inventory.update(player)
                }
            }

            else -> {
                Unit
            }
        }
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
    }

    private fun seed(text: String): ItemStack =
        ItemStack
            .builder(Material.PAPER)
            .set(DataComponents.CUSTOM_NAME, Component.text(text))
            .build()

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
        const val RESULT_SLOT: Short = 2

        /** Ping ids start in a range unlikely to collide with other users of the ping packet. */
        val PINGS = AtomicInteger(0x57_46_00_00)
    }
}
