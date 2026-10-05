package com.chunkzero.window.internal

import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.WindowManifest
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.KeybindComponent
import net.kyori.adventure.text.NBTComponent
import net.kyori.adventure.text.ObjectComponent
import net.kyori.adventure.text.ScoreComponent
import net.kyori.adventure.text.SelectorComponent
import net.kyori.adventure.text.TextComponent
import net.kyori.adventure.text.TranslatableComponent
import net.kyori.adventure.text.format.Style
import net.kyori.adventure.text.format.TextDecoration
import org.slf4j.LoggerFactory
import java.util.concurrent.ConcurrentHashMap

/** Registry of measurable fonts plus component-tree measurement helpers. */
internal class FontRegistry(
    private val defaultMetrics: FontMetrics,
    private val metricsByFont: Map<String, FontMetrics>,
) {
    private val warnedFonts: MutableSet<String> = ConcurrentHashMap.newKeySet()

    /**
     * Measures [component] as the client lays it out, using [fallbackFont] for runs without a font.
     * Fonts without metrics are measured with the default metrics and logged once.
     *
     * @throws IllegalArgumentException if the tree contains a non-text component (translatable,
     *   keybind, score, selector, NBT, or object), whose rendered text only the client knows.
     */
    fun measure(
        component: Component,
        fallbackFont: String,
    ): TextWidth = measureComponent(component, Style.style().font(Key.key(fallbackFont)).build())

    fun originFor(
        align: Align,
        slotX: Int,
        width: Int,
        textWidth: Int,
    ): Int =
        when (align) {
            Align.LEFT -> slotX
            Align.CENTER -> slotX + (width - textWidth) / 2
            Align.RIGHT -> slotX + width - textWidth
        }

    private fun measureComponent(
        component: Component,
        parentStyle: Style,
    ): TextWidth {
        require(component is TextComponent) { unmeasurableMessage(component) }
        val style = component.style().merge(parentStyle, Style.Merge.Strategy.IF_ABSENT_ON_TARGET)
        val bold = style.decoration(TextDecoration.BOLD) == TextDecoration.State.TRUE
        var width = metricsFor(style.font()).measureWidths(component.content(), bold)
        for (child in component.children()) {
            width = width.append(measureComponent(child, style))
        }
        return width
    }

    private fun metricsFor(font: Key?): FontMetrics {
        val id = font?.asString() ?: return defaultMetrics
        val metrics = metricsByFont[id]
        if (metrics != null) return metrics

        if (warnedFonts.add(id)) {
            LOGGER.warn("No metrics for font {}; using default metrics", id)
        }
        return defaultMetrics
    }

    internal companion object {
        private val LOGGER = LoggerFactory.getLogger(FontRegistry::class.java)

        fun fromManifest(manifest: WindowManifest): FontRegistry {
            val legacy =
                FontMetrics(
                    "legacy",
                    manifest.textAdvances,
                    manifest.textGlyphWidths,
                    FontMetrics.DEFAULT_BOLD_ADVANCE,
                )
            if (manifest.fontMetrics.isEmpty()) {
                return FontRegistry(legacy, mapOf("minecraft:default" to legacy))
            }
            val metrics =
                manifest.fontMetrics.mapValues { (name, entry) ->
                    FontMetrics.fromEntry(name, entry)
                }
            val defaultMetrics =
                metrics["minecraft:default"] ?: metrics.values.firstOrNull() ?: legacy
            return FontRegistry(defaultMetrics, metrics)
        }

        private fun unmeasurableMessage(component: Component): String {
            val kind =
                when (component) {
                    is TranslatableComponent -> "translatable"
                    is KeybindComponent -> "keybind"
                    is ScoreComponent -> "score"
                    is SelectorComponent -> "selector"
                    is NBTComponent<*> -> "NBT"
                    is ObjectComponent -> "object"
                    else -> component.javaClass.simpleName
                }
            return "Window cannot measure $kind components because the client resolves their text; " +
                "render slot content to literal text first (for translatables, use " +
                "GlobalTranslator.render(component, locale))"
        }
    }
}
