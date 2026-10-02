package dev.oglass.window.diagnostics

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
public enum class RenderLayerKind {
    @SerialName("static_chrome")
    STATIC_CHROME,

    @SerialName("text_slot")
    TEXT_SLOT,

    @SerialName("sprite_slot")
    SPRITE_SLOT,

    @SerialName("hud_static")
    HUD_STATIC,

    @SerialName("hud_text")
    HUD_TEXT,
}

/** One semantically-addressable server expectation within a [RenderFrame]. */
@Serializable
public data class RenderLayerTrace(
    @SerialName("semantic_id") public val semanticId: String,
    public val kind: RenderLayerKind,
    public val content: String,
    public val font: String,
    public val style: RenderStyleTrace,
    @SerialName("expected_bounds") public val expectedBounds: RenderBounds,
    @SerialName("cursor_start") public val cursorStart: Int,
    @SerialName("content_cursor_start") public val contentCursorStart: Int,
    @SerialName("content_cursor_end") public val contentCursorEnd: Int,
    @SerialName("cursor_end") public val cursorEnd: Int,
    public val advance: Int,
    @SerialName("visual_width") public val visualWidth: Int,
    @SerialName("net_cursor_delta") public val netCursorDelta: Int,
    @SerialName("sprite_id") public val spriteId: String? = null,
    public val glyph: String? = null,
) {
    internal fun validate() {
        require(semanticId.length in 1..512) { "Invalid semantic layer id" }
        require(content.length <= 65_536) { "Layer content is too large" }
        require(font.length in 1..256) { "Invalid layer font" }
        expectedBounds.validate()
        require(visualWidth >= 0) { "Visual width must be non-negative" }
        require(netCursorDelta == cursorEnd - cursorStart) { "Invalid net cursor delta" }
        require(contentCursorEnd - contentCursorStart == advance) { "Invalid content advance" }
        require(spriteId == null || spriteId.length in 1..256) { "Invalid sprite id" }
        require(glyph == null || glyph.codePointCount(0, glyph.length) == 1) {
            "Sprite glyph must be one codepoint"
        }
    }
}

@Serializable
public data class RenderBounds(
    public val x: Int,
    public val y: Int,
    public val width: Int,
    public val height: Int,
) {
    internal fun validate() {
        require(width >= 0 && height >= 0) { "Render bounds dimensions must be non-negative" }
    }
}

@Serializable
public data class RenderStyleTrace(
    public val color: String,
    public val shadow: Boolean,
    public val bold: Boolean = false,
    public val italic: Boolean = false,
    public val underlined: Boolean = false,
    public val strikethrough: Boolean = false,
    public val obfuscated: Boolean = false,
)
