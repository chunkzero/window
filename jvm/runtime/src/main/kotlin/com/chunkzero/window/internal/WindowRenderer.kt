package com.chunkzero.window.internal

import com.chunkzero.window.SlotRef
import com.chunkzero.window.Tooltip
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
    private val bindings: WindowBindings<I>,
    reactivity: Reactivity,
    private val buildItem: (WindowItem) -> I,
) {
    private val switches = bindings.switches
    private val title = WindowTitle(definition, bindings, reactivity)
    private val inventory = WindowInventory(definition, bindings, reactivity, buildItem)

    /** Selects every switch's initial case and renders the initial title. */
    fun seedTitle(): ComposedRender {
        for (name in switches.names) switches.update(name)
        for (name in bindings.buttonStates.keys) switches.select(name, inventory.renderButtonState(name))
        title.seed()
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
                is RenderKey.ButtonState -> select(key.name, inventory.renderButtonState(key.name))
                is RenderKey.Item -> inventory.writeItem(key.name)
                is RenderKey.CollectionCell -> inventory.writeCollectionCell(key.name, key.index)
                is RenderKey.CollectionSelection -> title.updateCollectionSelection(key.name)
                is RenderKey.Switch -> if (switches.update(key.name)) updateCases()
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
        tooltip: Tooltip?,
    ): SlotWrites<I> = setButtonItem(name, tooltip?.let { buildItem(definition.tooltipHitbox(it)) })

    /** Switches button [name] to [state]; null when it already has that state. */
    fun setButtonState(
        name: String,
        state: String,
    ): WindowFrame<I>? {
        definition.requireStates(name)
        if (!select(name, state)) return null
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

    /** Selects [state] in button [name]'s state switch; returns whether it changed. */
    private fun select(
        name: String,
        state: String,
    ): Boolean = switches.select(name, state).also { if (it) updateCases() }

    /** Swaps the art and regions of switch cases whose activity changed. */
    private fun updateCases() {
        title.updateCases()
        inventory.claimRegions()
    }

    private fun frame(): WindowFrame<I> = WindowFrame(title.compose(), inventory.drain())
}
