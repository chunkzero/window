package com.chunkzero.window.manifest

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** Which backing inventory a control slot belongs to. */
@Serializable
public enum class SlotAreaEntry {
    @SerialName("container")
    CONTAINER,

    @SerialName("player")
    PLAYER,
}

/** A typed inventory slot owned by a window control. */
@Serializable
public data class SlotRefEntry(
    /** Backing inventory area. */
    val area: SlotAreaEntry,
    /** Slot index in that area. */
    val index: Int,
)

/** A clickable region mapped to typed inventory slots. */
@Serializable
public data class ButtonEntry(
    /** Top-left x of the button rect in GUI pixels. */
    val x: Int,
    /** Top-left y of the button rect in GUI pixels. */
    val y: Int,
    /** Button rect width in GUI pixels. */
    val width: Int,
    /** Button rect height in GUI pixels. */
    val height: Int,
    /** Backing inventory slots whose clicks route to this region. */
    val slots: List<SlotRefEntry>,
    /**
     * Slots this region fills with its own hitbox/state item.
     *
     * `null` means every slot in [slots]. A repeater cell that hands one of its slots to an item
     * control emits a strict subset here: clicks still route to the whole cell, but the yielded
     * slot carries the item control's real stack (and therefore its native hover tooltip).
     */
    @SerialName("fill_slots") val fillSlots: List<SlotRefEntry>? = null,
    /** Built-in behavior when no handler is bound, or `null` if a handler is required. */
    val default: ButtonDefault? = null,
    /** Whether this region should accept a generated/runtime click handler. */
    val action: Boolean = true,
    /** Default tooltip shown for this button or hotspot. */
    val tooltip: ButtonTooltip? = null,
    /** Named item states for dynamic visual/tooltip toggles. */
    val states: Map<String, ButtonState> = emptyMap(),
    /** Generated sprite font used by state sprites, when present. */
    @SerialName("sprite_font") val spriteFont: String? = null,
) {
    /** The slots this region paints with its own item: [fillSlots] when present, else [slots]. */
    public val filledSlots: List<SlotRefEntry>
        get() = fillSlots ?: slots
}

/** A dynamic inventory item region. */
@Serializable
public data class ItemEntry(
    /** Backing inventory slots this item control populates. */
    val slots: List<SlotRefEntry>,
)

/** A repeated dynamic inventory item region. */
@Serializable
public data class CollectionEntry(
    /** Backing inventory slots, one visible cell per slot in order. */
    val slots: List<SlotRefEntry>,
    /** Whether this collection accepts a click handler. */
    val action: Boolean = true,
    /**
     * The selected-cell sprite placed over each cell's 18x18 box, indexed like [slots]; empty when
     * the collection has no selected sprite.
     */
    val selection: List<SpriteSlotEntry> = emptyList(),
)

/** A native anvil rename-field binding. */
@Serializable
public data class AnvilInputEntry(
    /** Input slot seeded to make the vanilla field editable. */
    val slot: SlotRefEntry,
    /** Initial field contents. */
    val initial: String = "",
    /** Optional item model used for the seed item. */
    @SerialName("item_model") val itemModel: String? = null,
)

/** A non-binding slot claim/fill region. */
@Serializable
public data class SlotRectEntry(
    /** Backing inventory slots claimed and cleared by this primitive. */
    val slots: List<SlotRefEntry>,
)

/** Group metadata for controls flattened from a repeater. */
@Serializable
public data class RepeatGroupEntry(
    /** Number of repeated cells. */
    val count: Int,
    /** Dynamic text slots by repeated child field, each vector in index order. */
    val slots: Map<String, List<String>> = emptyMap(),
    /** Runtime sprite slots by repeated child field, each vector in index order. */
    @SerialName("sprite_slots") val spriteSlots: Map<String, List<String>> = emptyMap(),
    /** Dynamic inventory item controls by repeated child field, each vector in index order. */
    val items: Map<String, List<String>> = emptyMap(),
    /** Root cell buttons, in index order. */
    val buttons: List<String> = emptyList(),
)

/** Plain manifest tooltip text. Runtime APIs can provide rich Adventure components. */
@Serializable
public data class ButtonTooltip(
    /** Tooltip title/name. */
    val title: String,
    /** Additional lore lines. */
    val lines: List<String> = emptyList(),
)

/** One named item state for a button. */
@Serializable
public data class ButtonState(
    /** Optional item model id, e.g. `"example:gui/shop_button_active"`. */
    @SerialName("item_model") val itemModel: String? = null,
    /** Optional Window sprite rendered over the button rect for this state. */
    val sprite: String? = null,
    /** Optional tooltip override for this state. */
    val tooltip: ButtonTooltip? = null,
)

/** Built-in button behavior used when a manifest button has no user handler. */
@Serializable
public enum class ButtonDefault {
    @SerialName("close")
    CLOSE,
}
