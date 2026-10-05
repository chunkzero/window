package com.chunkzero.window.internal

import com.chunkzero.window.diagnostics.RenderBounds
import com.chunkzero.window.diagnostics.RenderLayerKind
import com.chunkzero.window.diagnostics.RenderLayerTrace
import com.chunkzero.window.diagnostics.RenderStyleTrace
import com.chunkzero.window.manifest.SlotEntry
import com.chunkzero.window.manifest.SpriteSlotEntry
import com.chunkzero.window.manifest.WindowEntry
import com.chunkzero.window.manifest.WindowManifest
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor

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
    private val spacerStyle = baseStyle(font, NamedTextColor.WHITE)

    /** The static-chrome component (font applied), with no slot segments. */
    val staticComponent: Component = Component.text(window.static).style(spacerStyle)

    /**
     * Renders a single slot's net-zero segment, or `null` if [content] is empty.
     *
     * The segment is `spacer(xStart − originX)` + styled text + `spacer(−(dx + advanceWidth))`,
     * where `xStart` depends on the slot's alignment and the visible plain-text width of [content].
     */
    fun renderSlot(
        semanticId: String,
        slot: SlotEntry,
        content: Component,
    ): RenderedSegment? {
        val styled = content.applyFallbackStyle(slotStyle(slot, requireColor(slot.color)))
        val widths = fonts.measure(styled, slot.font)
        if (widths.advance == 0 && widths.visual == 0) return null

        val xStart = fonts.originFor(slot.align, slot.x, slot.width, widths.visual)
        val dx = xStart - originX
        return RenderedSegment(
            component = netZeroSegment(dx, widths.advance, styled),
            trace =
                textSlotTrace(
                    semanticId = semanticId,
                    kind = RenderLayerKind.TEXT_SLOT,
                    slot = slot,
                    styled = styled,
                    colorHex = slot.color,
                    xStart = xStart,
                    widths = widths,
                    cursor = originX,
                ),
        )
    }

    /** Renders a runtime sprite's net-zero segment, or `null` when [spriteName] is null. */
    fun renderSprite(
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
        val glyph = Component.text(sprite.glyph).style(baseStyle(slot.font, NamedTextColor.WHITE))
        return RenderedSegment(
            component = netZeroSegment(glyphStart - originX, sprite.advance, glyph),
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
    fun compose(
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

    private fun netZeroSegment(
        dx: Int,
        advance: Int,
        content: Component,
    ): Component {
        val lead = spacers.compose(dx)
        val trail = spacers.compose(-(dx + advance))
        var segment = Component.empty()
        if (lead.isNotEmpty()) segment = segment.append(Component.text(lead).style(spacerStyle))
        segment = segment.append(content)
        if (trail.isNotEmpty()) segment = segment.append(Component.text(trail).style(spacerStyle))
        return segment
    }
}
