package com.chunkzero.window.example

import com.chunkzero.window.HudView
import com.chunkzero.window.Windows
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.minestom.server.entity.Player

private val HUD_DEBUG_NAMES =
    listOf(
        "status_top_center",
        "status_top_left",
        "status_top_right",
        "status_left_side",
        "status_right_side",
        "status_bottom_center",
    )

internal fun sendHudSpriteDebug(
    player: Player,
    windows: Windows,
) {
    player.sendMessage(Component.text("HUD sprite debug (chat, window:ui font):"))
    for (hudName in HUD_DEBUG_NAMES) {
        player.sendMessage(Component.text(hudName, NamedTextColor.GRAY))
        player.sendMessage(windows.hud(hudName).staticHud.color(NamedTextColor.WHITE))
        repeat(8) { player.sendMessage(Component.text(" ")) }
    }
}

internal fun sendHudFlowDebug(
    player: Player,
    windows: Windows,
    huds: List<HudView>,
) {
    player.sendMessage(Component.text("HUD flow debug (chat, composed via Windows.renderHud):"))
    for (hud in huds) {
        player.sendMessage(Component.text(hud.hudName, NamedTextColor.GRAY))
        player.sendMessage(windows.renderHud(player, hud))
        repeat(8) { player.sendMessage(Component.text(" ")) }
    }
}
