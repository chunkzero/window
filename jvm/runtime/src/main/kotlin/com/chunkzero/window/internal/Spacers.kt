package com.chunkzero.window.internal

/**
 * Greedy decomposition of pixel offsets into spacer-font codepoints.
 *
 * The manifest carries a `space` provider table mapping codepoint -> advance (values ±1, ±2, ±4 …
 * ±1024). An arbitrary horizontal offset is composed greedily, largest magnitude first; offsets
 * beyond the largest table entry repeat that entry. Codepoints are never hardcoded — they come
 * entirely from the manifest's spacer map.
 */
internal class Spacers(
    spacers: Map<Int, Int>,
) {
    private class Entry(
        val codepoint: Int,
        val advance: Int,
    )

    /** Positive-advance entries, descending magnitude. */
    private val positive: List<Entry> =
        spacers
            .map { (cp, adv) -> Entry(cp, adv) }
            .filter { it.advance > 0 }
            .sortedByDescending { it.advance }

    /** Negative-advance entries, ascending (i.e. most negative first). */
    private val negative: List<Entry> =
        spacers
            .map { (cp, adv) -> Entry(cp, adv) }
            .filter { it.advance < 0 }
            .sortedBy { it.advance }

    /**
     * Returns the spacer string realising exactly [offset] pixels of horizontal movement.
     *
     * An empty string is returned for an offset of `0`. The decomposition is greedy from the
     * largest magnitude down; each codepoint is repeated as many times as its advance fits into the
     * remaining offset. With the canonical powers-of-two table the largest entry absorbs any
     * out-of-range magnitude and the ±1 entry guarantees the remainder closes to zero.
     */
    fun compose(offset: Int): String {
        if (offset == 0) return ""
        val positiveOffset = offset > 0
        val table = if (positiveOffset) positive else negative
        require(table.isNotEmpty()) {
            "Spacer table has no entries of sign matching offset $offset"
        }

        val sb = StringBuilder()
        var remaining = offset
        for (entry in table) {
            while (if (positiveOffset) remaining >= entry.advance else remaining <= entry.advance) {
                sb.appendCodePoint(entry.codepoint)
                remaining -= entry.advance
            }
            if (remaining == 0) break
        }
        check(remaining == 0) {
            "Spacer table cannot represent offset $offset exactly (remainder $remaining); a ±1 " +
                "entry is required"
        }
        return sb.toString()
    }
}
