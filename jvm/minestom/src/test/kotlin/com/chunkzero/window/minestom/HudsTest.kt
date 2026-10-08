package com.chunkzero.window.minestom

import com.chunkzero.window.HudDefinition
import com.chunkzero.window.HudScope
import com.chunkzero.window.HudView
import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.HudEntry
import com.chunkzero.window.manifest.HudSurfaceEntry
import com.chunkzero.window.manifest.LayerEntry
import com.chunkzero.window.manifest.LayerKind
import com.chunkzero.window.manifest.SlotEntry
import com.chunkzero.window.manifest.WindowManifest
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import net.kyori.adventure.text.Component
import net.minestom.server.scoreboard.Sidebar

class HudsTest :
    StringSpec({
        "a sidebar shows the HUD's current render as its title" {
            val hud = Label("sidebar")
            val sidebar = Sidebar(Component.empty())

            sidebar.showHud(hud)
            val first = sidebar.title
            hud.text = "B"
            sidebar.showHud(hud)

            first shouldNotBe sidebar.title
            sidebar.title shouldBe hud.render()
        }

        "a sidebar only shows sidebar HUDs" {
            shouldThrow<IllegalArgumentException> { Sidebar(Component.empty()).showHud(Label("actionbar")) }
        }
    })

private class Label(
    channel: String,
) : HudView(HudDefinition(manifest(channel), "label")) {
    var text = "A"

    override fun HudScope.bind() {
        slot("text") { Component.text(text) }
    }
}

private fun manifest(channel: String): WindowManifest {
    val magnitudes = (0..10).map { 1 shl it }
    val advances = magnitudes.reversed().map { -it } + magnitudes
    return WindowManifest(
        version = WindowManifest.VERSION,
        namespace = "window",
        font = "window:ui",
        spacers = advances.withIndex().associate { (index, advance) -> 0xF0000 + index to advance },
        textAdvances = ('A'..'Z').associate { it.toString() to 6 },
        windows = emptyMap(),
        huds =
            mapOf(
                "label" to
                    HudEntry(
                        surface = HudSurfaceEntry("hud", channel, width = 100, height = 16),
                        static = "",
                        slots = mapOf("text" to SlotEntry(0, 6, 40, Align.LEFT, "window:y0", "#ffffff", false)),
                        layers = listOf(LayerEntry(LayerKind.SLOT, "text")),
                    ),
            ),
    )
}
