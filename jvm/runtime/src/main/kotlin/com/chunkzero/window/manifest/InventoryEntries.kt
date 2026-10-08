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

/**
 * An inventory region: the slots it claims and fills with its hitbox item, and the action its clicks
 * name. A region listed by a switch case is claimed only while that case is active.
 */
@Serializable
public data class RegionEntry(
    /** Top-left x of the region rect in GUI pixels. */
    val x: Int,
    /** Top-left y of the region rect in GUI pixels. */
    val y: Int,
    /** Region rect width in GUI pixels. */
    val width: Int,
    /** Region rect height in GUI pixels. */
    val height: Int,
    /** Backing inventory slots whose clicks route to this region. */
    val slots: List<SlotRefEntry>,
    /**
     * Slots this region fills with its hitbox item.
     *
     * `null` means every slot in [slots]. A repeater cell that hands one of its slots to an item
     * control emits a strict subset here: clicks still route to the whole cell, but the yielded
     * slot carries the item control's real stack (and therefore its native hover tooltip).
     */
    @SerialName("fill_slots") val fillSlots: List<SlotRefEntry>? = null,
    /**
     * The action id clicks name: the handler bound to it runs, else [defaultAction]. Ids in the
     * `window:` namespace are runtime actions. `null` for hover-only and claim-only regions, which
     * ignore clicks.
     */
    val action: String? = null,
    /** The runtime action, such as `window:close`, run when no handler is bound to [action]. */
    @SerialName("default_action") val defaultAction: String? = null,
    /** The item filling [filledSlots]; `null` leaves them empty. */
    val hitbox: HitboxEntry? = null,
    /** The authored element this region comes from, such as ``button `buy` ``, for diagnostics. */
    val source: String? = null,
) {
    /** The slots this region paints with its hitbox item: [fillSlots] when present, else [slots]. */
    public val filledSlots: List<SlotRefEntry>
        get() = fillSlots ?: slots
}

/** The item a region fills its slots with. */
@Serializable
public data class HitboxEntry(
    /** Item model id; `null` uses the pack's invisible `{namespace}:gui/hitbox` model. */
    @SerialName("item_model") val itemModel: String? = null,
    /** Hover tooltip. */
    val tooltip: TooltipEntry? = null,
)

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
    /** The action of each cell's region, in index order. */
    val actions: List<String> = emptyList(),
)

/** Manifest tooltip text as MiniMessage templates; the runtime parses it into a `Tooltip`. */
@Serializable
public data class TooltipEntry(
    /** Tooltip title/name template. */
    val title: String,
    /** Additional lore line templates. */
    val lines: List<String> = emptyList(),
)
