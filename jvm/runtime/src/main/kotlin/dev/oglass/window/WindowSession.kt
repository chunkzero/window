package dev.oglass.window

import dev.oglass.window.diagnostics.RenderBounds
import dev.oglass.window.diagnostics.RenderCorrelation
import dev.oglass.window.diagnostics.RenderCursorConvention
import dev.oglass.window.diagnostics.RenderFrame
import dev.oglass.window.diagnostics.RenderFrameReason
import dev.oglass.window.diagnostics.RenderLayerKind
import dev.oglass.window.diagnostics.RenderLayerTrace
import dev.oglass.window.diagnostics.RenderStyleTrace
import dev.oglass.window.diagnostics.RenderSurfaceKind
import dev.oglass.window.internal.ClickInfo
import dev.oglass.window.internal.InventoryHandle
import dev.oglass.window.internal.LiveInventoryHandle
import dev.oglass.window.internal.Reactivity
import dev.oglass.window.internal.RenderScheduler
import dev.oglass.window.internal.RenderedSegment
import dev.oglass.window.manifest.Align
import dev.oglass.window.manifest.AnvilInputEntry
import dev.oglass.window.manifest.ButtonDefault
import dev.oglass.window.manifest.ButtonEntry
import dev.oglass.window.manifest.ButtonState
import dev.oglass.window.manifest.CollectionEntry
import dev.oglass.window.manifest.ItemEntry
import dev.oglass.window.manifest.SlotAreaEntry
import dev.oglass.window.manifest.SlotRectEntry
import dev.oglass.window.manifest.SlotRefEntry
import dev.oglass.window.manifest.SpriteSlotEntry
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.format.TextDecoration
import net.kyori.adventure.text.minimessage.MiniMessage
import net.minestom.server.entity.Player
import net.minestom.server.inventory.Inventory
import net.minestom.server.item.ItemStack
import org.slf4j.LoggerFactory
import java.util.UUID
import java.util.concurrent.atomic.AtomicLong

/**
 * A live window instance bound to a player.
 *
 * Owns the Minestom [inventory], routes clicks to button handlers, drives reactive slot re-renders,
 * and manages the window lifecycle. Created via [Windows.open]; not constructed directly.
 */
public class WindowSession
    internal constructor(
        private val definition: WindowDefinition,
        /** The view driving this session. */
        public val view: WindowView,
        private val player: Player,
        scheduler: RenderScheduler,
        private val handle: InventoryHandle,
        private val diagnosticsObserver: RenderDiagnosticsObserver = RenderDiagnosticsObserver.NONE,
    ) {
        private val entry = definition.entry
        private val composer = definition.composer

        /** Dynamic-slot render lambdas, by slot name. Populated during [bind]. */
        private val slotRenders = HashMap<String, () -> Component>()

        /** Runtime sprite render lambdas, by sprite slot name. Populated during [bind]. */
        private val spriteRenders = HashMap<String, () -> String?>()

        /** Button handlers, by button name. Populated during [bind]. */
        private val buttonHandlers = HashMap<String, (Click) -> Unit>()

        /** Dynamic button/hotspot item render lambdas, by button name. Populated during [bind]. */
        private val buttonItemRenders = HashMap<String, () -> ItemStack?>()

        /** Named-state render lambdas that update both hitbox items and title sprites. */
        private val buttonStateRenders = HashMap<String, () -> String>()
        private val buttonStateValues = HashMap<String, String>()

        /** Dynamic item render lambdas, by item name. Populated during [bind]. */
        private val itemRenders = HashMap<String, () -> ItemStack?>()

        /** Dynamic collection item render lambdas, by collection name. Populated during [bind]. */
        private val collectionItemRenders = HashMap<String, (Int) -> ItemStack?>()

        /** Collection handlers, by collection name. Populated during [bind]. */
        private val collectionHandlers = HashMap<String, (IndexedClick) -> Unit>()

        /** Native anvil-input handlers, by input name. */
        private val inputHandlers = LinkedHashMap<String, (String) -> Unit>()

        /** Typed backing slot -> routed control, for click routing. */
        private val slotRoutes = HashMap<SlotRef, Route>()

        /** Last composed net-zero segment per slot/static label/runtime sprite. */
        private val slotSegments = LinkedHashMap<String, RenderedSegment>()

        private val renderSessionId = UUID.randomUUID().toString()
        private val nextFrameId = AtomicLong(1)

        private val reactivity = Reactivity(scheduler) { dirty -> flush(dirty) }

        private var opened = false
        private var closing = false
        private var closed = false

        /** The names of dynamic slots (definition slots without a `text` label). */
        private val dynamicSlotNames: Set<String> = entry.slots.filterValues { it.text == null }.keys

        /** The names of runtime sprite slots. */
        private val dynamicSpriteNames: Set<String> =
            entry.spriteSlots.filterValues { it.sprite == null }.keys

        /** The Minestom inventory backing this window. */
        public val inventory: Inventory
            get() =
                (handle as? LiveInventoryHandle)?.inventory
                    ?: error("This session is not backed by a live inventory")

        internal fun open() {
            check(!opened) { "Session already opened" }
            view.attach(player, this, reactivity)

            collectBindings()
            validateBindings()
            seedSegments()

            val render = composer.compose(definition.name, slotSegments)
            handle.open(render.component)
            observe(render.layers, RenderFrameReason.OPEN)
            handle.registerListeners(::handleClick, ::handleClientClose, ::handleInput)
            seedInventoryItems()
            view.invokeOnOpen()
            opened = true
        }

        /** Collects slot/button bindings by running the view's bind block. */
        private fun collectBindings() {
            view.invokeBind(
                object : WindowScope {
                    override fun slot(
                        name: String,
                        render: () -> Component,
                    ) {
                        val slot =
                            entry.slots[name]
                                ?: throw IllegalArgumentException(
                                    "Unknown slot '$name' in window '${definition.name}'; " +
                                        "known slots: ${dynamicSlotNames.sorted()}",
                                )
                        require(slot.text == null) {
                            "Slot '$name' is a static label and must not be bound"
                        }
                        require(slotRenders.put(name, render) == null) {
                            "Slot '$name' bound more than once"
                        }
                    }

                    override fun sprite(
                        name: String,
                        render: () -> String?,
                    ) {
                        val slot =
                            entry.spriteSlots[name]
                                ?: throw IllegalArgumentException(
                                    "Unknown sprite slot '$name' in window '${definition.name}'; " +
                                        "known sprite slots: ${entry.spriteSlots.keys.sorted()}",
                                )
                        require(slot.sprite == null) {
                            "Sprite slot '$name' has a fixed authored sprite and must not be bound"
                        }
                        require(spriteRenders.put(name, render) == null) {
                            "Sprite slot '$name' bound more than once"
                        }
                    }

                    override fun button(
                        name: String,
                        handler: (Click) -> Unit,
                    ) {
                        val button =
                            entry.buttons[name]
                                ?: throw IllegalArgumentException(
                                    "Unknown button '$name' in window '${definition.name}'; " +
                                        "known buttons: ${entry.buttons.keys.sorted()}",
                                )
                        require(button.action) {
                            "Button '$name' is a hotspot and cannot be bound as an action"
                        }
                        require(buttonHandlers.put(name, handler) == null) {
                            "Button '$name' bound more than once"
                        }
                    }

                    override fun buttonItem(
                        name: String,
                        render: () -> ItemStack?,
                    ) {
                        entry.buttons[name]
                            ?: throw IllegalArgumentException(
                                "Unknown button or hotspot '$name' in window '${definition.name}'; " +
                                    "known buttons: ${entry.buttons.keys.sorted()}",
                            )
                        require(name !in buttonStateRenders) {
                            "Button '$name' already has a named-state binding"
                        }
                        require(buttonItemRenders.put(name, render) == null) {
                            "Button item '$name' bound more than once"
                        }
                    }

                    override fun item(
                        name: String,
                        render: () -> ItemStack?,
                    ) {
                        entry.items[name]
                            ?: throw IllegalArgumentException(
                                "Unknown item '$name' in window '${definition.name}'; " +
                                    "known items: ${entry.items.keys.sorted()}",
                            )
                        require(itemRenders.put(name, render) == null) {
                            "Item '$name' bound more than once"
                        }
                    }

                    override fun collection(
                        name: String,
                        render: (Int) -> ItemStack?,
                        handler: (IndexedClick) -> Unit,
                    ) {
                        val collection =
                            entry.collections[name]
                                ?: throw IllegalArgumentException(
                                    "Unknown collection '$name' in window '${definition.name}'; " +
                                        "known collections: ${entry.collections.keys.sorted()}",
                                )
                        require(collection.action) {
                            "Collection '$name' is display-only and cannot be bound as an action"
                        }
                        collectionItem(name, render)
                        require(collectionHandlers.put(name, handler) == null) {
                            "Collection '$name' handler bound more than once"
                        }
                    }

                    override fun collectionItem(
                        name: String,
                        render: (Int) -> ItemStack?,
                    ) {
                        entry.collections[name]
                            ?: throw IllegalArgumentException(
                                "Unknown collection '$name' in window '${definition.name}'; " +
                                    "known collections: ${entry.collections.keys.sorted()}",
                            )
                        require(collectionItemRenders.put(name, render) == null) {
                            "Collection '$name' item renderer bound more than once"
                        }
                    }

                    override fun anvilInput(
                        name: String,
                        handler: (String) -> Unit,
                    ) {
                        entry.inputs[name]
                            ?: throw IllegalArgumentException(
                                "Unknown anvil input '$name' in window '${definition.name}'; " +
                                    "known inputs: ${entry.inputs.keys.sorted()}",
                            )
                        require(inputHandlers.put(name, handler) == null) {
                            "Anvil input '$name' bound more than once"
                        }
                    }

                    override fun buttonState(
                        name: String,
                        render: () -> String,
                    ) {
                        entry.buttons[name]
                            ?: throw IllegalArgumentException(
                                "Unknown button or hotspot '$name' in window '${definition.name}'",
                            )
                        require(name !in buttonItemRenders) {
                            "Button '$name' already has an item binding"
                        }
                        require(buttonStateRenders.put(name, render) == null) {
                            "Button state '$name' bound more than once"
                        }
                    }

                    override fun tooltip(
                        name: String,
                        tooltip: ButtonTooltip?,
                    ) {
                        buttonItem(name) { tooltip?.let { hitbox(it) } }
                    }
                },
            )
        }

        /** Fails fast on unbound dynamic slots and buttons lacking both a handler and a default. */
        private fun validateBindings() {
            val unbound = dynamicSlotNames - slotRenders.keys
            check(unbound.isEmpty()) {
                "Unbound dynamic slots in window '${definition.name}': ${unbound.sorted()}"
            }
            val unboundSprites = dynamicSpriteNames - spriteRenders.keys
            check(unboundSprites.isEmpty()) {
                "Unbound dynamic sprite slots in window '${definition.name}': " +
                    unboundSprites.sorted()
            }
            val unhandled =
                entry.buttons
                    .filter { (name, b) -> b.action && name !in buttonHandlers && b.default == null }
                    .keys
            check(unhandled.isEmpty()) {
                "Buttons in window '${definition.name}' have neither a handler nor a default: " +
                    "${unhandled.sorted()}"
            }
            val unboundItems = entry.items.keys - itemRenders.keys
            check(unboundItems.isEmpty()) {
                "Unbound dynamic items in window '${definition.name}': ${unboundItems.sorted()}"
            }
            val unboundCollectionItems = entry.collections.keys - collectionItemRenders.keys
            check(unboundCollectionItems.isEmpty()) {
                "Unbound collection items in window '${definition.name}': " +
                    unboundCollectionItems.sorted()
            }
            val unhandledCollections =
                entry.collections.filter { (name, c) -> c.action && name !in collectionHandlers }.keys
            check(unhandledCollections.isEmpty()) {
                "Collections in window '${definition.name}' have no handler: " +
                    unhandledCollections.sorted()
            }
            val unboundInputs = entry.inputs.keys - inputHandlers.keys
            check(unboundInputs.isEmpty()) {
                "Unbound anvil inputs in window '${definition.name}': ${unboundInputs.sorted()}"
            }
            // Map typed backing slots to controls for routing.
            for ((name, button) in entry.buttons) {
                for (slot in button.slots) {
                    slotRoutes[slot.toApi()] = Route.Button(name)
                }
            }
            for ((name, collection) in entry.collections) {
                if (!collection.action) continue
                for ((index, slot) in collection.slots.withIndex()) {
                    slotRoutes[slot.toApi()] = Route.Collection(name, index)
                }
            }
        }

        /** Seeds manifest/default inventory items after the inventory exists. */
        private fun seedInventoryItems() {
            for ((_, slotRect) in entry.slotRects) {
                applySlotRect(slotRect)
            }
            for ((name, button) in entry.buttons) {
                val item =
                    when {
                        name in buttonStateValues -> {
                            itemForState(name, buttonStateValues.getValue(name))
                        }

                        name in buttonItemRenders -> {
                            renderButtonItem(name)
                        }

                        else -> {
                            defaultButtonItem(button)
                        }
                    } ?: ItemStack.AIR
                applyButtonItem(button, item)
            }
            for ((name, item) in entry.items) {
                applyItem(item, renderItem(name) ?: ItemStack.AIR)
            }
            for ((name, collection) in entry.collections) {
                seedCollection(name, collection)
            }
            for ((_, input) in entry.inputs) {
                applyInput(input, input.initial)
            }
        }

        private fun handleInput(value: String) {
            if (closed) return
            val (name, handler) = inputHandlers.entries.singleOrNull() ?: return
            val input = entry.inputs.getValue(name)
            // Reactive text changes update the inventory title, which reopens the vanilla menu. Keep
            // slot zero's custom name in sync first so Minecraft restores the current edit-box value
            // instead of replacing it with the original seed text after every keystroke.
            applyInput(input, value)
            handler(value)
        }

        private fun applyInput(
            input: AnvilInputEntry,
            value: String,
        ) {
            val model = input.itemModel ?: "${definition.manifest.namespace}:gui/hitbox"
            handle.setItem(input.slot.toApi(), WindowItems.anvilInput(value, model))
        }

        /** Renders static labels and every dynamic slot once to seed title segments. */
        private fun seedSegments() {
            for ((name, button) in entry.buttons) {
                val state =
                    when {
                        name in buttonStateRenders -> renderButtonState(name)
                        "default" in button.states -> "default"
                        else -> null
                    }
                if (state != null) {
                    buttonStateValues[name] = state
                    slotSegments[BUTTON_VISUAL_PREFIX + name] = renderButtonVisual(name, state)
                }
            }
            for ((name, slot) in entry.spriteSlots) {
                val sprite = slot.sprite ?: continue
                slotSegments[SPRITE_RENDER_PREFIX + name] =
                    composer.renderSprite("window/${definition.name}/sprite/$name", slot, sprite)
                        ?: emptySegment(
                            "window/${definition.name}/sprite/$name",
                            RenderLayerKind.SPRITE_SLOT,
                            slot.x,
                            slot.y,
                            slot.font,
                        )
            }
            for (name in dynamicSpriteNames) {
                slotSegments[SPRITE_RENDER_PREFIX + name] = renderSpriteSegment(name)
            }
            for ((name, slot) in entry.slots) {
                val text = slot.text
                if (text != null) {
                    slotSegments[name] =
                        composer.renderSlot(
                            "window/${definition.name}/slot/$name",
                            slot,
                            Component.text(text),
                        ) ?: continue
                } else {
                    slotSegments[name] = renderSegment(name)
                }
            }
        }

        private fun renderButtonState(name: String): String {
            val render = buttonStateRenders.getValue(name)
            return reactivity.withRendering(BUTTON_STATE_PREFIX + name) { render() }
        }

        private fun renderButtonVisual(
            name: String,
            stateName: String,
        ): RenderedSegment {
            val button = entry.buttons.getValue(name)
            val state = requireButtonState(name, stateName)
            val font = button.spriteFont
            val sprite = state.sprite
            if (font == null || sprite == null) {
                return emptySegment(
                    "window/${definition.name}/button/$name",
                    RenderLayerKind.SPRITE_SLOT,
                    button.x,
                    button.y,
                    font ?: definition.manifest.font,
                )
            }
            val slot =
                SpriteSlotEntry(
                    x = button.x,
                    y = button.y,
                    width = button.width,
                    height = button.height,
                    align = Align.LEFT,
                    font = font,
                )
            return composer.renderSprite("window/${definition.name}/button/$name", slot, sprite)
                ?: error("Button '$name' state '$stateName' has an empty sprite")
        }

        /** Renders one slot's segment under dependency capture. */
        private fun renderSegment(name: String): RenderedSegment {
            val render = slotRenders.getValue(name)
            val content = reactivity.withRendering(name) { render() }
            val slot = entry.slots.getValue(name)
            return composer.renderSlot("window/${definition.name}/slot/$name", slot, content)
                ?: emptySegment(
                    "window/${definition.name}/slot/$name",
                    RenderLayerKind.TEXT_SLOT,
                    slot.x,
                    slot.y,
                    slot.font,
                )
        }

        /** Renders one runtime sprite segment under dependency capture. */
        private fun renderSpriteSegment(name: String): RenderedSegment {
            val render = spriteRenders.getValue(name)
            val sprite = reactivity.withRendering(SPRITE_RENDER_PREFIX + name) { render() }
            val slot = entry.spriteSlots.getValue(name)
            return composer.renderSprite("window/${definition.name}/sprite/$name", slot, sprite)
                ?: emptySegment(
                    "window/${definition.name}/sprite/$name",
                    RenderLayerKind.SPRITE_SLOT,
                    slot.x,
                    slot.y,
                    slot.font,
                )
        }

        /** Recomputes the dirty slots, rebuilds the title, and sends it. */
        private fun flush(dirty: Set<String>) {
            if (closed) return
            for (name in dirty) {
                if (name in slotRenders) {
                    slotSegments[name] = renderSegment(name)
                }
                if (name.startsWith(SPRITE_RENDER_PREFIX)) {
                    val spriteName = name.removePrefix(SPRITE_RENDER_PREFIX)
                    if (spriteName in spriteRenders) {
                        slotSegments[name] = renderSpriteSegment(spriteName)
                    }
                }
                if (name.startsWith(BUTTON_RENDER_PREFIX)) {
                    val buttonName = name.removePrefix(BUTTON_RENDER_PREFIX)
                    val button = entry.buttons[buttonName] ?: continue
                    val item = renderButtonItem(buttonName) ?: ItemStack.AIR
                    applyButtonItem(button, item)
                }
                if (name.startsWith(BUTTON_STATE_PREFIX)) {
                    val buttonName = name.removePrefix(BUTTON_STATE_PREFIX)
                    val button = entry.buttons[buttonName] ?: continue
                    val state = renderButtonState(buttonName)
                    buttonStateValues[buttonName] = state
                    applyButtonItem(button, itemForState(buttonName, state))
                    slotSegments[BUTTON_VISUAL_PREFIX + buttonName] =
                        renderButtonVisual(buttonName, state)
                }
                if (name.startsWith(ITEM_RENDER_PREFIX)) {
                    val itemName = name.removePrefix(ITEM_RENDER_PREFIX)
                    val item = entry.items[itemName] ?: continue
                    applyItem(item, renderItem(itemName) ?: ItemStack.AIR)
                }
                if (name.startsWith(COLLECTION_RENDER_PREFIX)) {
                    val key = name.removePrefix(COLLECTION_RENDER_PREFIX)
                    val separator = key.lastIndexOf(':')
                    if (separator <= 0) continue
                    val collectionName = key.substring(0, separator)
                    val index = key.substring(separator + 1).toIntOrNull() ?: continue
                    val collection = entry.collections[collectionName] ?: continue
                    applyCollectionCell(collection, collectionName, index)
                }
            }
            val render = composer.compose(definition.name, slotSegments)
            handle.setTitle(render.component)
            observe(render.layers, RenderFrameReason.REACTIVE_UPDATE)
        }

        private fun observe(
            layers: List<RenderLayerTrace>,
            reason: RenderFrameReason,
        ) {
            val frame =
                RenderFrame(
                    frameId = nextFrameId.getAndIncrement(),
                    reason = reason,
                    correlation =
                        RenderCorrelation(
                            surfaceKind = RenderSurfaceKind.WINDOW,
                            semanticId = definition.name,
                            renderSessionId = renderSessionId,
                            containerId = handle.containerId,
                        ),
                    cursorConvention = RenderCursorConvention.INDEPENDENT_NET_ZERO_SEGMENTS,
                    cursorStart = entry.surface.titleOrigin[0],
                    cursorEnd = entry.surface.titleOrigin[0],
                    netCursorDelta = 0,
                    layers = layers,
                )
            try {
                diagnosticsObserver.observe(player, frame)
            } catch (error: RuntimeException) {
                LOGGER.warn("Window render diagnostics observer failed", error)
            }
        }

        private fun emptySegment(
            semanticId: String,
            kind: RenderLayerKind,
            x: Int,
            y: Int,
            font: String,
        ): RenderedSegment =
            RenderedSegment(
                Component.empty(),
                RenderLayerTrace(
                    semanticId = semanticId,
                    kind = kind,
                    content = "",
                    font = font,
                    style = RenderStyleTrace(color = "#ffffff", shadow = false),
                    expectedBounds = RenderBounds(x, y, 0, 0),
                    cursorStart = entry.surface.titleOrigin[0],
                    contentCursorStart = x,
                    contentCursorEnd = x,
                    cursorEnd = entry.surface.titleOrigin[0],
                    advance = 0,
                    visualWidth = 0,
                    netCursorDelta = 0,
                ),
            )

        private fun renderButtonItem(name: String): ItemStack? {
            val render = buttonItemRenders.getValue(name)
            return reactivity.withRendering(BUTTON_RENDER_PREFIX + name) { render() }
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
            for (slot in button.filledSlots) {
                handle.setItem(slot.toApi(), item)
            }
        }

        private fun renderItem(name: String): ItemStack? {
            val render = itemRenders.getValue(name)
            return reactivity.withRendering(ITEM_RENDER_PREFIX + name) { render() }
        }

        private fun applyItem(
            item: ItemEntry,
            stack: ItemStack,
        ) {
            for (slot in item.slots) {
                handle.setItem(slot.toApi(), stack)
            }
        }

        private fun applySlotRect(slotRect: SlotRectEntry) {
            for (slot in slotRect.slots) {
                handle.setItem(slot.toApi(), ItemStack.AIR)
            }
        }

        private fun seedCollection(
            name: String,
            collection: CollectionEntry,
        ) {
            for (index in collection.slots.indices) {
                applyCollectionCell(collection, name, index)
            }
        }

        private fun applyCollectionCell(
            collection: CollectionEntry,
            name: String,
            index: Int,
        ) {
            val slot = collection.slots.getOrNull(index) ?: return
            val item = renderCollectionItem(name, index) ?: ItemStack.AIR
            handle.setItem(slot.toApi(), item)
        }

        private fun renderCollectionItem(
            name: String,
            index: Int,
        ): ItemStack? {
            val render = collectionItemRenders.getValue(name)
            return reactivity.withRendering(collectionKey(name, index)) { render(index) }
        }

        private fun defaultButtonItem(button: ButtonEntry): ItemStack? {
            val defaultState = button.states["default"]
            if (defaultState != null) return itemForState(button, defaultState)
            val tooltip = button.tooltip?.toApi() ?: return null
            return hitbox(tooltip)
        }

        internal fun setButtonItem(
            name: String,
            item: ItemStack?,
        ) {
            val button =
                entry.buttons[name]
                    ?: throw IllegalArgumentException(
                        "Unknown button or hotspot '$name' in window '${definition.name}'",
                    )
            applyButtonItem(button, item ?: ItemStack.AIR)
        }

        internal fun setItem(
            name: String,
            stack: ItemStack?,
        ) {
            val item =
                entry.items[name]
                    ?: throw IllegalArgumentException(
                        "Unknown item '$name' in window '${definition.name}'; " +
                            "known items: ${entry.items.keys.sorted()}",
                    )
            applyItem(item, stack ?: ItemStack.AIR)
        }

        internal fun setButtonState(
            name: String,
            state: String,
        ) {
            val button =
                entry.buttons[name]
                    ?: throw IllegalArgumentException(
                        "Unknown button or hotspot '$name' in window '${definition.name}'",
                    )
            requireButtonState(name, state)
            buttonStateValues[name] = state
            applyButtonItem(button, itemForState(name, state))
            slotSegments[BUTTON_VISUAL_PREFIX + name] = renderButtonVisual(name, state)
            if (!closed) {
                val render = composer.compose(definition.name, slotSegments)
                handle.setTitle(render.component)
                observe(render.layers, RenderFrameReason.REACTIVE_UPDATE)
            }
        }

        internal fun setTooltip(
            name: String,
            tooltip: ButtonTooltip?,
        ) {
            setButtonItem(name, tooltip?.let { hitbox(it) })
        }

        private fun itemForState(
            name: String,
            state: String,
        ): ItemStack {
            val button = entry.buttons.getValue(name)
            val value = requireButtonState(name, state)
            return itemForState(button, value)
        }

        private fun requireButtonState(
            name: String,
            state: String,
        ): ButtonState {
            val button =
                entry.buttons[name]
                    ?: throw IllegalArgumentException(
                        "Unknown button or hotspot '$name' in window '${definition.name}'",
                    )
            return button.states[state]
                ?: throw IllegalArgumentException(
                    "Unknown state '$state' for button '$name' in window '${definition.name}'; " +
                        "known states: ${button.states.keys.sorted()}",
                )
        }

        private fun itemForState(
            button: ButtonEntry,
            state: ButtonState,
        ): ItemStack {
            val tooltip = state.tooltip?.toApi() ?: button.tooltip?.toApi()
            val model = state.itemModel ?: "${definition.manifest.namespace}:gui/hitbox"
            return hitbox(tooltip ?: ButtonTooltip(Component.empty()), model)
        }

        private fun hitbox(
            tooltip: ButtonTooltip,
            itemModel: String = "${definition.manifest.namespace}:gui/hitbox",
        ): ItemStack = WindowItems.hitbox(tooltip, itemModel)

        private fun handleClick(info: ClickInfo) {
            if (closed) return
            when (val route = slotRoutes[info.slot] ?: return) {
                is Route.Button -> {
                    val handler = buttonHandlers[route.name]
                    if (handler != null) {
                        handler(Click(player, info.slot, info.shift, info.right))
                        return
                    }
                    // No user handler: apply the manifest default.
                    val button = entry.buttons.getValue(route.name)
                    if (button.default == ButtonDefault.CLOSE) {
                        close()
                    }
                }

                is Route.Collection -> {
                    val handler = collectionHandlers[route.name] ?: return
                    handler(IndexedClick(player, info.slot, route.index, info.shift, info.right))
                }
            }
        }

        private fun handleClientClose() {
            if (closed) return
            closed = true
            view.invokeOnClose()
            handle.teardownListeners()
        }

        /** Closes this window: invokes [WindowView.onClose], closes the inventory, and tears down. */
        public fun close() {
            if (closed || closing) return
            closing = true
            closed = true
            view.invokeOnClose()
            handle.close()
        }

        /**
         * Marks all dynamic slots dirty and schedules a single re-render. Backs `WindowView.refresh`.
         */
        internal fun refreshAll() {
            reactivity.markAllDirty(refreshableKeys())
        }

        private fun refreshableKeys(): Set<String> =
            buildSet {
                addAll(dynamicSlotNames)
                addAll(spriteRenders.keys.map { SPRITE_RENDER_PREFIX + it })
                addAll(buttonItemRenders.keys.map { BUTTON_RENDER_PREFIX + it })
                addAll(buttonStateRenders.keys.map { BUTTON_STATE_PREFIX + it })
                addAll(itemRenders.keys.map { ITEM_RENDER_PREFIX + it })
                for ((name, collection) in entry.collections) {
                    if (name !in collectionItemRenders) continue
                    for (index in collection.slots.indices) {
                        add(collectionKey(name, index))
                    }
                }
            }

        private fun dev.oglass.window.manifest.ButtonTooltip.toApi(): ButtonTooltip =
            ButtonTooltip(
                parseTooltipLine(title, NamedTextColor.WHITE),
                lines.map { parseTooltipLine(it, NamedTextColor.GRAY) },
            )

        private fun parseTooltipLine(
            template: String,
            defaultColor: TextColor,
        ): Component =
            Component
                .empty()
                .color(defaultColor)
                .decoration(TextDecoration.ITALIC, false)
                .append(MINI_MESSAGE.deserialize(template))

        private fun SlotRefEntry.toApi(): SlotRef =
            SlotRef(
                when (area) {
                    SlotAreaEntry.CONTAINER -> SlotArea.CONTAINER
                    SlotAreaEntry.PLAYER -> SlotArea.PLAYER
                },
                index,
            )

        private sealed interface Route {
            data class Button(
                val name: String,
            ) : Route

            data class Collection(
                val name: String,
                val index: Int,
            ) : Route
        }

        private companion object {
            val LOGGER = LoggerFactory.getLogger(WindowSession::class.java)
            const val BUTTON_RENDER_PREFIX = "button:"
            const val BUTTON_STATE_PREFIX = "button-state:"
            const val BUTTON_VISUAL_PREFIX = "button-visual:"
            const val SPRITE_RENDER_PREFIX = "sprite:"
            const val ITEM_RENDER_PREFIX = "item:"
            const val COLLECTION_RENDER_PREFIX = "collection:"
            val MINI_MESSAGE: MiniMessage = MiniMessage.miniMessage()

            fun collectionKey(
                name: String,
                index: Int,
            ): String = "$COLLECTION_RENDER_PREFIX$name:$index"
        }
    }
