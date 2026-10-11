package com.chunkzero.window

import com.chunkzero.window.internal.Spacers
import com.chunkzero.window.internal.columns
import com.chunkzero.window.internal.compose
import com.chunkzero.window.manifest.BarEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe

private const val WIDTH = 11
private const val X = 5

/** The columns each clip glyph draws, from the bar's left edge. */
private val clips = HashMap<Int, IntRange>()

/** A bar of [WIDTH] columns at [X] with a distinct glyph per clip, each advancing one pixel past its columns. */
private val bar: BarEntry =
    run {
        var glyph = 'a'
        val levels =
            (0..3).map { level ->
                val size = 1 shl level
                generateSequence(0) { it + 2 * size }
                    .takeWhile { it + size <= WIDTH }
                    .map { start -> glyph++.also { clips[it.code] = start until start + size } }
                    .joinToString("")
            }
        BarEntry(
            name = "hp",
            x = X,
            width = WIDTH,
            levels = levels,
            advances = clips.entries.associate { (glyph, columns) -> glyph.toChar().toString() to columns.count() + 1 },
        )
    }

class BarsTest :
    StringSpec({
        "every column count draws exactly its leftmost columns and returns the cursor" {
            val table = TestManifests.spacerTable()
            for (columns in 0..WIDTH) {
                val drawn = sortedSetOf<Int>()
                var cursor = 0
                for (codepoint in bar.compose(columns, Spacers(table)).codePoints().toArray()) {
                    val clip = clips[codepoint]
                    if (clip == null) {
                        cursor += table.getValue(codepoint)
                    } else {
                        (cursor - X).let { it shouldBe clip.first }
                        drawn += clip
                        cursor += clip.count() + 1
                    }
                }
                drawn.toList() shouldBe (0 until columns).toList()
                cursor shouldBe 0
            }
        }

        "a fraction fills the nearest whole column and is clamped to the bar" {
            bar.columns(0f) shouldBe 0
            bar.columns(0.04f) shouldBe 0
            bar.columns(0.05f) shouldBe 1
            bar.columns(0.5f) shouldBe 6
            bar.columns(1f) shouldBe WIDTH
            bar.columns(7f) shouldBe WIDTH
            bar.columns(-1f) shouldBe 0
            bar.columns(Float.NaN) shouldBe 0
        }
    })
