package dev.oglass.window.internal

import dev.oglass.window.diagnostics.RenderBounds
import dev.oglass.window.diagnostics.RenderLayerKind
import dev.oglass.window.diagnostics.RenderLayerTrace
import dev.oglass.window.diagnostics.RenderStyleTrace
import dev.oglass.window.manifest.HudEntry
import dev.oglass.window.manifest.SlotEntry
import dev.oglass.window.manifest.WindowManifest
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.ShadowColor
import net.kyori.adventure.text.format.Style
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.format.TextDecoration

/** Composes a HUD [Component] from baked static chrome plus per-slot overlay segments. */
internal class HudComposer(
    manifest: WindowManifest,
    private val hud: HudEntry,
) {
    private val font: String = manifest.font
    private val originX: Int = hud.surface.width
    private val shaderHud: Boolean = hud.shader != null
    private val staticColor: TextColor =
        if (hud.shader == null) {
            NamedTextColor.WHITE
        } else {
            requireNotNull(TextColor.fromHexString(hud.shader.staticMarker))
        }
    private val spacers = Spacers(manifest.spacers)
    private val fonts = FontRegistry.fromManifest(manifest)

    /** The baked static HUD component, with no slot segments. */
    val staticComponent: Component = Component.text(hud.static).style(baseStyle(font, staticColor))

    /** Renders one styled slot text component, or `null` if [content] is empty. */
    fun renderSlot(
        slot: SlotEntry,
        content: Component,
    ): Component? = renderSlot("slot", slot, content)?.component

    internal fun renderSlot(
        semanticId: String,
        slot: SlotEntry,
        content: Component,
    ): RenderedSegment? {
        val color = requireColor(slot.shaderMarker ?: slot.shaderColor ?: slot.color)
        val shadowColor = slotShadowColor(color)
        val styled = content.applyFallbackStyle(slotStyle(slot, color, shadowColor))
        val renderable =
            if (shaderHud) forceColor(styled, color, slot.shadow, shadowColor) else styled
        val widths = fonts.measure(renderable, slot.font)
        if (widths.advance == 0 && widths.visual == 0) return null
        val xStart = fonts.originFor(slot.align, slot.x, slot.width, widths.visual)
        return RenderedSegment(
            component = renderable,
            trace =
                RenderLayerTrace(
                    semanticId = semanticId,
                    kind = RenderLayerKind.HUD_TEXT,
                    content = plainContent(renderable),
                    font = renderable.style().font()?.asString() ?: slot.font,
                    style =
                        traceStyle(
                            renderable.style(),
                            RenderStyleTrace(
                                color = color.asHexString(),
                                shadow = slot.shadow,
                                bold = slot.bold,
                                italic = slot.italic,
                                underlined = slot.underlined,
                                strikethrough = slot.strikethrough,
                                obfuscated = slot.obfuscated,
                            ),
                        ),
                    expectedBounds = RenderBounds(xStart, slot.y, widths.visual, TEXT_HEIGHT),
                    cursorStart = 0,
                    contentCursorStart = xStart,
                    contentCursorEnd = xStart + widths.advance,
                    cursorEnd = 0,
                    advance = widths.advance,
                    visualWidth = widths.visual,
                    netCursorDelta = 0,
                ),
        )
    }

    /** Appends already-rendered slot text after the static segment. */
    fun compose(slotSegments: Map<String, Component>): Component {
        var component = staticComponent
        var cursor = originX
        for ((name, segment) in orderedSegments(slotSegments)) {
            val slot = hud.slots.getValue(name)
            val widths = fonts.measure(segment, slot.font)
            val xStart = fonts.originFor(slot.align, slot.x, slot.width, widths.visual)

            component = component.appendSpacer(if (shaderHud) xStart else xStart - cursor)
            component = component.append(segment)
            if (shaderHud) {
                component = component.appendSpacer(-(xStart + widths.advance))
            } else {
                cursor = xStart + widths.advance
            }
        }
        return if (shaderHud) component else component.appendSpacer(originX - cursor)
    }

    internal fun compose(
        hudName: String,
        slotSegments: Map<String, RenderedSegment>,
    ): ComposedRender {
        var component = staticComponent
        var cursor = originX
        val traces = ArrayList<RenderLayerTrace>(slotSegments.size + 1)
        traces +=
            RenderLayerTrace(
                semanticId = "hud/$hudName/static",
                kind = RenderLayerKind.HUD_STATIC,
                content = hud.static,
                font = font,
                style = RenderStyleTrace(color = staticColor.asHexString(), shadow = false),
                expectedBounds = RenderBounds(0, 0, hud.surface.width, hud.surface.height),
                cursorStart = 0,
                contentCursorStart = 0,
                contentCursorEnd = originX,
                cursorEnd = originX,
                advance = originX,
                visualWidth = hud.surface.width,
                netCursorDelta = originX,
            )
        for ((name, rendered) in orderedRenderedSegments(slotSegments)) {
            val trace = rendered.trace
            val xStart = trace.expectedBounds.x
            val start = if (shaderHud) 0 else cursor
            component = component.appendSpacer(if (shaderHud) xStart else xStart - cursor)
            component = component.append(rendered.component)
            if (shaderHud) {
                component = component.appendSpacer(-(xStart + trace.advance))
                traces += trace.copy(cursorStart = 0, cursorEnd = 0, netCursorDelta = 0)
            } else {
                cursor = xStart + trace.advance
                traces +=
                    trace.copy(
                        cursorStart = start,
                        cursorEnd = cursor,
                        netCursorDelta = cursor - start,
                    )
            }
        }
        if (!shaderHud) component = component.appendSpacer(originX - cursor)
        return ComposedRender(component, traces)
    }

    private fun orderedRenderedSegments(
        slotSegments: Map<String, RenderedSegment>,
    ): List<Map.Entry<String, RenderedSegment>> =
        slotSegments.entries.sortedWith(
            compareBy<Map.Entry<String, RenderedSegment>> { it.value.trace.expectedBounds.x }
                .thenBy { entry -> hud.slots.getValue(entry.key).y }
                .thenBy { it.key },
        )

    private fun orderedSegments(slotSegments: Map<String, Component>): List<Map.Entry<String, Component>> =
        slotSegments.entries.sortedWith(
            compareBy<Map.Entry<String, Component>> { entry ->
                val slot = hud.slots.getValue(entry.key)
                val visualWidth = fonts.measure(entry.value, slot.font).visual
                fonts.originFor(slot.align, slot.x, slot.width, visualWidth)
            }.thenBy { entry -> hud.slots.getValue(entry.key).y }
                .thenBy { entry -> entry.key },
        )

    private fun Component.appendSpacer(offset: Int): Component {
        val text = spacers.compose(offset)
        if (text.isEmpty()) return this
        return append(Component.text(text).style(baseStyle(font, staticColor)))
    }

    private fun baseStyle(
        font: String,
        color: TextColor,
    ): Style =
        Style
            .style()
            .font(Key.key(font))
            .color(color)
            .shadowColor(ShadowColor.none())
            .also(::clearDecorations)
            .build()

    private fun slotStyle(
        slot: SlotEntry,
        color: TextColor,
        shadowColor: ShadowColor,
    ): Style =
        Style
            .style()
            .font(Key.key(slot.font))
            .color(color)
            .also { builder -> applyShadow(builder, slot.shadow, shadowColor) }
            .also { builder ->
                builder.decoration(TextDecoration.BOLD, slot.bold)
                builder.decoration(TextDecoration.ITALIC, slot.italic)
                builder.decoration(TextDecoration.UNDERLINED, slot.underlined)
                builder.decoration(TextDecoration.STRIKETHROUGH, slot.strikethrough)
                builder.decoration(TextDecoration.OBFUSCATED, slot.obfuscated)
            }.build()

    private fun forceColor(
        component: Component,
        color: TextColor,
        shadow: Boolean,
        shadowColor: ShadowColor,
    ): Component =
        component
            .children(component.children().map { forceColor(it, color, shadow, shadowColor) })
            .style { builder ->
                builder.color(color)
                applyShadow(builder, shadow, shadowColor)
            }

    private fun requireColor(hex: String): TextColor =
        requireNotNull(TextColor.fromHexString(hex)) { "Invalid manifest text color $hex" }

    private fun slotShadowColor(color: TextColor): ShadowColor =
        if (shaderHud) ShadowColor.shadowColor(color, TEXT_SHADOW_ALPHA) else TEXT_SHADOW

    private fun applyShadow(
        builder: Style.Builder,
        shadow: Boolean,
        shadowColor: ShadowColor,
    ) {
        builder.shadowColor(if (shadow) shadowColor else ShadowColor.none())
    }

    private fun clearDecorations(builder: Style.Builder) {
        for (decoration in TEXT_DECORATIONS) {
            builder.decoration(decoration, false)
        }
    }

    private companion object {
        const val TEXT_HEIGHT = 8
        const val TEXT_SHADOW_ALPHA = 180
        val TEXT_SHADOW: ShadowColor = ShadowColor.shadowColor(0, 0, 0, TEXT_SHADOW_ALPHA)

        val TEXT_DECORATIONS =
            listOf(
                TextDecoration.BOLD,
                TextDecoration.ITALIC,
                TextDecoration.UNDERLINED,
                TextDecoration.STRIKETHROUGH,
                TextDecoration.OBFUSCATED,
            )
    }
}
