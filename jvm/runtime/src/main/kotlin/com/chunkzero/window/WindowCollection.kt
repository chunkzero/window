package com.chunkzero.window

/**
 * The contents of a repeated item collection, bound with [WindowScope.collection].
 *
 * Cells are indexed in authored slot order. [item] and [selected] are called while rendering, so
 * reactive state they read re-renders the collection when it changes.
 *
 * @param I the host's native item type.
 */
public interface WindowCollection<I> {
    /** The item to show in [cell], or `null` for an empty cell. */
    public fun item(cell: Int): I?

    /** Handles [click] on [cell]. Only called for collections that accept clicks. */
    public fun click(
        cell: Int,
        click: Click,
    )

    /**
     * The cell to mark with the collection's selected sprite, or `null` for none. Only read for
     * collections with a selected sprite.
     */
    public fun selected(): Int?
}
