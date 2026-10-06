package com.chunkzero.window.internal

import com.chunkzero.window.ButtonTooltip
import com.chunkzero.window.SlotRef
import com.chunkzero.window.WindowDefinition
import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.manifest.AnvilInputEntry
import com.chunkzero.window.manifest.ButtonEntry
import com.chunkzero.window.manifest.ButtonState
import com.chunkzero.window.manifest.SlotAreaEntry
import com.chunkzero.window.manifest.TooltipEntry
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.minimessage.MiniMessage

/**
 * Renders a window's inventory items: button and hotspot hitboxes, named button states, item
 * regions, collection cells, and anvil input seeds. Rendered items collect until [drain] takes them.
 */
internal class WindowInventory<I : Any>(
    private val definition: WindowDefinition,
    private val bindings: WindowBindings<I>,
    private val reactivity: Reactivity,
    private val buildItem: (WindowItem) -> I,
) {
    private val entry = definition.entry

    /** Current named state per button that has one. */
    private val buttonStateValues = HashMap<String, String>()

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

    /** Resolves and records button [name]'s initial named state, if it has one. */
    fun initialButtonState(name: String): String? {
        val state =
            when {
                name in bindings.buttonStates -> renderButtonState(name)
                "default" in entry.buttons.getValue(name).states -> "default"
                else -> null
            }
        if (state != null) buttonStateValues[name] = state
        return state
    }

    /** Renders every manifest-owned inventory slot. */
    fun seed() {
        for (slotRect in entry.slotRects.values) {
            for (slot in slotRect.slots) items[slot.toApi()] = null
        }
        for ((name, button) in entry.buttons) {
            applyButtonItem(button, seedButtonItem(name, button))
        }
        for (name in entry.items.keys) writeItem(name)
        for ((name, collection) in entry.collections) {
            for (index in collection.slots.indices) writeCollectionCell(name, index)
        }
        for (input in entry.inputs.values) applyInput(input, input.initial)
    }

    private fun seedButtonItem(
        name: String,
        button: ButtonEntry,
    ): I? {
        val state = buttonStateValues[name]
        return when {
            state != null -> itemForState(button, definition.requireButtonState(name, state))
            name in bindings.buttonItems -> renderButtonItem(name)
            else -> defaultButtonItem(button)
        }
    }

    fun renderButtonState(name: String): String {
        val render = bindings.buttonStates.getValue(name)
        return reactivity.withRendering(RenderKey.ButtonState(name)) { render() }
    }

    /** Records [state] as button [name]'s current state and renders its item. */
    fun applyButtonState(
        name: String,
        state: String,
    ) {
        val value = definition.requireButtonState(name, state)
        buttonStateValues[name] = state
        val button = entry.buttons.getValue(name)
        applyButtonItem(button, itemForState(button, value))
    }

    fun writeButtonItem(name: String) {
        applyButtonItem(entry.buttons.getValue(name), renderButtonItem(name))
    }

    fun setButtonItem(
        name: String,
        item: I?,
    ) {
        val button = definition.requireEntry(entry.buttons, name, "button or hotspot")
        applyButtonItem(button, item)
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

    /** Button [name]'s current named state, if it has one. */
    fun buttonState(name: String): String? = buttonStateValues[name]

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

    /**
     * Renders [item] into the slots this button fills.
     *
     * Click routing covers every slot in [ButtonEntry.slots], but a repeater cell yields the slots
     * owned by its item children, so their real stacks are never overwritten by the cell's hitbox.
     */
    private fun applyButtonItem(
        button: ButtonEntry,
        item: I?,
    ) {
        for (slot in button.filledSlots) items[slot.toApi()] = item
    }

    private fun defaultButtonItem(button: ButtonEntry): I? {
        val defaultState = button.states["default"]
        if (defaultState != null) return itemForState(button, defaultState)
        val tooltip = button.tooltip?.let(::tooltip) ?: return null
        return buildItem(WindowItem.Hitbox(definition.hitboxModel, tooltip))
    }

    private fun itemForState(
        button: ButtonEntry,
        state: ButtonState,
    ): I {
        val tooltip = (state.tooltip ?: button.tooltip)?.let(::tooltip)
        val model = state.itemModel?.let(Key::key) ?: definition.hitboxModel
        return buildItem(WindowItem.Hitbox(model, tooltip))
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
