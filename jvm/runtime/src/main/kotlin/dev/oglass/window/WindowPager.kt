package dev.oglass.window

/**
 * Small helper for mapping a list onto a fixed set of authored collection cells.
 *
 * [cellCount] is the number of visible cells. [step] is how far next/previous move; use `cellCount`
 * for page buttons or a row width for vertical up/down scrolling.
 */
public class WindowPager(
    public val cellCount: Int,
    public val step: Int = cellCount,
) {
    init {
        require(cellCount > 0) { "cellCount must be positive" }
        require(step > 0) { "step must be positive" }
    }

    /** Returns [offset] clamped to a valid start index for [total] items. */
    public fun clamp(
        offset: Int,
        total: Int,
    ): Int = offset.coerceIn(0, lastOffset(total))

    /** True when [offset] can move backward. */
    public fun canPrevious(offset: Int): Boolean = offset > 0

    /** True when [offset] can move forward for [total] items. */
    public fun canNext(
        offset: Int,
        total: Int,
    ): Boolean = clamp(offset, total) < lastOffset(total)

    /** Previous offset for [total] items. */
    public fun previous(
        offset: Int,
        total: Int,
    ): Int = clamp(offset - step, total)

    /** Next offset for [total] items. */
    public fun next(
        offset: Int,
        total: Int,
    ): Int = clamp(offset + step, total)

    /** Absolute item index for a visible [cell], or `null` if it is beyond [total]. */
    public fun itemIndex(
        offset: Int,
        cell: Int,
        total: Int,
    ): Int? {
        require(cell in 0 until cellCount) { "cell index $cell outside 0 until $cellCount" }
        val index = clamp(offset, total) + cell
        return index.takeIf { it < total }
    }

    /** The 1-based page number for [offset]. */
    public fun page(
        offset: Int,
        total: Int,
    ): Int {
        if (total <= 0) return 1
        return clamp(offset, total) / step + 1
    }

    /** Total page count for [total] items. */
    public fun pageCount(total: Int): Int {
        val last = lastOffset(total)
        return if (last == 0) 1 else (last / step) + 1
    }

    private fun lastOffset(total: Int): Int {
        if (total <= cellCount) return 0
        return if (step >= cellCount) {
            ((total - 1) / step) * step
        } else {
            total - cellCount
        }
    }
}
