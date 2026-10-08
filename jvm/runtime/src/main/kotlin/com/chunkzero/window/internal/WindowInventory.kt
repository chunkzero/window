package com.chunkzero.window.internal

import com.chunkzero.window.ButtonTooltip
import com.chunkzero.window.SlotRef
import com.chunkzero.window.WindowDefinition
import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.manifest.AnvilInputEntry
import com.chunkzero.window.manifest.SlotAreaEntry
import com.chunkzero.window.manifest.TooltipEntry
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.minimessage.MiniMessage

/**
 * Renders a window's inventory items: the hitbox items of active regions, item regions, collection
 * cells, and anvil input seeds. Rendered items collect until [drain] takes them.
 */
internal class WindowInventory<I : Any>(
    private val definition: WindowDefinition,
    private val bindings: WindowBindings<I>,
    private val reactivity: Reactivity,
    private val buildItem: (WindowItem) -> I,
) {
    private val entry = definition.entry
    private val switches = bindings.switches

    /** The button or hotspot each region belongs to. */
    private val regionControls: Map<String, String> =
        definition.controls.flatMap { (control, regions) -> regions.map { it to control } }.toMap()

    /** The regions whose case path was active at the last [claimRegions]. */
    private val claimed = LinkedHashSet<String>()

    /** Parsed manifest tooltips; components are immutable, so they are shared across renders. */
    private val tooltips = HashMap<TooltipEntry, ButtonTooltip>()

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
        claimRegions()
        for (name in entry.items.keys) writeItem(name)
        for ((name, collection) in entry.collections) {
            for (index in collection.slots.indices) writeCollectionCell(name, index)
        }
        for (input in entry.inputs.values) applyInput(input, input.initial)
    }

    /**
     * Claims the regions of active cases: empties the fill slots of regions whose case became inactive,
     * then fills those of newly active regions with their items.
     */
    fun claimRegions() {
        val active = entry.regions.keys.filterTo(LinkedHashSet(), switches::regionActive)
        for (name in claimed) if (name !in active) fill(name, null)
        for (name in active) if (name !in claimed) fill(name, regionItem(name))
        claimed.clear()
        claimed.addAll(active)
    }

    fun renderButtonState(name: String): String {
        val render = bindings.buttonStates.getValue(name)
        return reactivity.withRendering(RenderKey.ButtonState(name)) { render() }
    }

    /** Re-renders the item bound to button or hotspot [name] into its claimed regions. */
    fun writeButtonItem(name: String) {
        val item = renderButtonItem(name)
        for (region in claimedRegions(name)) fill(region, item)
    }

    /** Places [item] in the claimed regions of button or hotspot [name]; regions claimed later show their own item. */
    fun setButtonItem(
        name: String,
        item: I?,
    ) {
        definition.requireEntry(definition.controls, name, "button or hotspot")
        for (region in claimedRegions(name)) fill(region, item)
    }

    fun writeItem(name: String) {
        val render = bindings.items.getValue(name)
        val stack = reactivity.withRendering(RenderKey.Item(name)) { render() }
        setItem(name, stack)
    }

    fun setItem(
        name: String,
        stack: I?,
    ) {
        val item = definition.requireEntry(entry.items, name, "item", known = "items")
        for (slot in item.slots) items[slot.toApi()] = stack
    }

    fun writeCollectionCell(
        name: String,
        index: Int,
    ) {
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

    private fun renderButtonItem(name: String): I? {
        val render = bindings.buttonItems.getValue(name)
        return reactivity.withRendering(RenderKey.ButtonItem(name)) { render() }
    }

    private fun claimedRegions(control: String): List<String> =
        definition.controls.getValue(control).filter { it in claimed }

    /** The item bound to region [name]'s button or hotspot, else its hitbox item. */
    private fun regionItem(name: String): I? {
        val control = regionControls[name]
        if (control != null && control in bindings.buttonItems) return renderButtonItem(control)
        val hitbox = entry.regions.getValue(name).hitbox ?: return null
        val model = hitbox.itemModel?.let(Key::key) ?: definition.hitboxModel
        return buildItem(WindowItem.Hitbox(model, hitbox.tooltip?.let(::tooltip)))
    }

    /**
     * Renders [item] into the slots region [name] fills. Clicks route to every slot of the region, but a
     * repeater cell yields the slots owned by its item children, so their real stacks are never overwritten.
     */
    private fun fill(
        name: String,
        item: I?,
    ) {
        for (slot in entry.regions.getValue(name).filledSlots) items[slot.toApi()] = item
    }

    private fun tooltip(source: TooltipEntry): ButtonTooltip =
        tooltips.getOrPut(source) {
            ButtonTooltip(
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
