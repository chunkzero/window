package com.chunkzero.window.internal

import com.chunkzero.window.ButtonTooltip
import com.chunkzero.window.SlotRef
import com.chunkzero.window.WindowDefinition
import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.manifest.AnvilInputEntry

/** Items to show: container and player slots to set, then container slots staged for the next title change. */
internal class SlotWrites<I : Any>(
    val items: Map<SlotRef, I?>,
    val staged: Map<Int, I?>,
)

/** A composed window title with the items rendered alongside it. */
internal class WindowFrame<I : Any>(
    val title: ComposedRender,
    val writes: SlotWrites<I>,
)

/**
 * Renders a window from its definition and a view's [bindings]: the title and the inventory items. It evaluates the
 * bound providers under [reactivity]'s dependency capture and returns what to show; delivering it is up to the caller.
 */
internal class WindowRenderer<I : Any>(
    private val definition: WindowDefinition,
    bindings: WindowBindings<I>,
    reactivity: Reactivity,
    private val buildItem: (WindowItem) -> I,
) {
    private val entry = definition.entry
    private val title = WindowTitle(definition, bindings, reactivity)
    private val inventory = WindowInventory(definition, bindings, reactivity, buildItem)

    /**
     * Renders the initial title: switch case art first, then button state sprites, so both draw beneath sprite and text
     * slots. Segments updated later keep this position, since title segments compose in insertion order.
     */
    fun seedTitle(): ComposedRender {
        for (name in entry.switches.keys) title.updateSwitch(name)
        for (name in entry.buttons.keys) {
            val state = inventory.initialButtonState(name)
            if (state != null) title.setButtonVisual(name, state) else title.reserveButtonVisual(name)
        }
        title.seedContent()
        return title.compose()
    }

    /** Renders every manifest-owned inventory slot; call after [seedTitle]. */
    fun seedItems(): SlotWrites<I> {
        inventory.seed()
        return inventory.drain()
    }

    /** Re-renders the [dirty] keys. */
    fun render(dirty: Set<RenderKey>): WindowFrame<I> {
        for (key in dirty) {
            when (key) {
                is RenderKey.Slot -> title.updateSlot(key.name)
                is RenderKey.Sprite -> title.updateSprite(key.name)
                is RenderKey.ButtonItem -> inventory.writeButtonItem(key.name)
                is RenderKey.ButtonState -> applyButtonState(key.name, inventory.renderButtonState(key.name))
                is RenderKey.Item -> inventory.writeItem(key.name)
                is RenderKey.CollectionCell -> inventory.writeCollectionCell(key.name, key.index)
                is RenderKey.CollectionSelection -> title.updateCollectionSelection(key.name)
                is RenderKey.Switch -> title.updateSwitch(key.name)
            }
        }
        return frame()
    }

    /** Takes the slot writes rendered but not yet delivered, such as those of a render that threw. */
    fun drainWrites(): SlotWrites<I> = inventory.drain()

    /** Composes the current title. */
    fun title(): ComposedRender = title.compose()

    fun setItem(
        name: String,
        item: I?,
    ): SlotWrites<I> {
        inventory.setItem(name, item)
        return inventory.drain()
    }

    fun setButtonItem(
        name: String,
        item: I?,
    ): SlotWrites<I> {
        inventory.setButtonItem(name, item)
        return inventory.drain()
    }

    fun setTooltip(
        name: String,
        tooltip: ButtonTooltip?,
    ): SlotWrites<I> {
        val button = definition.requireEntry(definition.entry.buttons, name, "button or hotspot")
        return setButtonItem(name, tooltip?.let { buildItem(definition.tooltipHitbox(button, it)) })
    }

    /** Switches button [name] to [state]; null when it already has that state. */
    fun setButtonState(
        name: String,
        state: String,
    ): WindowFrame<I>? {
        if (inventory.buttonState(name) == state) return null
        applyButtonState(name, state)
        return frame()
    }

    /** Renders [input]'s seed renamed to [value], which sets the client's edit box in place. */
    fun applyInput(
        input: AnvilInputEntry,
        value: String,
    ): SlotWrites<I> {
        inventory.applyInput(input, value)
        return inventory.drain()
    }

    /** Stages [input]'s seed renamed to [value], since reopens and resyncs reset the edit box to it. */
    fun stageInput(
        input: AnvilInputEntry,
        value: String,
    ): SlotWrites<I> {
        inventory.stageInput(input, value)
        return inventory.drain()
    }

    private fun applyButtonState(
        name: String,
        state: String,
    ) {
        inventory.applyButtonState(name, state)
        title.setButtonVisual(name, state)
    }

    private fun frame(): WindowFrame<I> = WindowFrame(title.compose(), inventory.drain())
}
