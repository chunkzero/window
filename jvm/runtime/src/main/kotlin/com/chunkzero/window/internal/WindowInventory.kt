package com.chunkzero.window.internal

import com.chunkzero.window.SlotRef
import com.chunkzero.window.Tooltip
import com.chunkzero.window.WindowDefinition
import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.manifest.AnvilInputEntry
import com.chunkzero.window.manifest.SlotAreaEntry
import com.chunkzero.window.manifest.SlotRefEntry
import com.chunkzero.window.manifest.TooltipEntry
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.minimessage.MiniMessage

/**
 * Renders a window's inventory items: the hitbox items of active regions on slots no active item or collection
 * owns, active items and collection cells, and anvil input seeds. Rendered items collect until [drain] takes them.
 */
internal class WindowInventory<I : Any>(
    private val definition: WindowDefinition,
    private val bindings: WindowBindings<I>,
    private val reactivity: Reactivity,
    private val buildItem: (WindowItem) -> I,
) {
    private val entry = definition.entry
    private val switches = bindings.switches

    /** The regions, items, and collections whose case path was active at the last [claimCases]. */
    private val claimed = LinkedHashSet<String>()
    private val claimedItems = LinkedHashSet<String>()
    private val claimedCollections = LinkedHashSet<String>()

    /** Parsed manifest tooltips; components are immutable, so they are shared across renders. */
    private val tooltips = HashMap<TooltipEntry, Tooltip>()

    private var inputRevisions = 0
    private val items = LinkedHashMap<SlotRef, I?>()
    private val staged = LinkedHashMap<Int, I?>()

    /** Returns the items rendered since the last drain, and forgets them. */
    fun drain(): SlotWrites<I> {
        val writes = SlotWrites(LinkedHashMap(items), LinkedHashMap(staged))
        items.clear()
        staged.clear()
        return writes
    }

    /** Renders every manifest-owned inventory slot. */
    fun seed() {
        claimCases()
        for (input in entry.inputs.values) applyInput(input, input.initial)
    }

    /**
     * Re-derives the slots of regions, items, and collections whose case changed: an active item or collection owns
     * its slots' contents, otherwise the active region's hitbox fills them, otherwise they are empty. Items and
     * collections that became active then render over them.
     */
    fun claimCases() {
        val active = entry.regions.keys.filterTo(LinkedHashSet(), switches::regionActive)
        val activeItems = entry.items.keys.filterTo(LinkedHashSet(), switches::itemActive)
        val activeCollections = entry.collections.keys.filterTo(LinkedHashSet(), switches::collectionActive)
        val touched = LinkedHashSet<SlotRefEntry>()
        for (name in claimed union active) {
            if ((name in claimed) != (name in active)) touched += entry.regions.getValue(name).filledSlots
        }
        for (name in claimedItems union activeItems) {
            if ((name in claimedItems) != (name in activeItems)) touched += entry.items.getValue(name).slots
        }
        for (name in claimedCollections union activeCollections) {
            if ((name in claimedCollections) != (name in activeCollections)) {
                touched += entry.collections.getValue(name).slots
            }
        }
        val owned =
            activeItems.flatMapTo(HashSet()) { entry.items.getValue(it).slots } +
                activeCollections.flatMapTo(HashSet()) { entry.collections.getValue(it).slots }
        for (slot in touched) {
            if (slot in owned) continue
            items[slot.toApi()] =
                active.firstOrNull { slot in entry.regions.getValue(it).filledSlots }?.let(::regionItem)
        }
        val newItems = activeItems - claimedItems
        val newCollections = activeCollections - claimedCollections
        claimed.clear()
        claimed.addAll(active)
        claimedItems.clear()
        claimedItems.addAll(activeItems)
        claimedCollections.clear()
        claimedCollections.addAll(activeCollections)
        for (name in newItems) writeItem(name)
        for (name in newCollections) {
            for (index in entry.collections
                .getValue(name)
                .slots.indices) {
                writeCollectionCell(name, index)
            }
        }
    }

    /** Renders item [name] into its slots while its case is active. */
    fun writeItem(name: String) {
        if (!switches.itemActive(name)) return
        val render = bindings.items.getValue(name)
        val stack = reactivity.withRendering(RenderKey.Item(name)) { render() }
        setItem(name, stack)
    }

    fun setItem(
        name: String,
        stack: I?,
    ) {
        val item = definition.requireEntry(entry.items, name, "item", known = "items")
        if (switches.itemActive(name)) writeSlots(item.slots, stack)
    }

    fun writeCollectionCell(
        name: String,
        index: Int,
    ) {
        if (!switches.collectionActive(name)) return
        val slot =
            entry.collections
                .getValue(name)
                .slots
                .getOrNull(index) ?: return
        val render = bindings.collectionItems.getValue(name)
        items[slot.toApi()] = reactivity.withRendering(RenderKey.CollectionCell(name, index)) { render(index) }
    }

    /**
     * Renders the input seed renamed to [value]. Each render carries a new revision, since the client
     * only resets its edit box to the seed's name when the item changes.
     */
    fun applyInput(
        input: AnvilInputEntry,
        value: String,
    ) {
        items[input.slot.toApi()] = buildItem(inputSeed(input, value, ++inputRevisions))
    }

    /** Stages the input seed renamed to [value], so the next reopen restores [value]. */
    fun stageInput(
        input: AnvilInputEntry,
        value: String,
    ) {
        check(input.slot.area == SlotAreaEntry.CONTAINER) { "Anvil inputs must use container slots" }
        staged[input.slot.index] = buildItem(inputSeed(input, value))
    }

    private fun inputSeed(
        input: AnvilInputEntry,
        value: String,
        revision: Int = 0,
    ): WindowItem.AnvilSeed =
        WindowItem.AnvilSeed(input.itemModel?.let(Key::key) ?: definition.hitboxModel, value, revision)

    /** Region [name]'s hitbox item. */
    private fun regionItem(name: String): I? {
        val hitbox = entry.regions.getValue(name).hitbox ?: return null
        val model = hitbox.itemModel?.let(Key::key) ?: definition.hitboxModel
        return buildItem(WindowItem.Hitbox(model, hitbox.tooltip?.let(::tooltip)))
    }

    private fun writeSlots(
        slots: List<SlotRefEntry>,
        item: I?,
    ) {
        for (slot in slots) items[slot.toApi()] = item
    }

    private fun tooltip(source: TooltipEntry): Tooltip =
        tooltips.getOrPut(source) {
            Tooltip(
                parseTooltipLine(source.title, NamedTextColor.WHITE),
                source.lines.map { parseTooltipLine(it, NamedTextColor.GRAY) },
            )
        }

    private fun parseTooltipLine(
        template: String,
        defaultColor: TextColor,
    ): Component = Component.empty().withDefaults(defaultColor).append(MINI_MESSAGE.deserialize(template))

    private companion object {
        val MINI_MESSAGE: MiniMessage = MiniMessage.miniMessage()
    }
}
