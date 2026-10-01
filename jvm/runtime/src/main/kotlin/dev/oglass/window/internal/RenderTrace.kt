package dev.oglass.window.internal

import dev.oglass.window.diagnostics.RenderLayerTrace
import dev.oglass.window.diagnostics.RenderStyleTrace
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
