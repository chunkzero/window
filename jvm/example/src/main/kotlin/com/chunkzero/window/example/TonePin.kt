package com.chunkzero.window.example

import net.minestom.server.MinecraftServer
import net.minestom.server.command.builder.Command
import net.minestom.server.command.builder.arguments.ArgumentType
import net.minestom.server.entity.Player
import net.minestom.server.tag.Tag

private val TONE_TAG = Tag.String("window_example_tone")
private const val AUTO = "auto"

/** The tone [player] pinned with `/tone`, or `null` when each view picks its own. */
internal fun pinnedTone(player: Player): String? = player.getTag(TONE_TAG)

/**
 * Registers `/tone <ok|warn|danger|auto>`, a debug command pinning the tone the caller's status HUD draws in
 * from its next send, and the shops the caller opens afterwards; `auto` unpins it.
 */
internal fun registerToneCommand() {
    val tone = ArgumentType.Word("tone").from("ok", "warn", "danger", AUTO)
    val command = Command("tone")
    command.addSyntax({ sender, context ->
        val player = sender as? Player ?: return@addSyntax
        val value = context.get(tone)
        if (value == AUTO) player.removeTag(TONE_TAG) else player.setTag(TONE_TAG, value)
    }, tone)
    MinecraftServer.getCommandManager().register(command)
}
