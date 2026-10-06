package com.chunkzero.window.multistom.internal

import com.chunkzero.window.host.HudChannel
import com.chunkzero.window.host.HudDescriptor
import com.chunkzero.window.host.HudOutput
import net.kyori.adventure.bossbar.BossBar
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import net.minestom.server.scoreboard.Sidebar

/** Shows [hud] to [player] on its vanilla channel. */
internal fun showHud(
    player: Player,
    hud: HudDescriptor,
): HudOutput =
    when (hud.channel) {
        HudChannel.ACTION_BAR -> ActionBarOutput(player).also { it.update(hud.content) }
        HudChannel.BOSS_BAR -> BossBarOutput(player, hud.content)
        HudChannel.SIDEBAR -> SidebarOutput(player, hud.content)
    }

private class ActionBarOutput(
    private val player: Player,
) : HudOutput {
    override fun update(content: Component) = player.sendActionBar(content)

    override fun hide() = player.sendActionBar(Component.empty())
}

private class BossBarOutput(
    private val player: Player,
    content: Component,
) : HudOutput {
    private val bar =
        BossBar.bossBar(content, 0.0f, BossBar.Color.WHITE, BossBar.Overlay.PROGRESS).also(player::showBossBar)

    override fun update(content: Component) {
        bar.name(content)
    }

    override fun hide() = player.hideBossBar(bar)
}

private class SidebarOutput(
    private val player: Player,
    content: Component,
) : HudOutput {
    private val sidebar = Sidebar(content).also { it.addViewer(player) }

    override fun update(content: Component) = sidebar.setTitle(content)

    override fun hide() {
        sidebar.removeViewer(player)
    }
}
