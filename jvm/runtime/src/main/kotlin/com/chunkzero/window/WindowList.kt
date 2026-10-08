package com.chunkzero.window

import com.chunkzero.window.internal.LazyMemo
import com.chunkzero.window.internal.LazyState
import com.chunkzero.window.internal.Reactivity

/**
 * A reactive list shown through a fixed number of cells, created with [WindowView.list].
 *
 * The offset of the first visible item and the selected item's key are view state: reading them
 * while rendering re-renders when they change. The offset is clamped to the current list when read,
 * and the selection is kept by key across paging and reordering.
 *
 * Cell arguments must be in `0 until cells`, or an [IllegalArgumentException] is thrown. The step
 * must be in `1..cells` so every offset it reaches shows the items it moves past.
 *
 * @param T the item type.
 * @param K the key identifying an item across list changes.
 */
public class WindowList<T, K> internal constructor(
    /** The number of visible cells. */
    public val cells: Int,
    private val key: (T) -> K,
    /** How far [previous] and [next] move: [cells] to page, or a row width of at most [cells] to scroll. */
    public val step: Int,
    private val mode: Select,
    source: () -> List<T>,
    engine: () -> Reactivity?,
) {
    /** What is selected when no explicitly selected item is in the list. */
    public enum class Select {
        /** The first item of the list, if any. */
        FIRST,

        /** Nothing. */
        NONE,
    }

    /** A selected key; wrapped so a `null` key stays distinct from no selection. */
    private data class Selected<K>(
        val key: K,
    )

    private var rawOffset by LazyState(0, engine)
    private var selectedKey: Selected<K>? by LazyState(null, engine)
    private val items = LazyMemo(engine, source)
    private val selectedIndex =
        LazyMemo(engine) {
            val list = items.get()
            val selected = selectedKey
            val index = selected?.let { s -> list.indexOfFirst { key(it) == s.key } } ?: -1
            when {
                index >= 0 -> index
                mode == Select.FIRST && list.isNotEmpty() -> 0
                else -> null
            }
        }

    init {
        require(cells > 0) { "cells must be positive" }
        require(step in 1..cells) { "step must be in 1..$cells" }
    }

    /** The number of items in the list. */
    public val size: Int
        get() = items.get().size

    /** Absolute indices of the items currently shown, in cell order. */
    public val visible: IntRange
        get() {
            val offset = offset()
            return offset until minOf(offset + cells, size)
        }

    /** The 1-based page of the current offset; a partial final step counts as a page. */
    public val page: Int
        get() = pages(offset())

    /** The number of pages, at least 1. */
    public val pageCount: Int
        get() = pages(lastOffset(size))

    /** The selected item, or `null` for none. */
    public val selected: T?
        get() = selectedIndex.get()?.let { items.get()[it] }

    /** The cell showing the selected item, or `null` when it is not visible or nothing is selected. */
    public val selectedCell: Int?
        get() = selectedIndex.get()?.minus(offset())?.takeIf { it in 0 until cells }

    /** The item shown in [cell], or `null` when the cell is past the end of the list. */
    public fun at(cell: Int): T? {
        requireCell(cell)
        return items.get().getOrNull(offset() + cell)
    }

    /** True when [cell] shows the selected item. */
    public fun isSelected(cell: Int): Boolean {
        requireCell(cell)
        return selectedCell == cell
    }

    /** Selects the item shown in [cell] and returns it; an empty cell returns `null` and keeps the selection. */
    public fun selectAt(cell: Int): T? = at(cell)?.also { selectedKey = Selected(key(it)) }

    /** Selects [item] by key and moves the offset as little as needed to show it, if it is in the list. */
    public fun select(item: T) {
        val itemKey = key(item)
        selectedKey = Selected(itemKey)
        val index = items.get().indexOfFirst { key(it) == itemKey }
        if (index < 0) return
        val offset = offset()
        when {
            index < offset -> rawOffset = clamp(index / step * step)
            index >= offset + cells -> rawOffset = clamp((index - cells + step) / step * step)
        }
    }

    /** Clears the explicit selection; under [Select.FIRST] the first item is selected again. */
    public fun clearSelection() {
        selectedKey = null
    }

    /** True when [previous] would move. */
    public fun canPrevious(): Boolean = offset() > 0

    /** True when [next] would move. */
    public fun canNext(): Boolean = offset() < lastOffset(size)

    /** Moves back by [step]. */
    public fun previous() {
        rawOffset = clamp(offset() - step)
    }

    /** Moves forward by [step]. */
    public fun next() {
        rawOffset = clamp(offset() + step)
    }

    /** Shows the start of the list. */
    public fun first() {
        rawOffset = 0
    }

    /** Shows the end of the list. */
    public fun last() {
        rawOffset = lastOffset(size)
    }

    /** Shows the start of the list and clears the selection, as when created. */
    public fun reset() {
        rawOffset = 0
        selectedKey = null
    }

    /**
     * Adapts this list to a [WindowCollection] for [WindowScope.collection]: each cell shows
     * [render] of its item, and clicks on non-empty cells call [onClick], which selects the item by
     * default.
     */
    public fun <I> items(
        render: (T) -> I?,
        onClick: (T, Click) -> Unit = { item, _ -> select(item) },
    ): WindowCollection<I> =
        object : WindowCollection<I> {
            override fun item(cell: Int): I? = at(cell)?.let(render)

            override fun click(
                cell: Int,
                click: Click,
            ) {
                at(cell)?.let { onClick(it, click) }
            }

            override fun selected(): Int? = selectedCell
        }

    private fun pages(offset: Int): Int = (offset + step - 1) / step + 1

    private fun offset(): Int = clamp(rawOffset)

    private fun clamp(offset: Int): Int = offset.coerceIn(0, lastOffset(size))

    private fun lastOffset(total: Int): Int =
        when {
            total <= cells -> 0
            step >= cells -> (total - 1) / step * step
            else -> total - cells
        }

    private fun requireCell(cell: Int) {
        require(cell in 0 until cells) { "cell index $cell outside 0 until $cells" }
    }
}
