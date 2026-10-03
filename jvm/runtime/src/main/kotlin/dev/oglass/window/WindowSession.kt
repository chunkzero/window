package dev.oglass.window

import dev.oglass.window.diagnostics.RenderCorrelation
import dev.oglass.window.diagnostics.RenderCursorConvention
import dev.oglass.window.diagnostics.RenderFrameReason
import dev.oglass.window.diagnostics.RenderSurfaceKind
import dev.oglass.window.internal.AnvilReopenGate
import dev.oglass.window.internal.ClickInfo
import dev.oglass.window.internal.FrameCursor
import dev.oglass.window.internal.InventoryHandle
import dev.oglass.window.internal.LiveInventoryHandle
import dev.oglass.window.internal.Reactivity
import dev.oglass.window.internal.RenderKey
import dev.oglass.window.internal.RenderScheduler
import dev.oglass.window.internal.SessionFrames
import dev.oglass.window.internal.SlotRoutes
import dev.oglass.window.internal.WindowBindings
import dev.oglass.window.internal.WindowInventoryWriter
import dev.oglass.window.internal.WindowTitle
import dev.oglass.window.internal.hitboxModel
import dev.oglass.window.internal.titleSlots
import net.minestom.server.entity.Player
import net.minestom.server.inventory.Inventory
import net.minestom.server.item.ItemStack
import org.slf4j.LoggerFactory

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
        diagnosticsObserver: RenderDiagnosticsObserver = RenderDiagnosticsObserver.NONE,
    ) {
        private val entry = definition.entry
        private val reactivity = Reactivity(scheduler) { dirty -> flush(dirty) }

        private val bindings = WindowBindings(definition, definition.titleSlots(reactivity))
        private val title = WindowTitle(definition, bindings, reactivity)
        private val writer = WindowInventoryWriter(definition, bindings, handle, reactivity)
        private val routes = SlotRoutes(entry, bindings, player, ::close)
        private val reopens =
            entry.inputs.values
                .singleOrNull()
                ?.let { AnvilReopenGate(scheduler, it.initial, handle::ping, ::applyInput, ::sendTitle) }

        private val frames =
            SessionFrames(
                player,
                diagnosticsObserver,
                entry.surface.titleOrigin[0].let {
                    FrameCursor(RenderCursorConvention.INDEPENDENT_NET_ZERO_SEGMENTS, it, it)
                },
                LOGGER,
                "Window render diagnostics observer failed",
            ) { renderSessionId ->
                RenderCorrelation(
                    surfaceKind = RenderSurfaceKind.WINDOW,
                    semanticId = definition.name,
                    renderSessionId = renderSessionId,
                    containerId = handle.containerId,
                )
            }

        private var opened = false
        private var closing = false
        private var closed = false

        /** The Minestom inventory backing this window. */
        public val inventory: Inventory
            get() =
                (handle as? LiveInventoryHandle)?.inventory
                    ?: error("This session is not backed by a live inventory")

        internal fun open() {
            check(!opened) { "Session already opened" }
            view.attach(player, this, reactivity)
            view.invokeBind(bindings)
            bindings.validate()
            seedTitle()

            val render = title.compose()
            handle.bundle {
                handle.open(render.component)
                handle.registerListeners(::handleClick, ::handleClientClose, ::handleInput, ::handlePong)
                writer.seed()
                reopens?.sent()
            }
            frames.observe(render, RenderFrameReason.OPEN)
            view.invokeOnOpen()
            opened = true
        }

        /**
         * Renders button state sprites first so they draw beneath sprite and text slots. Buttons whose
         * state is set later keep this position, since title segments compose in insertion order.
         */
        private fun seedTitle() {
            for (name in entry.buttons.keys) {
                val state = writer.initialButtonState(name)
                if (state != null) title.setButtonVisual(name, state) else title.reserveButtonVisual(name)
            }
            title.seedContent()
        }

        /** Re-renders the dirty keys, then rebuilds and sends the title. */
        private fun flush(dirty: Set<RenderKey>) {
            if (closed) return
            for (key in dirty) {
                when (key) {
                    is RenderKey.Slot -> title.updateSlot(key.name)
                    is RenderKey.Sprite -> title.updateSprite(key.name)
                    is RenderKey.ButtonItem -> writer.writeButtonItem(key.name)
                    is RenderKey.ButtonState -> applyButtonState(key.name, writer.renderButtonState(key.name))
                    is RenderKey.Item -> writer.writeItem(key.name)
                    is RenderKey.CollectionCell -> writer.writeCollectionCell(key.name, key.index)
                }
            }
            sendTitle()
        }

        private fun sendTitle() {
            if (closed || reopens?.canReopen() == false) return
            val render = title.compose()
            handle.bundle {
                handle.setTitle(render.component)
                reopens?.sent()
            }
            frames.observe(render, RenderFrameReason.REACTIVE_UPDATE)
        }

        private fun applyButtonState(
            name: String,
            state: String,
        ) {
            writer.applyButtonState(name, state)
            title.setButtonVisual(name, state)
        }

        internal fun setButtonItem(
            name: String,
            item: ItemStack?,
        ) {
            writer.setButtonItem(name, item)
        }

        internal fun setItem(
            name: String,
            stack: ItemStack?,
        ) {
            writer.setItem(name, stack)
        }

        internal fun setButtonState(
            name: String,
            state: String,
        ) {
            if (writer.buttonState(name) == state) return
            applyButtonState(name, state)
            if (!closed) sendTitle()
        }

        internal fun setTooltip(
            name: String,
            tooltip: ButtonTooltip?,
        ) {
            setButtonItem(name, tooltip?.let { WindowItems.hitbox(it, definition.hitboxModel) })
        }

        private fun handleClick(info: ClickInfo) {
            if (closed) return
            reopens?.release()
            routes.dispatch(info)
        }

        private fun handleInput(value: String) {
            if (closed) return
            reopens?.input(value)
        }

        private fun handlePong(id: Int) {
            if (closed) return
            reopens?.pong(id)
        }

        private fun applyInput(value: String) {
            if (closed) return
            val (name, handler) = bindings.inputHandlers.entries.singleOrNull() ?: return
            // Title changes reopen the vanilla menu, which resets the edit box to the seed's name.
            // Stage the typed value as that name so the next reopen restores it.
            writer.stageInput(entry.inputs.getValue(name), value)
            handler(value)
        }

        private fun handleClientClose() {
            if (closed) return
            reopens?.release()
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
         * Marks all bound render targets dirty and schedules a single re-render. Backs
         * `WindowView.refresh`.
         */
        internal fun refreshAll() {
            reactivity.markAllDirty(bindings.renderKeys())
        }

        private companion object {
            val LOGGER = LoggerFactory.getLogger(WindowSession::class.java)
        }
    }
