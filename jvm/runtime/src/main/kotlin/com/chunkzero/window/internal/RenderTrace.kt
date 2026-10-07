package com.chunkzero.window.internal

import com.chunkzero.window.diagnostics.RenderLayerKind
import com.chunkzero.window.diagnostics.RenderLayerTrace
import com.chunkzero.window.diagnostics.RenderStyleTrace
import com.chunkzero.window.manifest.SlotEntry
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.TextComponent
import net.kyori.adventure.text.format.ShadowColor
import net.kyori.adventure.text.format.Style
import net.kyori.adventure.text.format.TextDecoration

internal data class RenderedSegment(
    val component: Component,
    val trace: RenderLayerTrace,
)

internal data class ComposedRender(
    val component: Component,
    val layers: List<RenderLayerTrace>,
)

/**
 * Trace for a net-zero text slot [placed] with its first line styled as [styled], with the cursor at [cursor] before
 * and after.
 */
internal fun textSlotTrace(
    semanticId: String,
    kind: RenderLayerKind,
    slot: SlotEntry,
    styled: Component,
    colorHex: String,
    placed: PlacedText,
    cursor: Int,
): RenderLayerTrace =
    RenderLayerTrace(
        semanticId = semanticId,
        kind = kind,
        content = placed.content,
        font = styled.style().font()?.asString() ?: slot.font,
        style =
            traceStyle(
                styled.style(),
                RenderStyleTrace(
                    color = colorHex,
                    shadow = slot.shadow,
                    bold = slot.bold,
                    italic = slot.italic,
                    underlined = slot.underlined,
                    strikethrough = slot.strikethrough,
                    obfuscated = slot.obfuscated,
                ),
            ),
        expectedBounds = placed.bounds,
        cursorStart = cursor,
        contentCursorStart = placed.start,
        contentCursorEnd = placed.end,
        cursorEnd = cursor,
        advance = placed.end - placed.start,
        visualWidth = placed.bounds.width,
        netCursorDelta = 0,
    )

internal fun plainContent(component: Component): String = buildString { appendContent(component) }

private fun StringBuilder.appendContent(component: Component) {
    if (component is TextComponent) append(component.content())
    for (child in component.children()) appendContent(child)
}

internal fun traceStyle(
    style: Style,
    fallback: RenderStyleTrace,
): RenderStyleTrace =
    RenderStyleTrace(
        color = style.color()?.asHexString() ?: fallback.color,
        shadow = style.shadowColor()?.let { it != ShadowColor.none() } ?: fallback.shadow,
        bold = style.decorationEnabled(TextDecoration.BOLD, fallback.bold),
        italic = style.decorationEnabled(TextDecoration.ITALIC, fallback.italic),
        underlined = style.decorationEnabled(TextDecoration.UNDERLINED, fallback.underlined),
        strikethrough =
            style.decorationEnabled(TextDecoration.STRIKETHROUGH, fallback.strikethrough),
        obfuscated = style.decorationEnabled(TextDecoration.OBFUSCATED, fallback.obfuscated),
    )

private fun Style.decorationEnabled(
    decoration: TextDecoration,
    fallback: Boolean,
): Boolean =
    when (decoration(decoration)) {
        TextDecoration.State.TRUE -> true
        TextDecoration.State.FALSE -> false
        TextDecoration.State.NOT_SET -> fallback
    }
