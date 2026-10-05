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
)
