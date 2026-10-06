package com.chunkzero.window

import net.kyori.adventure.audience.Audience
import net.kyori.adventure.bossbar.BossBar

/**
 * Renders [hud] and sends it as this audience's action bar. The client hides an action bar after about 60 ticks, so
 * resend it to keep it visible.
 *
 * @throws IllegalArgumentException if [hud] is not composed for [HudChannel.ACTION_BAR].
 */
public fun Audience.sendHud(hud: HudView) {
    requireChannel(hud.channel, HudChannel.ACTION_BAR)
    sendActionBar(hud.render())
}

/**
 * Renders [stack] and sends it as this audience's action bar, like [sendHud] for a single HUD.
 *
 * @throws IllegalArgumentException if [stack] is not an [HudChannel.ACTION_BAR] stack.
 */
public fun Audience.sendHud(stack: HudStack) {
    requireChannel(stack.channel, HudChannel.ACTION_BAR)
    sendActionBar(stack.render())
}

/**
 * Renders [hud] into this boss bar's name and returns the bar. Showing the bar to players stays with the caller.
 *
 * @throws IllegalArgumentException if [hud] is not composed for [HudChannel.BOSS_BAR].
 */
public fun BossBar.showHud(hud: HudView): BossBar {
    requireChannel(hud.channel, HudChannel.BOSS_BAR)
    return name(hud.render())
}

/**
 * Renders [stack] into this boss bar's name and returns the bar, like [showHud] for a single HUD.
 *
 * @throws IllegalArgumentException if [stack] is not a [HudChannel.BOSS_BAR] stack.
 */
public fun BossBar.showHud(stack: HudStack): BossBar {
    requireChannel(stack.channel, HudChannel.BOSS_BAR)
    return name(stack.render())
}

internal fun requireChannel(
    actual: HudChannel,
    expected: HudChannel,
) {
    require(actual == expected) { "HUD composed for $actual cannot be sent on $expected" }
}
