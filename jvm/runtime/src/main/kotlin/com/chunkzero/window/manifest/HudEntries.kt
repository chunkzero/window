package com.chunkzero.window.manifest

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** A single HUD: its fallback channel, baked static segment, text slots, and switches. */
@Serializable
public data class HudEntry(
    /** HUD surface/channel metadata. */
    val surface: HudSurfaceEntry,
    /** Baked static string, sent in [WindowManifest.font] before slot overlays. */
    @SerialName("static") val static: String,
    /** Dynamic text slots, including static labels (which carry [SlotEntry.text]). */
    val slots: Map<String, SlotEntry> = emptyMap(),
    /** Optional generated core-shader relocation metadata. */
    val shader: HudShaderEntry? = null,
    /** Runtime-selected cases keyed by switch key; see [WindowEntry.switches]. */
    val switches: Map<String, SwitchEntry> = emptyMap(),
    /** Every slot and switch in authored tree order; see [WindowEntry.layers]. */
    val layers: List<LayerEntry> = emptyList(),
)

/** HUD fallback channel and fixed canvas size. */
@Serializable
public data class HudSurfaceEntry(
    /** Surface kind discriminator (v1: always `"hud"`). */
    val kind: String,
    /** Vanilla transport channel: `"actionbar"`, `"bossbar"`, or `"sidebar"`. */
    val channel: String,
    /** Fixed line width in GUI pixels. */
    val width: Int,
    /** Informational canvas height in GUI pixels. */
    val height: Int,
)

/** Optional generated core-shader relocation metadata. */
@Serializable
public data class HudShaderEntry(
    /** Marker color used for the baked static HUD segment. */
    @SerialName("static_marker") val staticMarker: String = "#fefefe",
    /** Source surface top from the bottom of the vanilla HUD channel. */
    @SerialName("source_bottom") val sourceBottom: Int,
    /** Normalized target origin in GUI space: 0.0 = left, 1.0 = right. */
    @SerialName("origin_x") val originX: Double = 0.5,
    /** Normalized target origin in GUI space: 0.0 = top, 1.0 = bottom. */
    @SerialName("origin_y") val originY: Double = 1.0,
    /** Normalized point inside the HUD surface placed on the target origin. */
    @SerialName("anchor_x") val anchorX: Double = 0.5,
    /** Normalized point inside the HUD surface placed on the target origin. */
    @SerialName("anchor_y") val anchorY: Double = 0.0,
    /** Horizontal target-origin nudge in GUI pixels. */
    @SerialName("offset_x") val offsetX: Int = 0,
    /** Vertical target-origin nudge in GUI pixels. */
    @SerialName("offset_y") val offsetY: Int = -sourceBottom,
)
