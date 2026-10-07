package com.chunkzero.window.example

import com.chunkzero.window.example.generated.StatusBottomCenterHud
import com.chunkzero.window.example.generated.StatusLeftSideHud
import com.chunkzero.window.example.generated.StatusRightSideHud
import com.chunkzero.window.example.generated.StatusTopCenterHud
import com.chunkzero.window.example.generated.StatusTopLeftHud
import com.chunkzero.window.example.generated.StatusTopRightHud
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player

class MyStatusTopLeftHud(
    private val player: Player,
    private val market: Market,
) : StatusTopLeftHud() {
    override fun coins(): Component = Component.text(market.balanceOf(player))

    override fun rate(): Component = Component.text("+${market.unitPrice}")

    override fun power(): Component = Component.text("x1")
}

class MyStatusTopCenterHud(
    private val startedAt: Long,
) : StatusTopCenterHud() {
    override fun runtime(): Component {
        val seconds = (System.currentTimeMillis() - startedAt) / 1000L
        return Component.text("%d:%02d".format(seconds / 60, seconds % 60))
    }
}

class MyStatusTopRightHud(
    private val player: Player,
    private val startedAt: Long,
) : StatusTopRightHud() {
    override fun wave(): Component = Component.text(1 + (System.currentTimeMillis() - startedAt) / WAVE_MILLIS)

    override fun biome(): Component = Component.text("Plains")

    override fun latency(): Component = Component.text("${player.latency}ms")
}

class MyStatusLeftSideHud(
    private val player: Player,
) : StatusLeftSideHud() {
    override fun coords(): Component {
        val pos = player.position
        return Component.text("${pos.blockX()}, ${pos.blockZ()}")
    }

    override fun altitude(): Component = Component.text("Y ${player.position.blockY()}")
}

class MyStatusRightSideHud(
    private val market: Market,
) : StatusRightSideHud() {
    override fun objective(): Component = Component.text("Sell goods")

    override fun stock(): Component = Component.text("${market.unitPrice} per item")
}

class MyStatusBottomCenterHud(
    private val startedAt: Long,
) : StatusBottomCenterHud() {
    override fun hint(): Component {
        val remaining = WAVE_MILLIS - (System.currentTimeMillis() - startedAt) % WAVE_MILLIS
        return Component.text("Next wave in ${(remaining + 999) / 1000}s")
    }

    override fun waveProgress(index: Int): WaveProgress {
        val lit = (System.currentTimeMillis() - startedAt) % WAVE_MILLIS * 10 / WAVE_MILLIS >= index
        return if (lit) WaveProgress.ON else WaveProgress.OFF
    }
}

private const val WAVE_MILLIS = 10_000L
