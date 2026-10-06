package com.chunkzero.window.minestom

import com.chunkzero.window.HudChannel
import com.chunkzero.window.HudView
import net.minestom.server.scoreboard.Sidebar

/**
 * Renders [hud] into this sidebar's title. Adding viewers and resending after changes stay with the caller.
 *
 * @throws IllegalArgumentException if [hud] is not composed for [HudChannel.SIDEBAR].
 */
public fun Sidebar.showHud(hud: HudView) {
    require(hud.channel == HudChannel.SIDEBAR) { "HUD composed for ${hud.channel} cannot be shown on a sidebar" }
    setTitle(hud.render())
}
