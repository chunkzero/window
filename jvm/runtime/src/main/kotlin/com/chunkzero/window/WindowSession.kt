package com.chunkzero.window

import com.chunkzero.window.diagnostics.RenderCorrelation
import com.chunkzero.window.diagnostics.RenderCursorConvention
import com.chunkzero.window.diagnostics.RenderFrameReason
import com.chunkzero.window.diagnostics.RenderSurfaceKind
import com.chunkzero.window.host.ContainerListener
import com.chunkzero.window.host.OpenContainer
import com.chunkzero.window.host.WindowHost
import com.chunkzero.window.internal.FrameCursor
import com.chunkzero.window.internal.Reactivity
import com.chunkzero.window.internal.RenderKey
import com.chunkzero.window.internal.RenderScheduler
import com.chunkzero.window.internal.SessionFrames
import com.chunkzero.window.internal.SlotRoutes
import com.chunkzero.window.internal.Switches
import com.chunkzero.window.internal.WindowBindings
import com.chunkzero.window.internal.WindowInventoryWriter
import com.chunkzero.window.internal.WindowTitle
import com.chunkzero.window.internal.hasStaticTitle
import com.chunkzero.window.internal.requireEntry
import com.chunkzero.window.internal.titleSlots
import com.chunkzero.window.internal.tooltipHitbox
import net.kyori.adventure.text.Component
import org.slf4j.LoggerFactory

/**
 * A live window opened by [WindowView.open].
 *
 * Routes clicks to button handlers, drives reactive re-renders, and manages the window lifecycle.
 */
public sealed class WindowSession {
    /** The view driving this session. */
    public abstract val view: WindowView<*>

    /** Closes this window: invokes [WindowView.onClose] and closes the container. */
    public abstract fun close()
}

/** Creates the session for [definition]'s window, with an anvil session for an anvil input window. */
internal fun <I : Any> windowSession(
    definition: WindowDefinition,
    view: WindowView<I>,
    host: WindowHost<I>,
): ContainerWindowSession<I> =
    when {
        definition.entry.inputs.isEmpty() -> ContainerWindowSession(definition, view, host)
        definition.entry.hasStaticTitle -> AnvilWindowSession(definition, view, host)
        else -> ReopeningAnvilWindowSession(definition, view, host)
    }

/** A window session over a container opened through [host]. */
internal open class ContainerWindowSession<I : Any>(
    internal val definition: WindowDefinition,
    override val view: WindowView<I>,
    private val host: WindowHost<I>,
) : WindowSession() {
    internal val entry = definition.entry
    internal val scheduler = RenderScheduler(host::scheduleNextTick)
    private val reactivity = Reactivity(scheduler) { dirty -> flush(dirty) }

    internal val bindings =
        WindowBindings(
            definition,
            host,
            definition.titleSlots(reactivity),
            Switches("window", definition.name, entry.switches, reactivity),
        )
    private val title = WindowTitle(definition, bindings, reactivity)

    /** The open container; set when the window opens. */
    internal lateinit var container: OpenContainer<I>
        private set
    internal val writer = WindowInventoryWriter(definition, bindings, host, { container }, reactivity)
    private val routes = SlotRoutes(entry, bindings, ::close)
    private var sentTitle: Component? = null

    private val frames =
        SessionFrames(
            host,
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
            )
        }

    private val listener =
        object : ContainerListener {
            override fun onClick(
                slot: SlotRef,
                shift: Boolean,
                right: Boolean,
            ) = handleClick(Click(slot, shift, right))

            override fun onClose() = handleClientClose()

            override fun onAnvilInput(text: String) {
                if (!closed) onInput(text)
            }

            override fun onPong(id: Int) {
                if (!closed) this@ContainerWindowSession.onPong(id)
            }
        }

    private var closing = false
    internal var closed = false
        private set

    internal fun open() {
        view.attach(this, reactivity)
        view.invokeBind(bindings)
        bindings.validate()
        bindInput()
        seedTitle()

        val render = title.compose()
        container = host.open(definition.kind, render.component, listener)
        send { writer.seed() }
        sentTitle = render.component
        frames.observe(render, RenderFrameReason.OPEN)
        view.invokeOnOpen()
    }

    /**
     * Renders switch case art first, then button state sprites, so both draw beneath sprite and
     * text slots. Segments updated later keep this position, since title segments compose in
     * insertion order.
     */
    private fun seedTitle() {
        for (name in entry.switches.keys) title.updateSwitch(name)
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
                is RenderKey.CollectionSelection -> title.updateCollectionSelection(key.name)
                is RenderKey.Switch -> title.updateSwitch(key.name)
            }
        }
        sendTitle()
    }

    /** Sends the title when it changed, or when [reopen] asks to restore an anvil's edit box. */
    internal fun sendTitle(reopen: Boolean = false) {
        if (closed) return
        val render = title.compose()
        if (!reopen && render.component == sentTitle) return
        if (!send { container.setTitle(render.component) }) return
        sentTitle = render.component
        frames.observe(render, RenderFrameReason.REACTIVE_UPDATE)
    }

    /**
     * Runs [action], which seeds the opened container or changes its title; false when it was
     * deferred.
     */
    internal open fun send(action: () -> Unit): Boolean {
        action()
        return true
    }

    /** Binds the window's anvil input, after the view's bindings are validated. */
    internal open fun bindInput() {}

    /** Handles the client's anvil edit-box [value]. */
    internal open fun onInput(value: String) {}

    /** Handles the client's answer to ping [id]. */
    internal open fun onPong(id: Int) {}

    /** Runs before a client click or close is handled. */
    internal open fun onClientPacket() {}

    /** Replaces the text of anvil input [name]; backs `WindowView.input`. */
    internal open fun setInput(
        name: String,
        value: String,
    ) {
        definition.requireEntry(entry.inputs, name, "anvil input", known = "inputs")
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
        item: I?,
    ) {
        writer.setButtonItem(name, item)
    }

    internal fun setItem(
        name: String,
        item: I?,
    ) {
        writer.setItem(name, item)
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
        setButtonItem(name, tooltip?.let { host.item(definition.tooltipHitbox(it)) })
    }

    private fun handleClick(click: Click) {
        if (closed) return
        onClientPacket()
        if (closed) return
        routes.dispatch(click)
    }

    private fun handleClientClose() {
        if (closed) return
        onClientPacket()
        if (closed) return
        closed = true
        view.invokeOnClose()
    }

    override fun close() {
        if (closed || closing) return
        closing = true
        closed = true
        view.invokeOnClose()
        if (::container.isInitialized) container.close()
    }

    /**
     * Marks all bound render targets dirty and schedules a single re-render. Backs
     * `WindowView.refresh`.
     */
    internal fun refreshAll() {
        reactivity.markAllDirty(bindings.renderKeys())
    }

    private companion object {
        val LOGGER = LoggerFactory.getLogger(ContainerWindowSession::class.java)
    }
}
