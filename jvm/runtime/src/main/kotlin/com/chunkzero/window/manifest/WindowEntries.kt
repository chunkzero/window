package com.chunkzero.window.manifest

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/**
 * A single window: its surface, baked chrome, and primitives: text and sprite slots, inventory
 * regions, items, collections, inputs, and switches whose cases hold any of these.
 */
@Serializable
public data class WindowEntry(
    /** Surface (container) metadata. */
    val surface: SurfaceEntry,
    /**
     * Net-zero baked chrome string, sent verbatim in [WindowManifest.font] before every layer.
     */
    @SerialName("static") val static: String,
    /** Dynamic text slots, including static labels (which carry [SlotEntry.text]). */
    val slots: Map<String, SlotEntry> = emptyMap(),
    /** Runtime sprite regions keyed by name. */
    @SerialName("sprite_slots") val spriteSlots: Map<String, SpriteSlotEntry> = emptyMap(),
    /** Inventory regions keyed by name. */
    val regions: Map<String, RegionEntry> = emptyMap(),
    /** Dynamic inventory item regions keyed by name. */
    val items: Map<String, ItemEntry> = emptyMap(),
    /** Dynamic repeated inventory item regions keyed by name. */
    val collections: Map<String, CollectionEntry> = emptyMap(),
    /** Native anvil rename-field bindings keyed by name. */
    val inputs: Map<String, AnvilInputEntry> = emptyMap(),
    /** Group metadata for flattened repeater controls. */
    val groups: Map<String, RepeatGroupEntry> = emptyMap(),
    /** Runtime-selected cases keyed by switch key. */
    val switches: Map<String, SwitchEntry> = emptyMap(),
    /**
     * Every slot, sprite slot, switch, and collection with a selected sprite, once each, in authored
     * tree order: the order runtime layers compose above [static].
     */
    val layers: List<LayerEntry> = emptyList(),
)

/**
 * Cases of which at most one is active: the one whose value the switch's binding returns, else
 * [initial]. A switch listed by a case's [SwitchCaseEntry.switches] is active only while that case is.
 */
@Serializable
public data class SwitchEntry(
    /** Cases in authoring order. */
    val cases: List<SwitchCaseEntry>,
    /**
     * The binding name when this switch is one case's copy of a binding shared across mutually
     * exclusive cases, keyed `{binding}.{case path}`; `null` when the key is the binding name.
     */
    val binding: String? = null,
    /**
     * Whether this switch selects the named states of the button or hotspot it is keyed by. It is
     * selected through `buttonState` and its helpers, not `switch`.
     */
    val states: Boolean = false,
    /** The case active until the switch is bound; `null` for none. */
    val initial: String? = null,
    /** The authored element this switch comes from, such as ``button `buy` ``, for diagnostics. */
    val source: String? = null,
)

/** One case of a [SwitchEntry]; it lists only the entries directly inside it. */
@Serializable
public data class SwitchCaseEntry(
    /** Value the switch binding returns to select this case. */
    val value: String,
    /**
     * Net-zero baked art of this case in [WindowManifest.font], drawn at its switch's position in
     * the layers. Window cases start and end at the title origin; HUD cases at the HUD's left edge.
     */
    @SerialName("static") val static: String = "",
    /** Text slots, including static labels, drawn only while this case is active. */
    val slots: List<String> = emptyList(),
    /** Sprite slots drawn only while this case is active. */
    @SerialName("sprite_slots") val spriteSlots: List<String> = emptyList(),
    /** Inventory regions claimed only while this case is active. */
    val regions: List<String> = emptyList(),
    /** Nested switches, active only while this case is active. */
    val switches: List<String> = emptyList(),
)

/** One runtime layer: an entry composed above the static chrome, in authored tree order. */
@Serializable
public data class LayerEntry(
    /** The kind of entry [name] refers to. */
    val kind: LayerKind,
    /** The entry's key in its map. */
    val name: String,
)

/** What a [LayerEntry] draws. */
@Serializable
public enum class LayerKind {
    /** A text slot or static label. */
    @SerialName("slot")
    SLOT,

    /** A fixed or bound sprite slot. */
    @SerialName("sprite_slot")
    SPRITE_SLOT,

    /** The baked art of a switch's active case. */
    @SerialName("switch")
    SWITCH,

    /** A collection's selected-cell sprite. */
    @SerialName("collection")
    COLLECTION,
}

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
     * The binding name when this slot is one case's copy of a binding shared across mutually
     * exclusive cases, keyed `{binding}.{case path}`; `null` when the key is the binding name.
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
     * The binding name when this sprite slot is one case's copy of a binding shared across mutually
     * exclusive cases, keyed `{binding}.{case path}`; `null` when the key is the binding name.
     */
    val binding: String? = null,
)
