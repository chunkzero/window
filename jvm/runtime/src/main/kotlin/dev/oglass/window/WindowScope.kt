package dev.oglass.window

import net.kyori.adventure.text.Component
import net.minestom.server.item.ItemStack

/**
 * The binding surface available inside [WindowView.bind].
 *
 * A view declares how each dynamic slot renders and how each button responds. Every dynamic slot in
 * the compiled definition must be bound exactly once; static labels (definition slots carrying
 * `text`) render automatically and must not be bound. Every button must either be bound here or
 * carry a definition `default`.
 */
public interface WindowScope {
    /**
     * Binds a dynamic slot to a [render] lambda producing its current [Component].
     *
     * The lambda is re-invoked whenever a reactive state it reads changes. Its width is measured
     * from the plain-text content of the returned component (styling is preserved).
     *
     * @throws IllegalArgumentException if [name] is not a dynamic slot of the window.
     */
    public fun slot(
        name: String,
        render: () -> Component,
    )

    /**
     * Binds a runtime sprite slot to a [render] lambda producing a sprite id from the compiled
     * Window sprite catalog, or `null` to draw nothing.
     *
     * @throws IllegalArgumentException if [name] is not a dynamic sprite slot of the window.
     */
    public fun sprite(
        name: String,
        render: () -> String?,
    )

    /**
     * Binds a button to a [handler] invoked on each click within its mapped slots.
     *
     * @throws IllegalArgumentException if [name] is not a button of the window.
     */
    public fun button(
        name: String,
        handler: (Click) -> Unit,
    )

    /**
     * Binds a dynamic inventory item to a button or hotspot.
     *
     * The item is placed into every container slot covered by the region. Use this for custom
     * tooltip hitboxes and item-model based visual states.
     */
    public fun buttonItem(
        name: String,
        render: () -> ItemStack?,
    )

    /**
     * Binds a dynamic inventory item region.
     *
     * The rendered item is placed into every slot owned by the item region.
     */
    public fun item(
        name: String,
        render: () -> ItemStack?,
    )

    /**
     * Binds a dynamic repeated inventory item collection.
     *
     * [render] is called once per authored collection cell. [handler] receives the cell index that
     * was clicked.
     */
    public fun collection(
        name: String,
        render: (Int) -> ItemStack?,
        handler: (IndexedClick) -> Unit,
    )

    /** Binds a display-only repeated inventory item collection. */
    public fun collectionItem(
        name: String,
        render: (Int) -> ItemStack?,
    )

    /**
     * Binds a native anvil rename field. The handler receives each value the player types; the text
     * the client sends back after a title change reopens the anvil is not passed on.
     */
    public fun anvilInput(
        name: String,
        handler: (String) -> Unit,
    )

    /** Places a fixed inventory [item] on a button or hotspot. */
    public fun buttonItem(
        name: String,
        item: ItemStack?,
    ) {
        buttonItem(name) { item }
    }

    /** Places a fixed inventory [item] on a dynamic item region. */
    public fun item(
        name: String,
        item: ItemStack?,
    ) {
        item(name) { item }
    }

    /** Places a fixed sprite id in a runtime sprite slot. */
    public fun sprite(
        name: String,
        sprite: String?,
    ) {
        sprite(name) { sprite }
    }

    /** Uses one of the named states declared by the Window source. */
    public fun buttonState(
        name: String,
        render: () -> String,
    )

    /** Uses a fixed named state declared by the Window source. */
    public fun buttonState(
        name: String,
        state: String,
    ) {
        buttonState(name) { state }
    }

    /** Sets a fixed tooltip on the invisible hitbox item for a button or hotspot. */
    public fun tooltip(
        name: String,
        tooltip: ButtonTooltip?,
    ) {
        buttonItem(name, tooltip?.let { WindowItems.hitbox(it) })
    }

    /**
     * Binds a two-state toggle using states named [onState] and [offState], then registers
     * [handler] as the click action.
     */
    public fun toggle(
        name: String,
        selected: () -> Boolean,
        onState: String = "on",
        offState: String = "off",
        handler: (Click) -> Unit,
    ) {
        buttonState(name) { if (selected()) onState else offState }
        button(name, handler)
    }

    /**
     * Binds one button in a mutually exclusive choice group using `selected`/`unselected` states.
     */
    public fun <T> choice(
        name: String,
        value: T,
        selected: () -> T,
        selectedState: String = "selected",
        unselectedState: String = "unselected",
        handler: (T, Click) -> Unit,
    ) {
        buttonState(name) { if (selected() == value) selectedState else unselectedState }
        button(name) { click -> handler(value, click) }
    }

    /**
     * Binds an enabled/disabled button, updates its item state reactively, and suppresses disabled
     * clicks.
     */
    public fun enabledButton(
        name: String,
        enabled: () -> Boolean,
        enabledState: String = "enabled",
        disabledState: String = "disabled",
        handler: (Click) -> Unit,
    ) {
        buttonState(name) { if (enabled()) enabledState else disabledState }
        button(name) { click -> if (enabled()) handler(click) }
    }

    /** Binds previous/next controls to a [WindowPager] and reactive offset/total suppliers. */
    public fun pager(
        previous: String,
        next: String,
        pager: WindowPager,
        offset: () -> Int,
        total: () -> Int,
        onOffsetChanged: (Int) -> Unit,
    ) {
        enabledButton(previous, { pager.canPrevious(offset()) }) {
            onOffsetChanged(pager.previous(offset(), total()))
        }
        enabledButton(next, { pager.canNext(offset(), total()) }) {
            onOffsetChanged(pager.next(offset(), total()))
        }
    }
}
