package com.chunkzero.window.internal

import com.chunkzero.window.ButtonTooltip
import com.chunkzero.window.Click
import com.chunkzero.window.IndexedClick
import com.chunkzero.window.WindowDefinition
import com.chunkzero.window.WindowItems
import com.chunkzero.window.WindowScope
import com.chunkzero.window.manifest.ButtonState
import net.kyori.adventure.text.Component
import net.minestom.server.item.ItemStack

/**
 * Collects a window view's bindings: the [WindowScope] passed to `bind`, the resulting render and
 * handler tables, and the check that every manifest control requiring a binding received one.
 */
internal class WindowBindings(
    private val definition: WindowDefinition,
    val slots: DynamicSlots,
    val switches: Switches,
) : WindowScope {
    private val entry = definition.entry

    val sprites = HashMap<String, () -> String?>()
    val buttonHandlers = HashMap<String, (Click) -> Unit>()
    val buttonItems = HashMap<String, () -> ItemStack?>()
    val buttonStates = HashMap<String, () -> String>()
    val items = HashMap<String, () -> ItemStack?>()
    val collectionItems = HashMap<String, (Int) -> ItemStack?>()
    val collectionHandlers = HashMap<String, (IndexedClick) -> Unit>()
    val collectionSelections = HashMap<String, () -> Int?>()
    val inputHandlers = LinkedHashMap<String, (String) -> Unit>()

    override fun slot(
        name: String,
        render: () -> Component,
    ) = slots.bind(name, render)

    override fun sprite(
        name: String,
        render: () -> String?,
    ) {
        val slot = definition.requireEntry(entry.spriteSlots, name, "sprite slot", known = "sprite slots")
        require(slot.sprite == null) {
            "Sprite slot '$name' has a fixed authored sprite and must not be bound"
        }
        bindOnce(sprites, name, render) { "Sprite slot '$name' bound more than once" }
    }

    override fun button(
        name: String,
        handler: (Click) -> Unit,
    ) {
        val button = definition.requireEntry(entry.buttons, name, "button", known = "buttons")
        require(button.action) { "Button '$name' is a hotspot and cannot be bound as an action" }
        bindOnce(buttonHandlers, name, handler) { "Button '$name' bound more than once" }
    }

    override fun buttonItem(
        name: String,
        render: () -> ItemStack?,
    ) {
        definition.requireEntry(entry.buttons, name, "button or hotspot", known = "buttons")
        require(name !in buttonStates) { "Button '$name' already has a named-state binding" }
        bindOnce(buttonItems, name, render) { "Button item '$name' bound more than once" }
    }

    override fun item(
        name: String,
        render: () -> ItemStack?,
    ) {
        definition.requireEntry(entry.items, name, "item", known = "items")
        bindOnce(items, name, render) { "Item '$name' bound more than once" }
    }

    override fun collection(
        name: String,
        render: (Int) -> ItemStack?,
        handler: (IndexedClick) -> Unit,
    ) {
        val collection = definition.requireEntry(entry.collections, name, "collection", known = "collections")
        require(collection.action) {
            "Collection '$name' is display-only and cannot be bound as an action"
        }
        collectionItem(name, render)
        bindOnce(collectionHandlers, name, handler) { "Collection '$name' handler bound more than once" }
    }

    override fun collectionItem(
        name: String,
        render: (Int) -> ItemStack?,
    ) {
        definition.requireEntry(entry.collections, name, "collection", known = "collections")
        bindOnce(collectionItems, name, render) {
            "Collection '$name' item renderer bound more than once"
        }
    }

    override fun collectionSelection(
        name: String,
        render: () -> Int?,
    ) {
        val collection = definition.requireEntry(entry.collections, name, "collection", known = "collections")
        require(collection.selection.isNotEmpty()) { "Collection '$name' has no selected sprite" }
        bindOnce(collectionSelections, name, render) { "Collection '$name' selection bound more than once" }
    }

    override fun switch(
        name: String,
        render: () -> String,
    ) = switches.bind(name, render)

    override fun anvilInput(
        name: String,
        handler: (String) -> Unit,
    ) {
        definition.requireEntry(entry.inputs, name, "anvil input", known = "inputs")
        bindOnce(inputHandlers, name, handler) { "Anvil input '$name' bound more than once" }
    }

    override fun buttonState(
        name: String,
        render: () -> String,
    ) {
        definition.requireEntry(entry.buttons, name, "button or hotspot")
        require(name !in buttonItems) { "Button '$name' already has an item binding" }
        bindOnce(buttonStates, name, render) { "Button state '$name' bound more than once" }
    }

    override fun tooltip(
        name: String,
        tooltip: ButtonTooltip?,
    ) {
        buttonItem(name) { tooltip?.let { WindowItems.hitbox(it, definition.hitboxModel) } }
    }

    /** Fails fast on unbound dynamic content and on actions lacking both a handler and a default. */
    fun validate() {
        slots.validate()
        switches.validate()
        val dynamicSprites = entry.spriteSlots.filterValues { it.sprite == null }.keys
        checkNone(dynamicSprites - sprites.keys) { "Unbound dynamic sprite slots in window '$it'" }
        val actionsWithoutDefault = entry.buttons.filterValues { it.action && it.default == null }.keys
        checkNone(actionsWithoutDefault - buttonHandlers.keys) {
            "Buttons in window '$it' have neither a handler nor a default"
        }
        checkNone(entry.items.keys - items.keys) { "Unbound dynamic items in window '$it'" }
        checkNone(entry.collections.keys - collectionItems.keys) { "Unbound collection items in window '$it'" }
        val actionCollections = entry.collections.filterValues { it.action }.keys
        checkNone(actionCollections - collectionHandlers.keys) { "Collections in window '$it' have no handler" }
        checkNone(entry.inputs.keys - inputHandlers.keys) { "Unbound anvil inputs in window '$it'" }
    }

    /** Every bound render target, for a full refresh. */
    fun renderKeys(): Set<RenderKey> =
        buildSet {
            slots.names.mapTo(this, RenderKey::Slot)
            sprites.keys.mapTo(this, RenderKey::Sprite)
            buttonItems.keys.mapTo(this, RenderKey::ButtonItem)
            buttonStates.keys.mapTo(this, RenderKey::ButtonState)
            items.keys.mapTo(this, RenderKey::Item)
            collectionSelections.keys.mapTo(this, RenderKey::CollectionSelection)
            switches.names.mapTo(this, RenderKey::Switch)
            for ((name, collection) in entry.collections) {
                if (name !in collectionItems) continue
                collection.slots.indices.mapTo(this) { RenderKey.CollectionCell(name, it) }
            }
        }

    private fun checkNone(
        missing: Set<String>,
        describe: (windowName: String) -> String,
    ) {
        check(missing.isEmpty()) { "${describe(definition.name)}: ${missing.sorted()}" }
    }

    private fun <V> bindOnce(
        bindings: MutableMap<String, V>,
        name: String,
        value: V,
        duplicate: () -> String,
    ) {
        require(bindings.put(name, value) == null, duplicate)
    }
}

/**
 * Returns the manifest entry [name] from [entries], or throws naming the [kind] and, when [known]
 * labels them, the valid names.
 */
internal fun <V> WindowDefinition.requireEntry(
    entries: Map<String, V>,
    name: String,
    kind: String,
    known: String? = null,
): V =
    entries[name] ?: throw IllegalArgumentException(
        "Unknown $kind '$name' in window '${this.name}'" +
            (known?.let { "; known $it: ${entries.keys.sorted()}" } ?: ""),
    )

/** Returns the named [state] of button [name], or throws listing the button's states. */
internal fun WindowDefinition.requireButtonState(
    name: String,
    state: String,
): ButtonState {
    val button = requireEntry(entry.buttons, name, "button or hotspot")
    return button.states[state]
        ?: throw IllegalArgumentException(
            "Unknown state '$state' for button '$name' in window '${this.name}'; " +
                "known states: ${button.states.keys.sorted()}",
        )
}

/** The item model of this window's invisible hitbox items. */
internal val WindowDefinition.hitboxModel: String
    get() = "${manifest.namespace}:gui/hitbox"
