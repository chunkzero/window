package dev.oglass.window.internal

import dev.oglass.window.diagnostics.RenderBounds
import dev.oglass.window.diagnostics.RenderLayerKind
import dev.oglass.window.diagnostics.RenderLayerTrace
import dev.oglass.window.diagnostics.RenderStyleTrace
import dev.oglass.window.manifest.SlotEntry
import dev.oglass.window.manifest.SpriteSlotEntry
import dev.oglass.window.manifest.WindowEntry
import dev.oglass.window.manifest.WindowManifest
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.ShadowColor
import net.kyori.adventure.text.format.Style
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.format.TextDecoration

/**
 * Composes a window title [Component] from the baked static chrome plus per-slot net-zero segments.
 *
 * The static string is sent verbatim in the manifest's main [WindowManifest.font]; it already
 * begins and ends at `title_origin.x` (net-zero). Each slot fill is bracketed by spacers so the
 * cursor returns to `title_origin.x` afterwards, letting segments compose in any order. Slot text
 * uses the slot's shifted font and color as fallbacks. Runtime-supplied Adventure component styling
 * is preserved, and width is measured from each text run's effective font/bold state.
 */
internal class TitleComposer(
    manifest: WindowManifest,
    private val window: WindowEntry,
) {
    private val font: String = manifest.font
    private val sprites = manifest.sprites
    private val originX: Int = window.surface.titleOrigin[0]
    private val spacers = Spacers(manifest.spacers)
    private val fonts = FontRegistry.fromManifest(manifest)

    /** The static-chrome component (font applied), with no slot segments. */
    val staticComponent: Component =
        Component.text(window.static).style(baseStyle(font, NamedTextColor.WHITE))

    /**
     * Renders a single slot's net-zero segment as a [Component], or `null` if [content] is empty.
     *
     * The segment is `spacer(xStart − originX)` + styled text + `spacer(−(dx + advanceWidth))`,
     * where `xStart` depends on the slot's alignment and the visible plain-text width of [content].
     */
    fun renderSlot(
        slot: SlotEntry,
        content: Component,
    ): Component? = renderSlot("slot", slot, content)?.component

    internal fun renderSlot(
        semanticId: String,
        slot: SlotEntry,
        content: Component,
    ): RenderedSegment? {
        val styled = content.applyFallbackStyle(slotStyle(slot, requireColor(slot.color)))
        val widths = fonts.measure(styled, slot.font)
        if (widths.advance == 0 && widths.visual == 0) return null

        val xStart = fonts.originFor(slot.align, slot.x, slot.width, widths.visual)
        val dx = xStart - originX

        val lead = spacers.compose(dx)
        val trail = spacers.compose(-(dx + widths.advance))

        var segment = Component.empty()
        if (lead.isNotEmpty()) {
            segment =
                segment.append(Component.text(lead).style(baseStyle(font, NamedTextColor.WHITE)))
        }
        segment = segment.append(styled)
        if (trail.isNotEmpty()) {
            segment =
                segment.append(Component.text(trail).style(baseStyle(font, NamedTextColor.WHITE)))
        }
        return RenderedSegment(
            component = segment,
            trace =
                RenderLayerTrace(
                    semanticId = semanticId,
                    kind = RenderLayerKind.TEXT_SLOT,
                    content = plainContent(styled),
                    font = styled.style().font()?.asString() ?: slot.font,
                    style =
                        traceStyle(
                            styled.style(),
                            RenderStyleTrace(
                                color = slot.color,
                                shadow = slot.shadow,
                                bold = slot.bold,
                                italic = slot.italic,
                                underlined = slot.underlined,
                                strikethrough = slot.strikethrough,
                                obfuscated = slot.obfuscated,
                            ),
                        ),
                    expectedBounds = RenderBounds(xStart, slot.y, widths.visual, TEXT_HEIGHT),
                    cursorStart = originX,
                    contentCursorStart = xStart,
                    contentCursorEnd = xStart + widths.advance,
                    cursorEnd = originX,
                    advance = widths.advance,
                    visualWidth = widths.visual,
                    netCursorDelta = 0,
                ),
        )
    }

    /** Renders a runtime sprite's net-zero segment, or `null` when [spriteName] is null. */
    fun renderSprite(
        slot: SpriteSlotEntry,
        spriteName: String?,
    ): Component? = renderSprite("sprite", slot, spriteName)?.component

    internal fun renderSprite(
        semanticId: String,
        slot: SpriteSlotEntry,
        spriteName: String?,
    ): RenderedSegment? {
        if (spriteName == null) return null
        val sprite =
            sprites[spriteName]
                ?: throw IllegalArgumentException(
                    "Unknown Window sprite '$spriteName'; known sprites: ${sprites.keys.sorted()}",
                )

        val xStart = fonts.originFor(slot.align, slot.x, slot.width, sprite.glyphWidth)
        val glyphStart = xStart - sprite.xOffset
        val dx = glyphStart - originX
        val lead = spacers.compose(dx)
        val trail = spacers.compose(-(dx + sprite.advance))

        var segment = Component.empty()
        if (lead.isNotEmpty()) {
            segment =
                segment.append(Component.text(lead).style(baseStyle(font, NamedTextColor.WHITE)))
        }
        segment =
            segment.append(
                Component.text(sprite.glyph).style(baseStyle(slot.font, NamedTextColor.WHITE)),
            )
        if (trail.isNotEmpty()) {
            segment =
                segment.append(Component.text(trail).style(baseStyle(font, NamedTextColor.WHITE)))
        }
        return RenderedSegment(
            component = segment,
            trace =
                RenderLayerTrace(
                    semanticId = semanticId,
                    kind = RenderLayerKind.SPRITE_SLOT,
                    content = sprite.glyph,
                    font = slot.font,
                    style = RenderStyleTrace(color = "#ffffff", shadow = false),
                    expectedBounds = RenderBounds(xStart, slot.y, sprite.glyphWidth, sprite.height),
                    cursorStart = originX,
                    contentCursorStart = glyphStart,
                    contentCursorEnd = glyphStart + sprite.advance,
                    cursorEnd = originX,
                    advance = sprite.advance,
                    visualWidth = sprite.glyphWidth,
                    netCursorDelta = 0,
                    spriteId = spriteName,
                    glyph = sprite.glyph,
                ),
        )
    }

    /**
     * Composes the full title: the static component followed by each already-rendered net-zero slot
     * segment, in the iteration order of [slotSegments].
     */
    fun compose(slotSegments: Map<String, Component>): Component {
        var title = staticComponent
        for (segment in slotSegments.values) {
            title = title.append(segment)
        }
        return title
    }

    internal fun compose(
        windowName: String,
        slotSegments: Map<String, RenderedSegment>,
    ): ComposedRender {
        var title = staticComponent
        for (segment in slotSegments.values) title = title.append(segment.component)
        val surfaceSize = window.surface.size
        val staticTrace =
            RenderLayerTrace(
                semanticId = "window/$windowName/static",
                kind = RenderLayerKind.STATIC_CHROME,
                content = window.static,
                font = font,
                style = RenderStyleTrace(color = "#ffffff", shadow = false),
                expectedBounds = RenderBounds(0, 0, surfaceSize[0], surfaceSize[1]),
                cursorStart = originX,
                contentCursorStart = originX,
                contentCursorEnd = originX,
                cursorEnd = originX,
                advance = 0,
                visualWidth = surfaceSize[0],
                netCursorDelta = 0,
            )
        return ComposedRender(title, listOf(staticTrace) + slotSegments.values.map { it.trace })
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
    ): Style =
        Style
            .style()
            .font(Key.key(slot.font))
            .color(color)
            .also { builder -> applyShadow(builder, slot.shadow) }
            .also { builder ->
                builder.decoration(TextDecoration.BOLD, slot.bold)
                builder.decoration(TextDecoration.ITALIC, slot.italic)
                builder.decoration(TextDecoration.UNDERLINED, slot.underlined)
                builder.decoration(TextDecoration.STRIKETHROUGH, slot.strikethrough)
                builder.decoration(TextDecoration.OBFUSCATED, slot.obfuscated)
            }.build()

    private fun requireColor(hex: String): TextColor =
        requireNotNull(TextColor.fromHexString(hex)) { "Invalid manifest text color $hex" }

    private fun applyShadow(
        builder: Style.Builder,
        shadow: Boolean,
    ) {
        builder.shadowColor(if (shadow) TEXT_SHADOW else ShadowColor.none())
    }

    private fun clearDecorations(builder: Style.Builder) {
        for (decoration in TEXT_DECORATIONS) {
            builder.decoration(decoration, false)
        }
    }

    private companion object {
        const val TEXT_HEIGHT = 8
        val TEXT_SHADOW: ShadowColor = ShadowColor.shadowColor(0, 0, 0, 180)

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
