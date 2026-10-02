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
