package com.chunkzero.window

import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.ControlEntry
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

/** A full-screen menu: a backdrop, a bound count, a `play` button with an outline, a `quit` button, and a switch. */
object FullscreenManifests {
    const val STATIC_MARKER = "#a00400"
    const val COUNT_MARKER = "#a00801"
    const val OVERLAY_MARKER = "#a3f800"

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
                            slots = mapOf("count" to TestManifests.slot(0, 40, Align.LEFT)),
                            controls =
                                mapOf(
                                    "play" to control(0, 100, action = "play", outline = "OUTLINE"),
                                    "quit" to control(1, 130, action = "window:close", default = "window:close"),
                                    "secret" to control(2, 160, action = "secret"),
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
                            layers = listOf(LayerEntry(LayerKind.SLOT, "count"), LayerEntry(LayerKind.SWITCH, "mode")),
                            markers = mapOf("count" to COUNT_MARKER, "mode" to "#a00c00"),
                            backdropMarker = "#a80000",
                        ),
                ),
            fullscreenCanvas = canvas,
        )

    /** A 40x20 control at x 100 and [y]. */
    private fun control(
        index: Int,
        y: Int,
        action: String,
        default: String? = null,
        outline: String? = null,
    ) = ControlEntry(index, 100, y, 40, 20, action, default, hover = outline != null, outline = outline)

    /** The yaw and pitch that put the cursor on the centre of canvas pixel ([x], [y]). */
    fun rotation(
        x: Int,
        y: Int,
    ): Pair<Float, Float> = ((x + 0.5 - 240) * 0.1875).toFloat() to ((y + 0.5 - 135) * 0.1875).toFloat()
}
