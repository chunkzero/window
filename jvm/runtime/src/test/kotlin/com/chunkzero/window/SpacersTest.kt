package com.chunkzero.window

import com.chunkzero.window.internal.Spacers
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe

class SpacersTest :
    StringSpec({
        val table = TestManifests.spacerTable()
        val spacers = Spacers(table)

        // Decode a spacer string back into its summed advance using the table.
        fun decode(s: String): Int {
            var total = 0
            var i = 0
            while (i < s.length) {
                val cp = s.codePointAt(i)
                total += table.getValue(cp)
                i += Character.charCount(cp)
            }
            return total
        }

        fun cp(advance: Int) = TestManifests.spacerCodepoint(advance)

        "offset of 0 produces empty string" { spacers.compose(0) shouldBe "" }

        "exact decomposition of +13" {
            // 13 = 8 + 4 + 1
            val expected =
                StringBuilder()
                    .appendCodePoint(cp(8))
                    .appendCodePoint(cp(4))
                    .appendCodePoint(cp(1))
                    .toString()
            spacers.compose(13) shouldBe expected
        }

        "exact decomposition of -9" {
            // -9 = -8 + -1
            val expected =
                StringBuilder().appendCodePoint(cp(-8)).appendCodePoint(cp(-1)).toString()
            spacers.compose(-9) shouldBe expected
        }

        "offset beyond largest entry repeats it" {
            // 2050 = 1024 + 1024 + 2
            val expected =
                StringBuilder()
                    .appendCodePoint(cp(1024))
                    .appendCodePoint(cp(1024))
                    .appendCodePoint(cp(2))
                    .toString()
            spacers.compose(2050) shouldBe expected
        }

        "arbitrary sweep sums back to the requested offset" {
            for (offset in -5000..5000) {
                decode(spacers.compose(offset)) shouldBe offset
            }
        }
    })
