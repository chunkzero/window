package com.chunkzero.window.multistom

import com.chunkzero.window.host.ContainerKind
import com.chunkzero.window.host.ContainerListener
import com.chunkzero.window.host.OpenContainer
import com.chunkzero.window.host.WindowHost
import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.multistom.internal.MultistomContainer
import com.chunkzero.window.multistom.internal.itemStack
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import net.minestom.server.item.ItemStack
import net.minestom.server.tag.Tag

/**
 * The Multistom [WindowHost] for [player].
 *
 * Containers listen on a child of the player's event node, and scheduled work runs on the player's scheduler, so
 * Window state for this player must be changed from the thread that ticks the player.
 */
public class MultistomHost private constructor(
    public val player: Player,
) : WindowHost<ItemStack> {
    override fun open(
        kind: ContainerKind,
        title: Component,
        listener: ContainerListener,
    ): OpenContainer<ItemStack> = MultistomContainer.open(player, kind, title, listener)

    override fun item(item: WindowItem): ItemStack = itemStack(item)

    override fun scheduleNextTick(task: Runnable) {
        player.scheduler().scheduleNextTick(task)
    }

    public companion object {
        private val TAG = Tag.Transient<MultistomHost>("window:host")

        /** The host for [player], shared by every view opened for them. */
        @JvmStatic
        public fun of(player: Player): MultistomHost = player.updateAndGetTag(TAG) { it ?: MultistomHost(player) }
    }
}
