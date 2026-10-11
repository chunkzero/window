package com.chunkzero.window.example

import net.minestom.server.MinecraftServer
import net.minestom.server.command.builder.Command
import net.minestom.server.command.builder.arguments.ArgumentType
import net.minestom.server.entity.Player
import net.minestom.server.tag.Tag

private val BAR_TAG = Tag.Float("window_example_bar")

/** The fraction [player] pinned with `/bar`, or `null` when each view fills its own bar. */
internal fun pinnedBar(player: Player): Float? = player.getTag(BAR_TAG)

/**
 * Registers `/bar <fraction|auto>`, a debug command pinning how much of its bar the caller's status HUD fills
 * from its next send, and the shops the caller opens afterwards; `auto` unpins it.
 */
internal fun registerBarCommand() {
    val fraction = ArgumentType.Float("fraction").between(0f, 1f)
    val command = Command("bar")
    command.addSyntax({ sender, context ->
        (sender as? Player)?.setTag(BAR_TAG, context.get(fraction))
    }, fraction)
    command.addSyntax({ sender, _ -> (sender as? Player)?.removeTag(BAR_TAG) }, ArgumentType.Literal("auto"))
    MinecraftServer.getCommandManager().register(command)
}
