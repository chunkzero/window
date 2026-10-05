package com.chunkzero.window.internal

import com.chunkzero.window.ButtonTooltip
import com.chunkzero.window.WindowDefinition
import com.chunkzero.window.WindowItems
import com.chunkzero.window.manifest.AnvilInputEntry
import com.chunkzero.window.manifest.ButtonEntry
import com.chunkzero.window.manifest.ButtonState
import net.kyori.adventure.nbt.CompoundBinaryTag
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.format.TextDecoration
import net.kyori.adventure.text.minimessage.MiniMessage
import net.minestom.server.component.DataComponents
import net.minestom.server.item.ItemStack
import net.minestom.server.item.component.CustomData
import com.chunkzero.window.manifest.ButtonTooltip as ManifestTooltip

/**
 * Writes a window's inventory items: button and hotspot hitboxes, named button states, item
 * regions, collection cells, and anvil input seeds.
 */
internal class WindowInventoryWriter(
    private val definition: WindowDefinition,
    private val bindings: WindowBindings,
    private val handle: InventoryHandle,
    private val reactivity: Reactivity,
) {
    private val entry = definition.entry

    /** Current named state per button that has one. */
    private val buttonStateValues = HashMap<String, String>()

    /** Parsed manifest tooltips; components are immutable, so they are shared across renders. */
    private val tooltips = HashMap<ManifestTooltip, ButtonTooltip>()

    private var inputRevisions = 0

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

    /** Seeds every manifest-owned inventory slot after the inventory exists. */
    fun seed() {
        for (slotRect in entry.slotRects.values) {
            for (slot in slotRect.slots) handle.setItem(slot.toApi(), ItemStack.AIR)
        }
        for ((name, button) in entry.buttons) {
            applyButtonItem(button, seedButtonItem(name, button) ?: ItemStack.AIR)
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
    ): ItemStack? {
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

    /** Records [state] as button [name]'s current state and writes its item. */
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
        applyButtonItem(entry.buttons.getValue(name), renderButtonItem(name) ?: ItemStack.AIR)
    }

    fun setButtonItem(
        name: String,
        item: ItemStack?,
    ) {
        val button = definition.requireEntry(entry.buttons, name, "button or hotspot")
        applyButtonItem(button, item ?: ItemStack.AIR)
    }

    fun writeItem(name: String) {
        val render = bindings.items.getValue(name)
        val stack = reactivity.withRendering(RenderKey.Item(name)) { render() }
        setItem(name, stack)
    }

    fun setItem(
        name: String,
        stack: ItemStack?,
    ) {
        val item = definition.requireEntry(entry.items, name, "item", known = "items")
        for (slot in item.slots) handle.setItem(slot.toApi(), stack ?: ItemStack.AIR)
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
        val item = reactivity.withRendering(RenderKey.CollectionCell(name, index)) { render(index) }
        handle.setItem(slot.toApi(), item ?: ItemStack.AIR)
    }

    /**
     * Sends the input seed renamed to [value]. Each send carries a new revision, since the client only
     * resets its edit box to the seed's name when the item changes.
     */
    fun applyInput(
        input: AnvilInputEntry,
        value: String,
    ) {
        val revision = CompoundBinaryTag.builder().putInt(INPUT_REVISION, ++inputRevisions).build()
        val seed = inputSeed(input, value).with(DataComponents.CUSTOM_DATA, CustomData(revision))
        handle.setItem(input.slot.toApi(), seed)
    }

    /** Renames the input seed to [value] without sending it, so the next reopen restores [value]. */
    fun stageInput(
        input: AnvilInputEntry,
        value: String,
    ) {
        handle.stageItem(input.slot.toApi(), inputSeed(input, value))
    }

    /** Button [name]'s current named state, if it has one. */
    fun buttonState(name: String): String? = buttonStateValues[name]

    private fun inputSeed(
        input: AnvilInputEntry,
        value: String,
    ): ItemStack = WindowItems.anvilInput(value, input.itemModel ?: definition.hitboxModel)

    private fun renderButtonItem(name: String): ItemStack? {
        val render = bindings.buttonItems.getValue(name)
        return reactivity.withRendering(RenderKey.ButtonItem(name)) { render() }
    }

    /**
     * Writes [item] into the slots this button fills.
     *
     * Click routing covers every slot in [ButtonEntry.slots], but a repeater cell yields the slots
     * owned by its item children, so their real stacks are never overwritten by the cell's hitbox.
     */
    private fun applyButtonItem(
        button: ButtonEntry,
        item: ItemStack,
    ) {
        for (slot in button.filledSlots) handle.setItem(slot.toApi(), item)
    }

    private fun defaultButtonItem(button: ButtonEntry): ItemStack? {
        val defaultState = button.states["default"]
        if (defaultState != null) return itemForState(button, defaultState)
        val tooltip = button.tooltip?.let(::tooltip) ?: return null
        return WindowItems.hitbox(tooltip, definition.hitboxModel)
    }

    private fun itemForState(
        button: ButtonEntry,
        state: ButtonState,
    ): ItemStack {
        val tooltip = (state.tooltip ?: button.tooltip)?.let(::tooltip)
        val model = state.itemModel ?: definition.hitboxModel
        return WindowItems.hitbox(tooltip ?: ButtonTooltip(Component.empty()), model)
    }

    private fun tooltip(source: ManifestTooltip): ButtonTooltip =
        tooltips.getOrPut(source) {
            ButtonTooltip(
                parseTooltipLine(source.title, NamedTextColor.WHITE),
                source.lines.map { parseTooltipLine(it, NamedTextColor.GRAY) },
            )
        }

    private fun parseTooltipLine(
        template: String,
        defaultColor: TextColor,
    ): Component =
        Component
            .empty()
            .color(defaultColor)
            .decoration(TextDecoration.ITALIC, false)
            .append(MINI_MESSAGE.deserialize(template))

    private companion object {
        val MINI_MESSAGE: MiniMessage = MiniMessage.miniMessage()
        const val INPUT_REVISION = "window_input_revision"
    }
}
