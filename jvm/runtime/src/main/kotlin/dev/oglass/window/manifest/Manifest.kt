package dev.oglass.window.manifest

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

/**
 * Compiled Window pack definition (schema v5).
 *
 * The rpp plugin generates Kotlin objects that instantiate these DTOs directly. JSON parsing
 * remains for compatibility with older tools and tests, but generated projects should load the
 * emitted `WindowPack` object instead of reading a file.
 *
 * All coordinates are GUI-space pixels (origin = container GUI top-left).
 */
@Serializable
public data class WindowManifest(
    /** Schema version. Only `5` is accepted by [parse]. */
    val version: Int,
    /** Resource-pack namespace (default `"window"`). */
    val namespace: String,
    /** Font id of the main (static + spacer) font, e.g. `"window:ui"`. */
    val font: String,
    /**
     * Spacer table: codepoint (integer) -> horizontal advance in pixels. Values are ±1, ±2, ±4 …
     * ±1024. Consumers compose arbitrary offsets greedily from these; never hardcode the table.
     */
    val spacers: Map<Int, Int>,
    /**
     * Vanilla default-font advance widths (pixels per character, including the 1px inter-glyph
     * gap). JSON keys are single-character strings. Applies to every shifted label font too.
     */
    @SerialName("text_advances") val textAdvances: Map<String, Int>,
    /**
     * Vanilla default-font visible glyph widths (pixels per character, excluding the inter-glyph
     * gap). JSON keys are single-character strings. Applies to every shifted label font too.
     */
    @SerialName("text_glyph_widths") val textGlyphWidths: Map<String, Int> = emptyMap(),
    /**
     * Font metrics keyed by font id. New manifests include entries for generated shifted Window
     * fonts plus `minecraft:default`; older manifests fall back to [textAdvances].
     */
    @SerialName("font_metrics") val fontMetrics: Map<String, FontMetricsEntry> = emptyMap(),
    /** Runtime sprite catalog keyed by sprite id. */
    val sprites: Map<String, SpriteEntry> = emptyMap(),
    /** Window definitions keyed by window name. */
    val windows: Map<String, WindowEntry>,
    /** HUD definitions keyed by HUD name. */
    val huds: Map<String, HudEntry> = emptyMap(),
) {
    public companion object {
        private val json =
            Json {
                ignoreUnknownKeys = true
                isLenient = false
            }

        /**
         * Parses a legacy JSON definition document.
         *
         * Unknown keys are ignored (forward compatibility). A definition whose [version] is not `5`
         * is rejected with an [IllegalArgumentException].
         */
        public fun parse(json: String): WindowManifest {
            val manifest = this.json.decodeFromString<WindowManifest>(json)
            require(manifest.version == 5) {
                "Unsupported Window definition version ${manifest.version}; this runtime only " +
                    "supports version 5"
            }
            return manifest
        }
    }
}

/** Per-font advance and visible-glyph metrics used by runtime alignment. */
@Serializable
public data class FontMetricsEntry(
    /** Cursor advance widths (pixels per character, including the inter-glyph gap). */
    val advances: Map<String, Int>,
    /** Visible glyph widths (pixels per character, excluding the inter-glyph gap). */
    @SerialName("glyph_widths") val glyphWidths: Map<String, Int> = emptyMap(),
    /** Extra cursor advance added per character when bold is active. */
    @SerialName("bold_advance") val boldAdvance: Int = 0,
)

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

/** A single HUD: its fallback channel, baked static segment, and text slots. */
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

/** Which backing inventory a control slot belongs to. */
@Serializable
public enum class SlotAreaEntry {
    @SerialName("container")
    CONTAINER,

    @SerialName("player")
    PLAYER,
}

/** A typed inventory slot owned by a window control. */
@Serializable
public data class SlotRefEntry(
    /** Backing inventory area. */
    val area: SlotAreaEntry,
    /** Slot index in that area. */
    val index: Int,
)

/** A clickable region mapped to typed inventory slots. */
@Serializable
public data class ButtonEntry(
    /** Top-left x of the button rect in GUI pixels. */
    val x: Int,
    /** Top-left y of the button rect in GUI pixels. */
    val y: Int,
    /** Button rect width in GUI pixels. */
    val width: Int,
    /** Button rect height in GUI pixels. */
    val height: Int,
    /** Backing inventory slots whose clicks route to this region. */
    val slots: List<SlotRefEntry>,
    /**
     * Slots this region fills with its own hitbox/state item.
     *
     * `null` means every slot in [slots]. A repeater cell that hands one of its slots to an item
     * control emits a strict subset here: clicks still route to the whole cell, but the yielded
     * slot carries the item control's real stack (and therefore its native hover tooltip).
     */
    @SerialName("fill_slots") val fillSlots: List<SlotRefEntry>? = null,
    /** Built-in behavior when no handler is bound, or `null` if a handler is required. */
    val default: ButtonDefault? = null,
    /** Whether this region should accept a generated/runtime click handler. */
    val action: Boolean = true,
    /** Default tooltip shown for this button or hotspot. */
    val tooltip: ButtonTooltip? = null,
    /** Named item states for dynamic visual/tooltip toggles. */
    val states: Map<String, ButtonState> = emptyMap(),
    /** Generated sprite font used by state sprites, when present. */
    @SerialName("sprite_font") val spriteFont: String? = null,
) {
    /** The slots this region paints with its own item: [fillSlots] when present, else [slots]. */
    public val filledSlots: List<SlotRefEntry>
        get() = fillSlots ?: slots
}

/** A dynamic inventory item region. */
@Serializable
public data class ItemEntry(
    /** Backing inventory slots this item control populates. */
    val slots: List<SlotRefEntry>,
)

/** A repeated dynamic inventory item region. */
@Serializable
public data class CollectionEntry(
    /** Backing inventory slots, one visible cell per slot in order. */
    val slots: List<SlotRefEntry>,
    /** Whether this collection accepts a click handler. */
    val action: Boolean = true,
)

/** A native anvil rename-field binding. */
@Serializable
public data class AnvilInputEntry(
    /** Input slot seeded to make the vanilla field editable. */
    val slot: SlotRefEntry,
    /** Initial field contents. */
    val initial: String = "",
    /** Optional item model used for the seed item. */
    @SerialName("item_model") val itemModel: String? = null,
)

/** A non-binding slot claim/fill region. */
@Serializable
public data class SlotRectEntry(
    /** Backing inventory slots claimed and cleared by this primitive. */
    val slots: List<SlotRefEntry>,
)

/** Group metadata for controls flattened from a repeater. */
@Serializable
public data class RepeatGroupEntry(
    /** Number of repeated cells. */
    val count: Int,
    /** Dynamic text slots by repeated child field, each vector in index order. */
    val slots: Map<String, List<String>> = emptyMap(),
    /** Runtime sprite slots by repeated child field, each vector in index order. */
    @SerialName("sprite_slots") val spriteSlots: Map<String, List<String>> = emptyMap(),
    /** Dynamic inventory item controls by repeated child field, each vector in index order. */
    val items: Map<String, List<String>> = emptyMap(),
    /** Root cell buttons, in index order. */
    val buttons: List<String> = emptyList(),
)

/** Plain manifest tooltip text. Runtime APIs can provide rich Adventure components. */
@Serializable
public data class ButtonTooltip(
    /** Tooltip title/name. */
    val title: String,
    /** Additional lore lines. */
    val lines: List<String> = emptyList(),
)

/** One named item state for a button. */
@Serializable
public data class ButtonState(
    /** Optional item model id, e.g. `"example:gui/shop_button_active"`. */
    @SerialName("item_model") val itemModel: String? = null,
    /** Optional Window sprite rendered over the button rect for this state. */
    val sprite: String? = null,
    /** Optional tooltip override for this state. */
    val tooltip: ButtonTooltip? = null,
)

/** Horizontal text alignment within a slot's reserved width. */
@Serializable
public enum class Align {
    @SerialName("left")
    LEFT,

    @SerialName("center")
    CENTER,

    @SerialName("right")
    RIGHT,
}

/** Built-in button behavior used when a manifest button has no user handler. */
@Serializable
public enum class ButtonDefault {
    @SerialName("close")
    CLOSE,
}
