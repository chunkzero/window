package com.chunkzero.window.example

import com.chunkzero.window.example.generated.WindowHudDefinitions
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.minestom.server.entity.Player

private val HUD_DEBUG_DEFINITIONS =
    listOf(
        WindowHudDefinitions.statusTopCenter,
        WindowHudDefinitions.statusTopLeft,
        WindowHudDefinitions.statusTopRight,
        WindowHudDefinitions.statusLeftSide,
        WindowHudDefinitions.statusRightSide,
        WindowHudDefinitions.statusBottomCenter,
    )

internal fun sendHudSpriteDebug(player: Player) {
    player.sendMessage(Component.text("HUD sprite debug (chat, window:ui font):"))
    for (definition in HUD_DEBUG_DEFINITIONS) {
        player.sendMessage(Component.text(definition.name, NamedTextColor.GRAY))
        player.sendMessage(definition.staticHud.color(NamedTextColor.WHITE))
        repeat(8) { player.sendMessage(Component.text(" ")) }
    }
}
