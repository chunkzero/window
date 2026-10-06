package com.chunkzero.window

/** Which backing inventory a window control slot belongs to. */
public enum class SlotArea {
    /** Slot in the opened container inventory. */
    CONTAINER,

    /** Slot in the viewing player's own inventory. */
    PLAYER,
}

/** A typed inventory slot owned by a window control. */
public data class SlotRef(
    val area: SlotArea,
    val index: Int,
)

/**
 * A click on a window button.
 *
 * @property slot the typed backing slot that was clicked.
 * @property shift whether the shift modifier was held.
 * @property right whether the click was a right click (otherwise treated as left).
 */
public data class Click(
    val slot: SlotRef,
    val shift: Boolean,
    val right: Boolean,
) {
    /** Convenience for old-style handlers that only care about the numeric index. */
    public val inventorySlot: Int
        get() = slot.index

    /** The clicked backing inventory area. */
    public val area: SlotArea
        get() = slot.area
}

/**
 * A click on a repeated collection cell.
 *
 * @property index the collection cell index, matching the authored slot order.
 */
public data class IndexedClick(
    val slot: SlotRef,
    val index: Int,
    val shift: Boolean,
    val right: Boolean,
)
