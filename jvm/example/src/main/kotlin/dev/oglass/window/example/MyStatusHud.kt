package dev.oglass.window.example

import dev.oglass.window.example.generated.StatusBottomCenterHud
import dev.oglass.window.example.generated.StatusLeftSideHud
import dev.oglass.window.example.generated.StatusRightSideHud
import dev.oglass.window.example.generated.StatusTopCenterHud
import dev.oglass.window.example.generated.StatusTopLeftHud
import dev.oglass.window.example.generated.StatusTopRightHud
import net.kyori.adventure.text.Component

class MyStatusTopLeftHud(
    private val market: Market,
) : StatusTopLeftHud() {
    override fun coins(): Component = Component.text(market.balanceOf(player))

    override fun rate(): Component = Component.text("+${market.unitPrice}")

    override fun power(): Component = Component.text("x1")
}

class MyStatusTopCenterHud(
    private val startedAt: Long,
) : StatusTopCenterHud() {
    override fun runtime(): Component = Component.text("${(System.currentTimeMillis() - startedAt) / 1000L}s")
}

class MyStatusTopRightHud(
    private val startedAt: Long,
) : StatusTopRightHud() {
    override fun wave(): Component = Component.text("#${1 + ((System.currentTimeMillis() - startedAt) / 10000L)}")

    override fun biome(): Component = Component.text("Plains")

    override fun latency(): Component = Component.text("${player.latency}ms")
}

class MyStatusLeftSideHud : StatusLeftSideHud() {
    override fun coords(): Component {
        val pos = player.position
        return Component.text("${pos.blockX()},${pos.blockZ()}")
    }

    override fun altitude(): Component = Component.text("Y${player.position.blockY()}")
}

class MyStatusRightSideHud(
    private val market: Market,
) : StatusRightSideHud() {
    override fun objective(): Component = Component.text("Sell goods")

    override fun stock(): Component = Component.text("${market.unitPrice} coins/item")
}

class MyStatusBottomCenterHud : StatusBottomCenterHud() {
    override fun leftNote(): Component = Component.text("center")

    override fun rightNote(): Component = Component.text("bottom")
}
