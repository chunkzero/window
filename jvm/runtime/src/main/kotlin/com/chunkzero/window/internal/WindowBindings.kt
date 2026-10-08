package com.chunkzero.window.internal

import com.chunkzero.window.ButtonTooltip
import com.chunkzero.window.Click
import com.chunkzero.window.IndexedClick
import com.chunkzero.window.WindowCollection
import com.chunkzero.window.WindowDefinition
import com.chunkzero.window.WindowScope
import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.manifest.WindowEntry
import net.kyori.adventure.text.Component

/**
 * Collects a window view's bindings: the [WindowScope] passed to `bind`, the resulting render and
 * handler tables, and the check that every manifest control requiring a binding received one.
 */
internal class WindowBindings<I : Any>(
    private val definition: WindowDefinition,
    private val buildItem: (WindowItem) -> I,
    val slots: DynamicSlots,
    val switches: Switches,
) : WindowScope<I> {
    private val entry = definition.entry

    /** The sprite slot keys of each binding name; a binding shared across switch cases has one per case. */
    private val spriteBindings = entry.spriteSlots.keys.groupBy { entry.spriteSlots.getValue(it).binding ?: it }

    /** The action ids regions name that a handler can bind; runtime actions are resolved by the runtime. */
    private val actions: Set<String> =
        entry.regions.values
            .mapNotNull { it.action }
            .filterTo(HashSet()) { RuntimeAction.of(it) == null }

    val sprites = HashMap<String, () -> String?>()

    /** Click handlers by action id. */
    val buttonHandlers = HashMap<String, (Click) -> Unit>()

    /** Item renders by button or hotspot name. */
    val buttonItems = HashMap<String, () -> I?>()

    /** State renders by button or hotspot name. */
    val buttonStates = HashMap<String, () -> String>()
    val items = HashMap<String, () -> I?>()
    val collectionItems = HashMap<String, (Int) -> I?>()
    val collectionHandlers = HashMap<String, (IndexedClick) -> Unit>()
    val collectionSelections = HashMap<String, () -> Int?>()
    val inputHandlers = LinkedHashMap<String, (String) -> Unit>()

    override fun slot(
        name: String,
        render: () -> Component,
    ) {
        requireBindable(name)
        slots.bind(name, render)
    }

    override fun sprite(
        name: String,
        render: () -> String?,
    ) {
        requireBindable(name)
        val keys = definition.requireEntry(spriteBindings, name, "sprite slot", known = "sprite slots")
        for (key in keys) {
            require(entry.spriteSlots.getValue(key).sprite == null) {
                "Sprite slot '$name' has a fixed authored sprite and must not be bound"
            }
            bindOnce(sprites, key, render) { "Sprite slot '$name' bound more than once" }
        }
    }

    override fun button(
        name: String,
        handler: (Click) -> Unit,
    ) {
        requireBindable(name)
        if (name !in actions) {
            require(name !in definition.controls) { "Button '$name' is a hotspot and cannot be bound as an action" }
            throw IllegalArgumentException(
                "Unknown button '$name' in window '${definition.name}'; known buttons: ${actions.sorted()}",
            )
        }
        bindOnce(buttonHandlers, name, handler) { "Button '$name' bound more than once" }
    }

    override fun buttonItem(
        name: String,
        render: () -> I?,
    ) {
        requireBindable(name)
        definition.requireEntry(definition.controls, name, "button or hotspot", known = "buttons")
        require(name !in buttonStates) { "Button '$name' already has a named-state binding" }
        bindOnce(buttonItems, name, render) { "Button item '$name' bound more than once" }
    }

    override fun item(
        name: String,
        render: () -> I?,
    ) {
        requireBindable(name)
        definition.requireEntry(entry.items, name, "item", known = "items")
        bindOnce(items, name, render) { "Item '$name' bound more than once" }
    }

    override fun collection(
        name: String,
        render: (Int) -> I?,
        handler: (IndexedClick) -> Unit,
    ) {
        requireBindable(name)
        val collection = definition.requireEntry(entry.collections, name, "collection", known = "collections")
        require(collection.action) {
            "Collection '$name' is display-only and cannot be bound as an action"
        }
        collectionItem(name, render)
        bindOnce(collectionHandlers, name, handler) { "Collection '$name' handler bound more than once" }
    }

    override fun collection(
        name: String,
        source: WindowCollection<I>,
    ) {
        requireBindable(name)
        val collection = definition.requireEntry(entry.collections, name, "collection", known = "collections")
        if (collection.action) {
            collection(name, source::item) { source.click(it.index, Click(it.slot, it.shift, it.right)) }
        } else {
            collectionItem(name, source::item)
        }
        if (collection.selection.isNotEmpty()) collectionSelection(name, source::selected)
    }

    override fun collectionItem(
        name: String,
        render: (Int) -> I?,
    ) {
        requireBindable(name)
        definition.requireEntry(entry.collections, name, "collection", known = "collections")
        bindOnce(collectionItems, name, render) {
            "Collection '$name' item renderer bound more than once"
        }
    }

    override fun collectionSelection(
        name: String,
        render: () -> Int?,
    ) {
        requireBindable(name)
        val collection = definition.requireEntry(entry.collections, name, "collection", known = "collections")
        require(collection.selection.isNotEmpty()) { "Collection '$name' has no selected sprite" }
        bindOnce(collectionSelections, name, render) { "Collection '$name' selection bound more than once" }
    }

    override fun switch(
        name: String,
        render: () -> String,
    ) {
        requireBindable(name)
        switches.bind(name, render)
    }

    override fun anvilInput(
        name: String,
        handler: (String) -> Unit,
    ) {
        requireBindable(name)
        definition.requireEntry(entry.inputs, name, "anvil input", known = "inputs")
        bindOnce(inputHandlers, name, handler) { "Anvil input '$name' bound more than once" }
    }

    override fun buttonState(
        name: String,
        render: () -> String,
    ) {
        requireBindable(name)
        definition.requireStates(name)
        require(name !in buttonItems) { "Button '$name' already has an item binding" }
        bindOnce(buttonStates, name, render) { "Button state '$name' bound more than once" }
    }

    override fun tooltip(
        name: String,
        tooltip: ButtonTooltip?,
    ) {
        requireBindable(name)
        val item = tooltip?.let { buildItem(definition.tooltipHitbox(it)) }
        buttonItem(name) { item }
    }

    private var sealed = false

    /** Rejects further bindings; called once `bind()` has returned. */
    fun seal() {
        sealed = true
    }

    private fun requireBindable(name: String) {
        check(!sealed) { "Bindings are fixed once bind() returns; declare '$name' in bind()" }
    }

    /**
     * Fails fast on unbound dynamic content, on actions lacking both a handler and a default, and on
     * unknown runtime actions.
     */
    fun validate() {
        slots.validate()
        switches.validate()
        val unboundSprites =
            entry.spriteSlots
                .filter { (key, slot) -> slot.sprite == null && key !in sprites }
                .map { (key, slot) -> slot.binding ?: key }
        checkNone(unboundSprites.toSet()) { "Unbound dynamic sprite slots in window '$it'" }
        val defaults = entry.regions.values.mapNotNull { it.defaultAction }
        val unknownDefaults = defaults.filterTo(HashSet()) { RuntimeAction.of(it) == null }
        checkNone(unknownDefaults) { "Unknown runtime actions in window '$it'" }
        val actionsWithoutDefault =
            entry.regions.values
                .filter { it.defaultAction == null }
                .mapNotNull { it.action }
        checkNone(actionsWithoutDefault.toSet() - buttonHandlers.keys) {
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

/** Throws unless button or hotspot [name] has named states. */
internal fun WindowDefinition.requireStates(name: String) {
    if (entry.switches[name]?.states == true) return
    requireEntry(controls, name, "button or hotspot")
    throw IllegalArgumentException("Button '$name' in window '${this.name}' has no named states")
}

/**
 * The region keys of each button or hotspot: the region keyed by its name, or every region of the
 * cases of its state switch. Claim-only regions belong to no control.
 */
internal fun WindowEntry.controlRegions(): Map<String, List<String>> {
    val stateful = switches.filterValues { it.states }.mapValues { (_, switch) -> switch.cases.flatMap { it.regions } }
    val owned = stateful.values.flatten().toSet()
    val plain = regions.filter { (key, region) -> key !in owned && (region.action != null || region.hitbox != null) }
    return plain.mapValues { (key, _) -> listOf(key) } + stateful
}
