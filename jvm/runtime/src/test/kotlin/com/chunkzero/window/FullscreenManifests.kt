package com.chunkzero.window

import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.ControlEntry
import com.chunkzero.window.manifest.ControlTooltipEntry
import com.chunkzero.window.manifest.CursorEntry
import com.chunkzero.window.manifest.FullscreenCanvasEntry
import com.chunkzero.window.manifest.FullscreenEntry
import com.chunkzero.window.manifest.FullscreenMode
import com.chunkzero.window.manifest.FullscreenSurfaceEntry
import com.chunkzero.window.manifest.LayerEntry
import com.chunkzero.window.manifest.LayerKind
import com.chunkzero.window.manifest.SwitchCaseEntry
import com.chunkzero.window.manifest.SwitchEntry
import com.chunkzero.window.manifest.WindowManifest

/**
 * A full-screen menu: a backdrop, a bound count, a `play` button with an outline, a `quit` button, a switch, and a
 * two-cell repeater paged by `next`, whose second cell has a tooltip.
 */
object FullscreenManifests {
    const val STATIC_MARKER = "#a00400"
    const val COUNT_MARKER = "#a00801"
    const val OVERLAY_MARKER = "#a3f800"
    const val TOOLTIP_MARKER = "#a3f000"
    const val TOOLTIP_TEXT_MARKER = "#a3f401"

    val canvas =
        FullscreenCanvasEntry(
            width = 480,
            height = 270,
            degreesPerPixel = 0.1875,
            edgeYaw = 45.0,
            edgePitch = 25.3125,
            snapEpsilon = 0.01,
            hitMargin = 1,
            displayScale = 256.0,
            markerAlpha = 253,
            cursor = CursorEntry(text = "CURSOR", marker = "#a7fc00", width = 8, height = 12),
            backdrop = "BACKDROP",
            overlayMarker = OVERLAY_MARKER,
            hudHiddenWorldAge = -1_000_000_001,
            promptWorldAge = -999_795_201,
            hudStripHeight = 56,
            hudStripHalfWidth = 96,
            hudMaxHealth = 20,
        )

    fun manifest(mode: FullscreenMode = FullscreenMode.UNLOCKED): WindowManifest =
        TestManifests.hudManifest().copy(
            huds = emptyMap(),
            fullscreens =
                mapOf(
                    "menu" to
                        FullscreenEntry(
                            surface = FullscreenSurfaceEntry("fullscreen", mode, listOf(480, 270)),
                            static = "STATIC",
                            staticMarker = STATIC_MARKER,
                            slots =
                                mapOf(
                                    "count" to TestManifests.slot(0, 40, Align.LEFT),
                                    "cell[0]" to TestManifests.slot(100, 40, Align.LEFT, y = 190),
                                    "cell[1]" to TestManifests.slot(150, 40, Align.LEFT, y = 190),
                                ),
                            controls =
                                mapOf(
                                    "play" to control(0, 100, action = "play", outline = "OUTLINE"),
                                    "quit" to control(1, 130, action = "window:close", default = "window:close"),
                                    "secret" to control(2, 160, action = "secret"),
                                    "pick[0]" to control(3, 190, action = "pick[0]"),
                                    "pick[1]" to control(4, 190, action = "pick[1]", x = 150, tooltip = tooltip),
                                    "next" to control(5, 220, action = "next"),
                                ),
                            switches =
                                mapOf(
                                    "mode" to
                                        SwitchEntry(
                                            listOf(
                                                SwitchCaseEntry("on", regions = listOf("secret")),
                                                SwitchCaseEntry("off"),
                                            ),
                                        ),
                                ),
                            layers =
                                listOf(
                                    LayerEntry(LayerKind.SLOT, "count"),
                                    LayerEntry(LayerKind.SWITCH, "mode"),
                                    LayerEntry(LayerKind.SLOT, "cell[0]"),
                                    LayerEntry(LayerKind.SLOT, "cell[1]"),
                                ),
                            markers =
                                mapOf(
                                    "count" to COUNT_MARKER,
                                    "mode" to "#a00c00",
                                    "cell[0]" to "#a01000",
                                    "cell[1]" to "#a01400",
                                ),
                            backdropMarker = "#a80000",
                        ),
                ),
            fullscreenCanvas = canvas,
        )

    private val tooltip =
        ControlTooltipEntry(
            panel = "PANEL",
            marker = TOOLTIP_MARKER,
            lines =
                listOf(
                    TestManifests.slot(150, 30, Align.LEFT, shaderMarker = TOOLTIP_TEXT_MARKER, text = "Tip", y = 214),
                ),
        )

    /** A 40x20 control at [x] and [y]. */
    private fun control(
        index: Int,
        y: Int,
        action: String,
        default: String? = null,
        outline: String? = null,
        x: Int = 100,
        tooltip: ControlTooltipEntry? = null,
    ) = ControlEntry(
        index,
        x,
        y,
        40,
        20,
        action,
        default,
        hover = outline != null,
        outline = outline,
        tooltip = tooltip,
    )

    /** The yaw and pitch that put the cursor on the centre of canvas pixel ([x], [y]). */
    fun rotation(
        x: Int,
        y: Int,
    ): Pair<Float, Float> = ((x + 0.5 - 240) * 0.1875).toFloat() to ((y + 0.5 - 135) * 0.1875).toFloat()
}
