package com.chunkzero.window.internal

import com.chunkzero.window.diagnostics.RenderBounds
import com.chunkzero.window.manifest.SlotEntry
import com.chunkzero.window.manifest.SwitchEntry
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.TextComponent
import net.kyori.adventure.text.format.Style

private const val SPACE = ' '.code
private const val ELLIPSIS = "…"
private const val ASCII_ELLIPSIS = "..."
private val NO_WIDTH = TextWidth(0, 0)

/** One line of slot text styled over its line's font, whose top is at [y]. */
internal class SlotLine(
    val component: Component,
    val font: String,
    val y: Int,
)

/** Slot lines aligned in their slot and chained into one [component] that moves the cursor from [start] to [end]. */
internal class PlacedText(
    val component: Component,
    val content: String,
    val start: Int,
    val end: Int,
    val bounds: RenderBounds,
)

/**
 * [content] fitted to [slot] and styled over [style], whose font each line of a multi-line slot replaces with the
 * font of the line's position. The used lines are vertically centered in the slot.
 */
internal fun FontRegistry.slotLines(
    slot: SlotEntry,
    content: Component,
    style: Style,
): List<SlotLine> {
    val lines = slot.lines
    if (lines == null) {
        if (slot.overflow == null) return listOf(SlotLine(content.applyFallbackStyle(style), slot.font, slot.y))
        return wrapText(content, style, slot.width, 1).map { SlotLine(it.applyFallbackStyle(style), slot.font, slot.y) }
    }
    val wrapped = wrapText(content.withoutFont(Key.key(slot.font)), style, slot.width, lines.count)
    return wrapped.mapIndexed { j, line ->
        val step = lines.count - wrapped.size + 2 * j
        val font = lines.fonts[step]
        SlotLine(line.applyFallbackStyle(style.font(Key.key(font))), font, slot.y + step * lines.lineHeight / 2)
    }
}

/** This content with its fonts cleared, so each line of a multi-line slot draws in its line's [font]. */
private fun Component.withoutFont(font: Key): Component {
    val own = style().font()
    require(own == null || own == font) { "Multi-line slots draw text in their own font, not $own" }
    return style(style().font(null)).children(children().map { it.withoutFont(font) })
}

/**
 * Aligns [lines] in [slot], joining consecutive lines with the cursor offset [spacer] renders, or `null` when they
 * draw nothing.
 */
internal fun FontRegistry.placeLines(
    slot: SlotEntry,
    lines: List<SlotLine>,
    spacer: (offset: Int) -> Component?,
): PlacedText? {
    val first = lines.firstOrNull() ?: return null
    if (lines.size == 1) {
        val widths = measure(first.component, first.font)
        if (widths.advance == 0 && widths.visual == 0) return null
        val x = originFor(slot.align, slot.x, slot.width, widths.visual)
        val bounds = RenderBounds(x, first.y, widths.visual, TEXT_HEIGHT)
        return PlacedText(first.component, plainContent(first.component), x, x + widths.advance, bounds)
    }
    val parts = ArrayList<Component>()
    var start = 0
    var cursor = 0
    var left = Int.MAX_VALUE
    var right = Int.MIN_VALUE
    for ((j, line) in lines.withIndex()) {
        val widths = measure(line.component, line.font)
        val x = originFor(slot.align, slot.x, slot.width, widths.visual)
        if (j == 0) start = x else spacer(x - cursor)?.let(parts::add)
        parts += line.component
        cursor = x + widths.advance
        left = minOf(left, x)
        right = maxOf(right, x + widths.visual)
    }
    val bounds = RenderBounds(left, first.y, right - left, lines.last().y + TEXT_HEIGHT - first.y)
    val content = lines.joinToString("\n") { plainContent(it.component) }
    return PlacedText(Component.text().append(parts).build(), content, start, cursor, bounds)
}

/**
 * Wraps [content] at spaces onto at most [maxLines] lines whose visible width fits [width], measuring each run as
 * styled over [fallback]. A word wider than [width] ends in an ellipsis, as does the last line when text is left over.
 * Content that fits on one line is returned untouched; blank content has no lines. Kept runs keep their own styling.
 */
internal fun FontRegistry.wrapText(
    content: Component,
    fallback: Style,
    width: Int,
    maxLines: Int,
): List<Component> {
    val text = FittedText(this, content, fallback)
    if (text.glyphs.none { it.codePoint != SPACE }) return emptyList()
    if (text.width(0, text.glyphs.size).visual <= width) return listOf(content)

    val lines = ArrayList<Component>()
    var i = 0
    while (lines.size < maxLines) {
        while (i < text.glyphs.size && text.glyphs[i].codePoint == SPACE) i++
        if (i == text.glyphs.size) break
        val rest = text.trimEnd(i, text.glyphs.size)
        if (text.width(i, rest).visual <= width) {
            lines += text.line(i, rest, ellipsis = false)
            break
        }
        if (lines.size == maxLines - 1) {
            lines += text.ellipsized(i, text.glyphs.size, width, NO_WIDTH)
            break
        }
        val breakAt = text.lastBreak(i, width)
        if (breakAt == null) {
            val wordEnd =
                (i until text.glyphs.size).firstOrNull { text.glyphs[it].codePoint == SPACE } ?: text.glyphs.size
            lines += text.ellipsized(i, wordEnd, width, NO_WIDTH)
            i = wordEnd
        } else {
            lines += text.line(i, text.trimEnd(i, breakAt), ellipsis = false)
            i = breakAt
        }
    }
    return lines
}

/**
 * [value] shortened with an ellipsis so that it, followed by [suffix], fits [width] when styled over [fallback];
 * followed by [suffix], which is kept even when it alone is wider.
 */
internal fun FontRegistry.fitText(
    value: Component,
    suffix: Component,
    fallback: Style,
    width: Int,
): Component {
    val text = FittedText(this, value, fallback)
    val suffixWidth = measure(suffix.applyFallbackStyle(fallback), fallback.font()!!.asString())
    val fitted =
        if (text.width(0, text.glyphs.size).append(suffixWidth).visual <= width) {
            value
        } else {
            text.ellipsized(0, text.glyphs.size, width, suffixWidth)
        }
    return if (suffix == Component.empty()) fitted else Component.text().append(fitted, suffix).build()
}

/** The slot binding [name] measures with: its own slot, or the copy in the first case of a switch sharing it. */
internal fun Map<String, SlotEntry>.bindingSlot(
    name: String,
    switches: Map<String, SwitchEntry>,
    surface: String,
    ownerName: String,
): SlotEntry {
    this[name]?.let { slot ->
        require(slot.text == null) { "Slot '$name' is a static label" }
        if (slot.binding == null) return slot
    }
    val shared =
        switches.values.firstNotNullOfOrNull { switch ->
            switch.cases.firstNotNullOfOrNull { case ->
                case.slots.firstNotNullOfOrNull { key -> this[key]?.takeIf { it.binding == name } }
            }
        }
    return shared ?: throw IllegalArgumentException(
        "Unknown slot '$name' in $surface '$ownerName'; known slots: " +
            filterValues { it.text == null }.map { (key, slot) -> slot.binding ?: key }.distinct().sorted(),
    )
}

/** One literal run of flattened content: its text and the style it inherits, without the fallback. */
private class Run(
    val text: String,
    val style: Style,
)

/** One rendered code point of run [run], spanning `[start, end)` of its text including the codes before it. */
private class Glyph(
    val run: Int,
    val start: Int,
    val end: Int,
    val codePoint: Int,
    val bold: Boolean,
    val width: TextWidth,
)

/** Content flattened into styled runs and measured per rendered code point. */
private class FittedText(
    private val fonts: FontRegistry,
    content: Component,
    private val fallback: Style,
) {
    private val runs = ArrayList<Run>()
    val glyphs = ArrayList<Glyph>()

    init {
        flatten(content, Style.empty())
        for ((index, run) in runs.withIndex()) {
            val effective = run.style.merge(fallback, Style.Merge.Strategy.IF_ABSENT_ON_TARGET)
            val metrics = fonts.metricsFor(effective.font())
            var start = 0
            forEachRenderedCodePoint(run.text, effective.isBold()) { codePoint, bold, end ->
                glyphs +=
                    Glyph(
                        index,
                        start,
                        end,
                        codePoint,
                        bold,
                        metrics.measureWidths(Character.toString(codePoint), bold),
                    )
                start = end
            }
        }
    }

    fun width(
        from: Int,
        to: Int,
    ): TextWidth {
        var width = NO_WIDTH
        for (i in from until to) width = width.append(glyphs[i].width)
        return width
    }

    fun trimEnd(
        from: Int,
        to: Int,
    ): Int {
        var end = to
        while (end > from && glyphs[end - 1].codePoint == SPACE) end--
        return end
    }

    /** The last space after [from] whose preceding text fits [width], or `null` when the first word is too wide. */
    fun lastBreak(
        from: Int,
        width: Int,
    ): Int? {
        var found: Int? = null
        var line = NO_WIDTH
        for (i in from until glyphs.size) {
            if (i > from && glyphs[i].codePoint == SPACE && glyphs[i - 1].codePoint != SPACE) {
                if (line.visual > width) break
                found = i
            }
            line = line.append(glyphs[i].width)
        }
        return found
    }

    /**
     * The longest prefix of `[from, to)` that, followed by an ellipsis and [tail], fits [width]; empty when not even
     * the ellipsis fits.
     */
    fun ellipsized(
        from: Int,
        to: Int,
        width: Int,
        tail: TextWidth,
    ): Component {
        val prefixes = ArrayList<TextWidth>(to - from + 1)
        prefixes += NO_WIDTH
        for (i in from until to) prefixes += prefixes.last().append(glyphs[i].width)
        var end = to
        while (end >= from && from < glyphs.size) {
            val kept = trimEnd(from, end)
            if (prefixes[kept - from].append(ellipsisWidth(kept, from)).append(tail).visual <= width) {
                return line(from, kept, ellipsis = true)
            }
            end = kept - 1
        }
        return Component.empty()
    }

    fun line(
        from: Int,
        to: Int,
        ellipsis: Boolean,
    ): Component {
        val parts = ArrayList<Pair<Int, String>>()
        var i = from
        while (i < to) {
            val run = glyphs[i].run
            var last = i
            while (last + 1 < to && glyphs[last + 1].run == run) last++
            val text = runs[run].text
            parts += run to legacyCodes(text, glyphs[i].start) + text.substring(glyphs[i].start, glyphs[last].end)
            i = last + 1
        }
        if (ellipsis) {
            val run = ellipsisRun(to, from)
            val last = parts.lastOrNull()
            if (last?.first ==
                run
            ) {
                parts[parts.lastIndex] = run to last.second + ellipsis()
            } else {
                parts += run to ellipsis()
            }
        }
        val components = parts.map { (run, text) -> Component.text(text, runs[run].style) }
        return components.singleOrNull() ?: Component.text().append(components).build()
    }

    private fun ellipsisRun(
        end: Int,
        from: Int,
    ): Int = glyphs[if (end > from) end - 1 else from].run

    private fun ellipsisWidth(
        end: Int,
        from: Int,
    ): TextWidth {
        val style = runs[ellipsisRun(end, from)].style.merge(fallback, Style.Merge.Strategy.IF_ABSENT_ON_TARGET)
        val bold = if (end > from) glyphs[end - 1].bold else style.isBold()
        return fonts.metricsFor(style.font()).measureWidths(ellipsis(), bold)
    }

    private fun ellipsis(): String =
        if (fonts.metricsFor(fallback.font()).hasAdvance(ELLIPSIS.codePointAt(0))) ELLIPSIS else ASCII_ELLIPSIS

    private fun flatten(
        component: Component,
        parent: Style,
    ) {
        require(component is TextComponent) { FontRegistry.unmeasurableMessage(component) }
        val style = component.style().merge(parent, Style.Merge.Strategy.IF_ABSENT_ON_TARGET)
        if (component.content().isNotEmpty()) runs += Run(component.content(), style)
        for (child in component.children()) flatten(child, style)
    }
}

/** The legacy `§` formatting codes in [text] before [end], which a run continued from [end] must repeat. */
private fun legacyCodes(
    text: String,
    end: Int,
): String =
    buildString {
        var i = 0
        while (i + 1 < end) {
            if (text[i] == '§') {
                append(text, i, i + 2)
                i += 2
            } else {
                i++
            }
        }
    }
