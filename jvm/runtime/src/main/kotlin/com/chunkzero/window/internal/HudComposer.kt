package com.chunkzero.window.internal

import com.chunkzero.window.diagnostics.RenderBounds
import com.chunkzero.window.diagnostics.RenderLayerKind
import com.chunkzero.window.diagnostics.RenderLayerTrace
import com.chunkzero.window.diagnostics.RenderStyleTrace
import com.chunkzero.window.manifest.HudEntry
import com.chunkzero.window.manifest.SlotEntry
import com.chunkzero.window.manifest.WindowManifest
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.ShadowColor
import net.kyori.adventure.text.format.TextColor

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
    private val spacerStyle = baseStyle(font, staticColor)

    /** The baked static HUD component, with no slot segments. */
    val staticComponent: Component = Component.text(hud.static).style(spacerStyle)

    /** Renders one styled slot text segment, or `null` if [content] is empty. */
    fun renderSlot(
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
                textSlotTrace(
                    semanticId = semanticId,
                    kind = RenderLayerKind.HUD_TEXT,
                    slot = slot,
                    styled = renderable,
                    colorHex = color.asHexString(),
                    xStart = xStart,
                    widths = widths,
                    cursor = 0,
                ),
        )
    }

    /**
     * Appends already-rendered slot segments after the static segment, ordered left to right. Each
     * [caseArt] entry, keyed by semantic id, is a net-zero segment from the HUD's left edge drawn
     * between the static segment and the slots.
     */
    fun compose(
        hudName: String,
        slotSegments: Map<String, RenderedSegment>,
        caseArt: Map<String, String> = emptyMap(),
    ): ComposedRender {
        val ordered = orderedRenderedSegments(slotSegments)
        val base = arrayListOf(staticTrace("hud/$hudName/static", hud.static, originX))
        caseArt.mapTo(base) { (id, art) -> staticTrace(id, art, 0) }
        val static = withCaseArt(caseArt.values)
        return if (shaderHud) composeShader(static, base, ordered) else composeFixedWidth(static, base, ordered)
    }

    /** The static component followed by each net-zero [art] segment, ending where the static ends. */
    private fun withCaseArt(art: Collection<String>): Component {
        if (art.isEmpty()) return staticComponent
        val end = if (shaderHud) 0 else originX
        var component = staticComponent.appendSpacer(-end)
        for (text in art) component = component.append(Component.text(text).style(spacerStyle))
        return component.appendSpacer(end)
    }

    /** Each segment is positioned from the HUD origin and returns to it independently. */
    private fun composeShader(
        static: Component,
        base: List<RenderLayerTrace>,
        ordered: List<Map.Entry<String, RenderedSegment>>,
    ): ComposedRender {
        var component = static
        val traces = ArrayList(base)
        for ((_, rendered) in ordered) {
            val trace = rendered.trace
            val xStart = trace.expectedBounds.x
            component =
                component
                    .appendSpacer(xStart)
                    .append(rendered.component)
                    .appendSpacer(-(xStart + trace.advance))
            traces += trace.copy(cursorStart = 0, cursorEnd = 0, netCursorDelta = 0)
        }
        return ComposedRender(component, traces)
    }

    /** Segments chain left to right from the HUD right edge, which the cursor returns to at the end. */
    private fun composeFixedWidth(
        static: Component,
        base: List<RenderLayerTrace>,
        ordered: List<Map.Entry<String, RenderedSegment>>,
    ): ComposedRender {
        var component = static
        var cursor = originX
        val traces = ArrayList(base)
        for ((_, rendered) in ordered) {
            val trace = rendered.trace
            val start = cursor
            component = component.appendSpacer(trace.expectedBounds.x - cursor).append(rendered.component)
            cursor = trace.expectedBounds.x + trace.advance
            traces += trace.copy(cursorStart = start, cursorEnd = cursor, netCursorDelta = cursor - start)
        }
        return ComposedRender(component.appendSpacer(originX - cursor), traces)
    }

    /** A static layer starting at the HUD's left edge whose cursor ends at [end]. */
    private fun staticTrace(
        semanticId: String,
        content: String,
        end: Int,
    ): RenderLayerTrace =
        RenderLayerTrace(
            semanticId = semanticId,
            kind = RenderLayerKind.HUD_STATIC,
            content = content,
            font = font,
            style = RenderStyleTrace(color = staticColor.asHexString(), shadow = false),
            expectedBounds = RenderBounds(0, 0, hud.surface.width, hud.surface.height),
            cursorStart = 0,
            contentCursorStart = 0,
            contentCursorEnd = end,
            cursorEnd = end,
            advance = end,
            visualWidth = hud.surface.width,
            netCursorDelta = end,
        )

    private fun orderedRenderedSegments(
        slotSegments: Map<String, RenderedSegment>,
    ): List<Map.Entry<String, RenderedSegment>> =
        slotSegments.entries.sortedWith(
            compareBy<Map.Entry<String, RenderedSegment>> { it.value.trace.expectedBounds.x }
                .thenBy { entry -> hud.slots.getValue(entry.key).y }
                .thenBy { it.key },
        )

    private fun Component.appendSpacer(offset: Int): Component {
        val text = spacers.compose(offset)
        if (text.isEmpty()) return this
        return append(Component.text(text).style(spacerStyle))
    }

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
                builder.applyShadow(shadow, shadowColor)
            }

    private fun slotShadowColor(color: TextColor): ShadowColor =
        if (shaderHud) ShadowColor.shadowColor(color, TEXT_SHADOW_ALPHA) else TEXT_SHADOW
}
