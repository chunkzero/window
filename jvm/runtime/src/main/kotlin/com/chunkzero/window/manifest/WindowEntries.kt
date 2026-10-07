package com.chunkzero.window.manifest

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** A single window: its surface, baked chrome, dynamic/static slots, and clickable buttons. */
@Serializable
public data class WindowEntry(
    /** Surface (container) metadata. */
    val surface: SurfaceEntry,
    /**
     * Net-zero baked chrome string, sent verbatim in [WindowManifest.font] before slot segments.
     */
    @SerialName("static") val static: String,
    /** Dynamic text slots, including static labels (which carry [SlotEntry.text]). */
    val slots: Map<String, SlotEntry> = emptyMap(),
    /** Runtime sprite regions keyed by name. */
    @SerialName("sprite_slots") val spriteSlots: Map<String, SpriteSlotEntry> = emptyMap(),
    /** Clickable regions mapped to container inventory slot indices. */
    val buttons: Map<String, ButtonEntry> = emptyMap(),
    /** Dynamic inventory item regions keyed by name. */
    val items: Map<String, ItemEntry> = emptyMap(),
    /** Dynamic repeated inventory item regions keyed by name. */
    val collections: Map<String, CollectionEntry> = emptyMap(),
    /** Native anvil rename-field bindings keyed by name. */
    val inputs: Map<String, AnvilInputEntry> = emptyMap(),
    /** Non-binding slot claims/fills keyed by name. */
    @SerialName("slot_rects") val slotRects: Map<String, SlotRectEntry> = emptyMap(),
    /** Group metadata for flattened repeater controls. */
    val groups: Map<String, RepeatGroupEntry> = emptyMap(),
    /** Runtime-selected visual cases keyed by binding name. */
    val switches: Map<String, SwitchEntry> = emptyMap(),
)

/** Visual cases of which the runtime draws only the one its binding selects. */
@Serializable
public data class SwitchEntry(
    /** Cases in authoring order. */
    val cases: List<SwitchCaseEntry>,
)

/** One case of a [SwitchEntry]. */
@Serializable
public data class SwitchCaseEntry(
    /** Value the switch binding returns to select this case. */
    val value: String,
    /**
     * Net-zero baked art of this case, sent in [WindowManifest.font] after the static segment. Window
     * cases start and end at the title origin; HUD cases at the HUD's left edge.
     */
    @SerialName("static") val static: String = "",
    /** Text slots, including static labels, drawn only while this case is active. */
    val slots: List<String> = emptyList(),
    /** Sprite slots drawn only while this case is active. */
    @SerialName("sprite_slots") val spriteSlots: List<String> = emptyList(),
)

/** Surface metadata: the container kind, GUI size, and title cursor origin. */
@Serializable
public data class SurfaceEntry(
    /** Surface kind discriminator (v1: always `"container"`). */
    val kind: String,
    /** window-core container kind id, e.g. `"generic_9x6"`. */
    val container: String,
    /** GUI texture size `[width, height]` in pixels. */
    val size: List<Int>,
    /** Where the title cursor starts `[x, y]` in GUI pixels. */
    @SerialName("title_origin") val titleOrigin: List<Int>,
)

/** A dynamic text slot or static label. */
@Serializable
public data class SlotEntry(
    /** Top-left x of the text line in GUI pixels. */
    val x: Int,
    /**
     * Top-left y of the text line in GUI pixels (8px tall). Informational; placement is in [font].
     */
    val y: Int,
    /** Reserved width for alignment/clipping. */
    val width: Int,
    /** Horizontal alignment within [width]. */
    val align: Align,
    /** Shifted font for this slot's y offset, e.g. `"window:y0"`. */
    val font: String,
    /** Default text color (hex, lowercase), e.g. `"#404040"`. */
    val color: String,
    /** Whether the text renders with a drop shadow. */
    val shadow: Boolean,
    /** Whether bold is enabled by default. */
    val bold: Boolean = false,
    /** Whether italic is enabled by default. */
    val italic: Boolean = false,
    /** Whether underline is enabled by default. */
    val underlined: Boolean = false,
    /** Whether strikethrough is enabled by default. */
    val strikethrough: Boolean = false,
    /** Whether obfuscated text is enabled by default. */
    val obfuscated: Boolean = false,
    /** Present ⇒ static label, auto-rendered; absent (`null`) ⇒ dynamic slot, bound at runtime. */
    val text: String? = null,
    /** Optional generated marker color used by generated HUD shaders. */
    @SerialName("shader_marker") val shaderMarker: String? = null,
    /** Optional legacy near-identical marker color used by older generated HUD shaders. */
    @SerialName("shader_color") val shaderColor: String? = null,
    /**
     * The binding name when this slot is one case's copy of a binding shared across a switch's cases,
     * keyed `{binding}.{case}`; `null` when the key is the binding name.
     */
    val binding: String? = null,
    /** How content wider than [width] is shortened; `null` leaves it untouched. */
    val overflow: TextOverflow? = null,
    /** The lines content wraps onto; `null` for a single line. */
    val lines: SlotLinesEntry? = null,
)

/** How a text slot shortens content wider than its width. */
@Serializable
public enum class TextOverflow {
    /** Truncate and end with an ellipsis. */
    @SerialName("ellipsis")
    ELLIPSIS,
}

/** The lines a multi-line text slot wraps onto, vertically centered in its box. */
@Serializable
public data class SlotLinesEntry(
    /** Maximum number of lines. */
    val count: Int,
    /** Distance between the tops of consecutive lines, in GUI pixels. */
    @SerialName("line_height") val lineHeight: Int,
    /**
     * Shifted fonts by half-line step: `fonts[s]` draws a line whose top is
     * `y + floor(s * lineHeight / 2)`. With `k` lines used, line `j` uses `s = count - k + 2j`.
     */
    val fonts: List<String>,
)

/** One runtime-renderable sprite in the compiled pack. */
@Serializable
public data class SpriteEntry(
    /** Rendered sprite width in GUI pixels. */
    val width: Int,
    /** Rendered sprite height in GUI pixels. */
    val height: Int,
    /** Left transparent padding before visible sprite ink, in GUI pixels. */
    @SerialName("x_offset") val xOffset: Int = 0,
    /** Visible ink width used for visual alignment, in GUI pixels. */
    @SerialName("glyph_width") val glyphWidth: Int = width,
    /** Cursor advance of this bitmap glyph in the generated sprite fonts. */
    val advance: Int = width + 1,
    /** Allocated glyph as a one-character string. */
    val glyph: String,
)

/** A runtime sprite region in a window. */
@Serializable
public data class SpriteSlotEntry(
    /** Top-left x of the reserved sprite region in GUI pixels. */
    val x: Int,
    /** Top-left y of the reserved sprite region in GUI pixels. */
    val y: Int,
    /** Reserved width for alignment. */
    val width: Int,
    /** Reserved height, informational and used by tooling. */
    val height: Int,
    /** Horizontal alignment within [width]. */
    val align: Align,
    /** Generated sprite font for this region's vertical offset. */
    val font: String,
    /** Fixed sprite id, or `null` when this slot must be bound by the view. */
    val sprite: String? = null,
    /**
     * The binding name when this sprite slot is one case's copy of a binding shared across a switch's
     * cases, keyed `{binding}.{case}`; `null` when the key is the binding name.
     */
    val binding: String? = null,
)
