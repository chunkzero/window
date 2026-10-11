package com.chunkzero.window.internal

import com.chunkzero.window.manifest.ArtEntry
import com.chunkzero.window.manifest.BarEntry
import kotlin.math.roundToInt

/**
 * The bars of one window or HUD: binds each bar to a lambda returning how much of it is filled, and tracks
 * the columns each last drew.
 */
internal class Bars(
    /** The surface name used in messages, `window` or `hud`. */
    private val surface: String,
    private val ownerName: String,
    art: Map<String, ArtEntry>,
    /** Captures state reads per binding; `null` for surfaces that render without reactivity. */
    private val reactivity: Reactivity?,
) {
    private val bars: Map<String, BarEntry> = art.values.mapNotNull { it.bar }.associateBy { it.name }

    /** Bar names. */
    val names: Set<String> = bars.keys

    private val renders = HashMap<String, () -> Float>()
    private val columns = HashMap<String, Int>()

    fun bind(
        name: String,
        render: () -> Float,
    ) {
        require(name in bars) { "Unknown bar '$name' in $surface '$ownerName'; known bars: ${names.sorted()}" }
        require(renders.put(name, render) == null) { "Bar '$name' bound more than once" }
    }

    fun validate() {
        val unbound = names - renders.keys
        check(unbound.isEmpty()) { "Unbound bars in $surface '$ownerName': ${unbound.sorted()}" }
    }

    /**
     * Evaluates bar [name]'s binding under dependency capture and takes the columns its fraction fills;
     * returns whether they changed.
     */
    fun update(name: String): Boolean {
        val render = renders.getValue(name)
        val fraction = if (reactivity == null) render() else reactivity.withRendering(RenderKey.Bar(name), render)
        val filled = bars.getValue(name).columns(fraction)
        return columns.put(name, filled) != filled
    }

    /** The columns [bar] last drew; `null` without a bar. */
    fun columns(bar: BarEntry?): Int? = bar?.let { columns.getValue(it.name) }
}

/** The columns this bar draws at [fraction]: the nearest whole column, with the fraction clamped to 0 through 1. */
internal fun BarEntry.columns(fraction: Float): Int =
    if (fraction.isNaN()) 0 else (fraction.coerceIn(0f, 1f) * width).roundToInt()

/** The net-zero text that draws the leftmost [columns] columns of this bar's fill. */
internal fun BarEntry.compose(
    columns: Int,
    spacers: Spacers,
): String {
    val text = StringBuilder()
    var cursor = 0
    var start = 0
    for (level in levels.indices.reversed()) {
        val size = 1 shl level
        if (columns and size == 0) continue
        val glyph = levels[level][start shr (level + 1)]
        if (glyph != ' ') {
            val at = x + start
            text.append(spacers.compose(at - cursor)).append(glyph)
            cursor = at + advances.getValue(glyph.toString())
        }
        start += size
    }
    return text.append(spacers.compose(-cursor)).toString()
}
