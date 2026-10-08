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

/** One HUD layer to compose: a rendered slot, or a switch case's net-zero art from the HUD's left edge. */
internal sealed interface HudLayer {
    class Slot(
        val segment: RenderedSegment,
    ) : HudLayer

    class Art(
        val semanticId: String,
        val static: String,
        val source: String?,
    ) : HudLayer
}

/** Composes a HUD [Component] from baked static chrome plus layer segments in authored order. */
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
        val lines =
            fonts.slotLines(slot, content, slotStyle(slot, color, shadowColor)).map { line ->
                if (shaderHud) {
                    SlotLine(
                        forceColor(line.component, color, slot.shadow, shadowColor),
                        line.font,
                        line.y,
                    )
                } else {
                    line
                }
            }
        val placed = fonts.placeLines(slot, lines, ::spacer) ?: return null
        return RenderedSegment(
            component = placed.component,
            trace =
                textSlotTrace(
                    semanticId = semanticId,
                    kind = RenderLayerKind.HUD_TEXT,
                    slot = slot,
                    styled = lines.first().component,
                    colorHex = color.asHexString(),
                    placed = placed,
                    cursor = 0,
                ),
        )
    }

    /** [value] shortened with an ellipsis so that it, followed by [suffix], fits [slot]; followed by [suffix]. */
    fun fit(
        slot: SlotEntry,
        value: Component,
        suffix: Component,
    ): Component = fonts.fitText(value, suffix, slotStyle(slot, requireColor(slot.color)), slot.width)

    /**
     * Composes the static segment followed by [layers] in order. Without a shader, slots chain from the
     * HUD's right edge, where the cursor ends; with one, every layer returns to the HUD origin.
     */
    fun compose(
        hudName: String,
        layers: List<HudLayer>,
    ): ComposedRender {
        val base = if (shaderHud) 0 else originX
        val traces = arrayListOf(staticTrace("hud/$hudName/static", hud.static, originX))
        val parts = ArrayList<Component>()
        var cursor = base
        for (layer in layers) {
            when (layer) {
                is HudLayer.Art -> {
                    parts.addSpacer(-cursor)
                    parts += Component.text(layer.static).style(spacerStyle)
                    cursor = 0
                    traces += staticTrace(layer.semanticId, layer.static, 0).copy(source = layer.source)
                }

                is HudLayer.Slot -> {
                    val trace = layer.segment.trace
                    val start = cursor
                    parts.addSpacer(trace.contentCursorStart - cursor)
                    parts += layer.segment.component
                    cursor = trace.contentCursorEnd
                    if (shaderHud) {
                        parts.addSpacer(-cursor)
                        cursor = 0
                    }
                    traces += trace.copy(cursorStart = start, cursorEnd = cursor, netCursorDelta = cursor - start)
                }
            }
        }
        parts.addSpacer(base - cursor)
        return ComposedRender(staticComponent.appendAll(parts), traces)
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

    private fun MutableList<Component>.addSpacer(offset: Int) {
        spacer(offset)?.let(::add)
    }

    private fun spacer(offset: Int): Component? =
        spacers.compose(offset).takeIf { it.isNotEmpty() }?.let { Component.text(it).style(spacerStyle) }

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
