package com.chunkzero.window.manifest

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/**
 * A full-screen window: drawn with no screen open as marker-colored glyph runs on a client-only text display, which
 * the generated text shader moves onto the fixed canvas of [FullscreenCanvasEntry]. Coordinates are canvas pixels.
 */
@Serializable
public data class FullscreenEntry(
    /** Full-screen surface metadata. */
    val surface: FullscreenSurfaceEntry,
    /** Net-zero baked static art from the canvas origin, sent first in [WindowManifest.font]. */
    @SerialName("static") val static: String,
    /** The marker color [static] is sent in. */
    @SerialName("static_marker") val staticMarker: String,
    /** Dynamic text slots, including static labels (which carry [SlotEntry.text]). */
    val slots: Map<String, SlotEntry> = emptyMap(),
    /** Clickable and hoverable rects keyed by region key, which the cursor is hit-tested against. */
    val controls: Map<String, ControlEntry> = emptyMap(),
    /** Runtime-selected cases keyed by switch key; a case's [SwitchCaseEntry.regions] name its controls. */
    val switches: Map<String, SwitchEntry> = emptyMap(),
    /** Art layers keyed by name; a tinted layer's [ArtEntry.tintMarkers] hold its marker per palette entry. */
    val art: Map<String, ArtEntry> = emptyMap(),
    /** Tint palettes keyed by tint id; see [WindowEntry.tints]. */
    val tints: Map<String, Map<String, String>> = emptyMap(),
    /** Every slot, switch, and art layer in authored tree order: the order they compose above [static]. */
    val layers: List<LayerEntry> = emptyList(),
    /**
     * The marker color each of [layers] is sent in, keyed by name. A tinted slot or art layer is instead sent in the
     * `tintMarkers` entry of its current palette entry.
     */
    val markers: Map<String, String> = emptyMap(),
    /** The marker color [FullscreenCanvasEntry.backdrop] is sent in; `null` when the window shows the world. */
    @SerialName("backdrop_marker") val backdropMarker: String? = null,
)

/** Full-screen surface metadata. */
@Serializable
public data class FullscreenSurfaceEntry(
    /** Surface kind discriminator (always `"fullscreen"`). */
    val kind: String,
    /** How the cursor follows the mouse. */
    val mode: FullscreenMode,
    /** Canvas size `[width, height]` in canvas pixels. */
    val size: List<Int>,
)

/** How a full-screen window's cursor follows the player's mouse. */
@Serializable
public enum class FullscreenMode {
    /** The camera stays on the player, and the text shader reads the cursor from the camera rotation. */
    @SerialName("unlocked")
    UNLOCKED,

    /** The view is locked, and the entity shader reads the cursor from a client-steered horse's armour. */
    @SerialName("horse")
    HORSE,
}

/** A clickable or hoverable rect of a full-screen window, in canvas pixels. */
@Serializable
public data class ControlEntry(
    /** The control's index in the shader include's `WINDOW_FS_CONTROLS`, unique across the pack. */
    val index: Int,
    val x: Int,
    val y: Int,
    val width: Int,
    val height: Int,
    /** The action id a click names; see [RegionEntry.action]. `null` for a hover-only control. */
    val action: String? = null,
    /** The runtime action, such as `window:close`, run when no handler is bound to [action]. */
    @SerialName("default_action") val defaultAction: String? = null,
    /** Whether the control is outlined while the cursor is over it. */
    val hover: Boolean,
    /**
     * The hover outline, net-zero from the line origin, sent in [FullscreenCanvasEntry.overlayMarker] while the cursor
     * is over the control; present when [hover] is set.
     */
    val outline: String? = null,
    /** The authored element this control comes from, for diagnostics. */
    val source: String? = null,
)

/**
 * The full-screen canvas contract, shared with the generated `window_fullscreen.glsl` include.
 *
 * The session faces the player to yaw and pitch 0 on open. The cursor's canvas point for a rotation from there wraps
 * yaw into `[-180, 180)`, then takes `floor(size / 2 + angle / degreesPerPixel + snapEpsilon)` per axis, clamped to
 * the canvas; the shader computes the same. A click hits a control within [hitMargin] pixels of it.
 *
 * Runs are sent as the single line of a text display with text opacity [markerAlpha], scale [displayScale], left
 * alignment, no shadow, no line wrapping, and a fixed billboard facing yaw 0. A glyph's canvas position is its
 * position in that line from the line origin, `y` down from the line's top. Each run's text color is a marker the
 * compiler computed: send the markers the entries list.
 */
@Serializable
public data class FullscreenCanvasEntry(
    /** Canvas width in canvas pixels. */
    val width: Int,
    /** Canvas height in canvas pixels. */
    val height: Int,
    /** Degrees of yaw or pitch per canvas pixel. */
    @SerialName("degrees_per_pixel") val degreesPerPixel: Double,
    /** The yaw from 0 at which the cursor reaches the left or right canvas edge. */
    @SerialName("edge_yaw") val edgeYaw: Double,
    /** The pitch from 0 at which the cursor reaches the top or bottom canvas edge. */
    @SerialName("edge_pitch") val edgePitch: Double,
    /** Added to each cursor coordinate before flooring, so both sides snap pixel boundaries alike. */
    @SerialName("snap_epsilon") val snapEpsilon: Double,
    /** Canvas pixels a hit-test grows each control by. */
    @SerialName("hit_margin") val hitMargin: Int,
    /** The uniform scale of the text display carrying the runs. */
    @SerialName("display_scale") val displayScale: Double,
    /** The text opacity of the text display carrying the runs. */
    @SerialName("marker_alpha") val markerAlpha: Int,
    /** The built-in cursor, sent from the line origin. */
    val cursor: CursorEntry,
    /** The built-in backdrop glyph, net-zero from the line origin; send it in a window's [FullscreenEntry.backdropMarker]. */
    val backdrop: String,
    /** The marker of the runtime's own overlays, such as hover outlines: above every window layer, drawn white. */
    @SerialName("overlay_marker") val overlayMarker: String,
    /** The world age the server sends while the HUD is hidden, which makes the shaders' `GameTime` negative. */
    @SerialName("hud_hidden_world_age") val hudHiddenWorldAge: Long,
    /** The HUD strip's height in GUI pixels, from the bottom of the screen. */
    @SerialName("hud_strip_height") val hudStripHeight: Int,
    /** Half the HUD strip's width in GUI pixels, either side of the screen's center. */
    @SerialName("hud_strip_half_width") val hudStripHalfWidth: Int,
    /** The most max health the client is told while the HUD is hidden, so its hearts fit the strip. */
    @SerialName("hud_max_health") val hudMaxHealth: Int,
    /** The horse of horse-mode windows, present when the pack has one. */
    val horse: HorseEntry? = null,
)

/**
 * The horse that horse-mode full-screen windows read the cursor from. The player rides it while their camera spectates
 * a fixed entity at their eye, and the pack's entity shader draws the cursor from its armour.
 */
@Serializable
public data class HorseEntry(
    /** Where the horse stands relative to the camera `[x, y, z]`, in blocks, with the camera facing yaw and pitch 0. */
    val offset: List<Double>,
    /** The equipment asset id of the cursor armour the horse wears in its body slot. */
    val armor: String,
)

/** The built-in full-screen cursor glyph. Its top-left texel is the hot spot. */
@Serializable
public data class CursorEntry(
    /** The cursor glyph, net-zero from the line origin, in [WindowManifest.font]. */
    val text: String,
    /** The marker color [text] is sent in. */
    val marker: String,
    /** Width in canvas pixels. */
    val width: Int,
    /** Height in canvas pixels. */
    val height: Int,
)
