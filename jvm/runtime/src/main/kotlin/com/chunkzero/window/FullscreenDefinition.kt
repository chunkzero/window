package com.chunkzero.window

import com.chunkzero.window.internal.HudComposer
import com.chunkzero.window.internal.HudLayer
import com.chunkzero.window.internal.baseStyle
import com.chunkzero.window.internal.requireColor
import com.chunkzero.window.manifest.FullscreenCanvasEntry
import com.chunkzero.window.manifest.FullscreenEntry
import com.chunkzero.window.manifest.FullscreenMode
import com.chunkzero.window.manifest.FullscreenSurfaceEntry
import com.chunkzero.window.manifest.HudEntry
import com.chunkzero.window.manifest.HudShaderEntry
import com.chunkzero.window.manifest.HudSurfaceEntry
import com.chunkzero.window.manifest.WindowManifest
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component

/**
 * The full-screen window [name] of [manifest], with the canvas contract its pack shares.
 *
 * @throws IllegalArgumentException if [manifest] has no full-screen window [name] (listing the known names).
 */
public class FullscreenDefinition(
    internal val manifest: WindowManifest,
    /** The full-screen window name. */
    public val name: String,
) {
    internal val entry: FullscreenEntry =
        manifest.fullscreens[name]
            ?: throw IllegalArgumentException(
                "Unknown fullscreen '$name'; known fullscreens: ${manifest.fullscreens.keys.sorted()}",
            )

    /** The canvas contract shared by every full-screen window of the pack. */
    public val canvas: FullscreenCanvasEntry =
        requireNotNull(manifest.fullscreenCanvas) { "The manifest has fullscreen '$name' but no fullscreen canvas" }

    /** Surface metadata for this full-screen window. */
    public val surface: FullscreenSurfaceEntry
        get() = entry.surface

    /** The pack's item model that draws nothing, which fills the hotbar while a full-screen window is open. */
    public val blankItemModel: Key = Key.key(manifest.namespace, "fullscreen/blank")

    /**
     * The window's layers as a shader-placed HUD: every run returns to the line origin, and each slot and layer is sent
     * in its marker.
     */
    internal val layers: HudEntry =
        HudEntry(
            surface = HudSurfaceEntry(entry.surface.kind, entry.surface.kind, canvas.width, canvas.height),
            static = entry.static,
            slots = entry.slots.mapValues { (slot, it) -> it.copy(shaderMarker = entry.markers[slot]) },
            shader = HudShaderEntry(staticMarker = entry.staticMarker, sourceBottom = 0),
            switches = entry.switches,
            art = entry.art,
            tints = entry.tints,
            layers = entry.layers,
        )

    internal val composer = HudComposer(manifest, layers, entry.markers)

    /** The backdrop run, or `null` when the window shows the world behind it. */
    internal val backdrop: Component? = entry.backdropMarker?.let { run(canvas.backdrop, it) }

    /**
     * The cursor run, which the text shader moves to the cursor, or `null` in horse mode, where the pack's entity shader
     * draws the cursor instead.
     */
    internal val cursor: Component? =
        run(canvas.cursor.text, canvas.cursor.marker).takeIf { entry.surface.mode == FullscreenMode.UNLOCKED }

    /** The hover outline run of each control with one, by control key. */
    internal val outlines: Map<String, Component> =
        entry.controls
            .mapNotNull { (key, control) -> control.outline?.let { key to run(it, canvas.overlayMarker) } }
            .toMap()

    /** The tooltip runs of each control with one, by control key: its panel, then its lines. */
    internal val tooltips: Map<String, Component> =
        entry.controls
            .mapNotNull { (key, control) ->
                control.tooltip?.let { tooltip ->
                    val id = "fullscreen/$name/tooltip/$key"
                    val lines =
                        tooltip.lines.mapNotNull { line ->
                            composer.renderSlot(id, line, Component.text(line.text.orEmpty()))?.let(HudLayer::Slot)
                        }
                    val panel = HudLayer.Art(id, tooltip.panel, null, requireColor(tooltip.marker))
                    key to composer.composeLayers(listOf(panel) + lines)
                }
            }.toMap()

    private fun run(
        text: String,
        marker: String,
    ): Component = Component.text(text, baseStyle(manifest.font, requireColor(marker)))
}
