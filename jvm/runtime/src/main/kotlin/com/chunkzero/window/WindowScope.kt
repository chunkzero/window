package com.chunkzero.window

import net.kyori.adventure.text.Component

/**
 * The binding surface available inside [WindowView.bind].
 *
 * A view declares how each dynamic slot renders and how each button responds. Every dynamic slot in
 * the compiled definition must be bound exactly once; static labels (definition slots carrying
 * `text`) render automatically and must not be bound. Every action a region names must either be
 * bound here or have a `default_action`; actions in the `window:` namespace, such as
 * `window:close`, are run by the runtime and cannot be bound.
 *
 * Bindings are fixed once `bind()` returns: calling any binder on a retained scope afterwards, such
 * as from `onOpen`, throws [IllegalStateException].
 *
 * @param I the host's native item type.
 */
public interface WindowScope<I : Any> {
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
     * Binds the action [name] to a [handler] invoked on each click within the slots of an active
     * region naming it, such as a button's.
     *
     * @throws IllegalArgumentException if [name] is not an action of the window.
     */
    public fun button(
        name: String,
        handler: (Click) -> Unit,
    )

    /**
     * Binds a dynamic inventory item to a button or hotspot.
     *
     * The item replaces the hitbox item in the slots of the control's active region. Use this for
     * custom tooltip hitboxes and item-model based visual states.
     */
    public fun buttonItem(
        name: String,
        render: () -> I?,
    )

    /**
     * Binds a dynamic inventory item region.
     *
     * The rendered item is placed into every slot owned by the item region.
     */
    public fun item(
        name: String,
        render: () -> I?,
    )

    /**
     * Binds a dynamic repeated inventory item collection.
     *
     * [render] is called once per authored collection cell. [handler] receives the cell index that
     * was clicked.
     */
    public fun collection(
        name: String,
        render: (Int) -> I?,
        handler: (IndexedClick) -> Unit,
    )

    /**
     * Binds a repeated inventory item collection to [source]: its items, its clicks when the
     * collection accepts them, and its selected cell when the collection has a selected sprite.
     *
     * @throws IllegalArgumentException if [name] is not a collection of the window.
     */
    public fun collection(
        name: String,
        source: WindowCollection<I>,
    )

    /** Binds a display-only repeated inventory item collection. */
    public fun collectionItem(
        name: String,
        render: (Int) -> I?,
    )

    /**
     * Binds the selected cell of a collection with a selected sprite. [render] returns the cell
     * index to mark, or `null` (or an index outside the collection) to mark none.
     *
     * @throws IllegalArgumentException if [name] is not a collection with a selected sprite.
     */
    public fun collectionSelection(
        name: String,
        render: () -> Int?,
    )

    /**
     * Binds a switch to a [render] lambda returning the value of the case to draw. Only the active
     * case's art, text, sprite slots, and nested switches are drawn, and only its regions are
     * claimed; the lambda is re-invoked whenever a reactive state it reads changes. A binding shared
     * by switches in mutually exclusive cases selects the case of each.
     *
     * @throws IllegalArgumentException if [name] is not a switch of the window, or when rendering
     *   returns a value that is not one of its cases.
     */
    public fun switch(
        name: String,
        render: () -> String,
    )

    /**
     * Binds a native anvil rename field. The handler receives each value the player types. With
     * experimental anvil updates, the text the client sends back after a reopen is not passed on.
     */
    public fun anvilInput(
        name: String,
        handler: (String) -> Unit,
    )

    /** Places a fixed inventory [item] on a button or hotspot. */
    public fun buttonItem(
        name: String,
        item: I?,
    ) {
        buttonItem(name) { item }
    }

    /** Places a fixed inventory [item] on a dynamic item region. */
    public fun item(
        name: String,
        item: I?,
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

    /**
     * Selects one of the named states declared by the Window source: the button's art, hitbox item,
     * and tooltip follow the state. A bound state cannot also be set imperatively with
     * `WindowView.buttonState(name, state)`, which throws for bound buttons.
     */
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

    /**
     * Sets a fixed tooltip on the invisible hitbox item for a button or hotspot. The title defaults
     * to white and the lines to gray, neither italic.
     */
    public fun tooltip(
        name: String,
        tooltip: Tooltip?,
    )

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
}
