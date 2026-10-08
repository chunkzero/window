package com.chunkzero.window

import com.chunkzero.window.diagnostics.RenderCorrelation
import com.chunkzero.window.diagnostics.RenderCursorConvention
import com.chunkzero.window.diagnostics.RenderFrameReason
import com.chunkzero.window.diagnostics.RenderSurfaceKind
import com.chunkzero.window.host.ContainerListener
import com.chunkzero.window.host.OpenContainer
import com.chunkzero.window.host.WindowHost
import com.chunkzero.window.internal.ComposedRender
import com.chunkzero.window.internal.FrameCursor
import com.chunkzero.window.internal.Reactivity
import com.chunkzero.window.internal.RenderKey
import com.chunkzero.window.internal.RenderScheduler
import com.chunkzero.window.internal.RuntimeAction
import com.chunkzero.window.internal.SessionFrames
import com.chunkzero.window.internal.SlotRoutes
import com.chunkzero.window.internal.SlotWrites
import com.chunkzero.window.internal.Switches
import com.chunkzero.window.internal.WindowBindings
import com.chunkzero.window.internal.WindowFrame
import com.chunkzero.window.internal.WindowRenderer
import com.chunkzero.window.internal.hasStaticTitle
import com.chunkzero.window.internal.requireEntry
import com.chunkzero.window.internal.titleSlots
import net.kyori.adventure.text.Component
import org.slf4j.LoggerFactory

/**
 * A live window opened by [WindowView.open].
 *
 * Routes clicks to button handlers and runtime actions, drives reactive re-renders, and manages the
 * window lifecycle.
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

/**
 * A window session over a container opened through [host]: turns container input into view calls, and delivers what
 * its [WindowRenderer] renders.
 */
internal open class ContainerWindowSession<I : Any>(
    protected val definition: WindowDefinition,
    override val view: WindowView<I>,
    private val host: WindowHost<I>,
) : WindowSession() {
    protected val entry = definition.entry
    protected val scheduler = RenderScheduler(host::scheduleNextTick)
    private val reactivity = Reactivity(scheduler) { dirty -> flush(dirty) }

    protected val bindings =
        WindowBindings(
            definition,
            host::item,
            definition.titleSlots(reactivity),
            Switches("window", definition.name, entry.switches, reactivity),
        )
    protected val renderer = WindowRenderer(definition, bindings, reactivity, host::item)

    /** The open container; set when the window opens. */
    protected lateinit var container: OpenContainer<I>
        private set
    private val routes =
        SlotRoutes(entry, bindings) { action ->
            when (action) {
                RuntimeAction.CLOSE -> close()
            }
        }
    private var sentTitle: Component? = null

    private val frames =
        SessionFrames(
            { observer, frame -> observer.observe(host, frame) },
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
    protected var closed = false
        private set

    internal fun open() {
        view.attach(this, reactivity)
        view.invokeBind(bindings)
        bindings.validate()
        bindInput()

        val title = renderer.seedTitle()
        container =
            try {
                host.open(definition.kind, title.component, listener)
            } catch (failure: Throwable) {
                closed = true
                throw failure
            }
        send { deliver(renderer.seedItems()) }
        sentTitle = title.component
        frames.observe(title, RenderFrameReason.OPEN)
        view.invokeOnOpen()
    }

    private fun flush(dirty: Set<RenderKey>) {
        if (closed) return
        val frame =
            try {
                renderer.render(dirty)
            } catch (failure: Throwable) {
                deliver(renderer.drainWrites())
                throw failure
            }
        deliver(frame)
    }

    private fun deliver(frame: WindowFrame<I>) {
        deliver(frame.writes)
        sendTitle(frame.title)
    }

    protected fun deliver(writes: SlotWrites<I>) {
        for ((slot, item) in writes.items) container.setItem(slot, item)
        for ((slot, item) in writes.staged) container.stageItem(slot, item)
    }

    /** Sends [title] when it changed, or when [reopen] asks to restore an anvil's edit box. */
    protected fun sendTitle(
        title: ComposedRender,
        reopen: Boolean = false,
    ) {
        if (closed) return
        if (!reopen && title.component == sentTitle) return
        if (!send { container.setTitle(title.component) }) return
        sentTitle = title.component
        frames.observe(title, RenderFrameReason.REACTIVE_UPDATE)
    }

    /**
     * Runs [action], which seeds the opened container or changes its title; false when it was
     * deferred.
     */
    protected open fun send(action: () -> Unit): Boolean {
        action()
        return true
    }

    /** Binds the window's anvil input, after the view's bindings are validated. */
    protected open fun bindInput() {}

    /** Handles the client's anvil edit-box [value]. */
    protected open fun onInput(value: String) {}

    /** Handles the client's answer to ping [id]. */
    protected open fun onPong(id: Int) {}

    /** Runs before a client click or close is handled. */
    protected open fun onClientPacket() {}

    /** Replaces the text of anvil input [name]; backs `WindowView.input`. */
    internal open fun setInput(
        name: String,
        value: String,
    ) {
        definition.requireEntry(entry.inputs, name, "anvil input", known = "inputs")
    }

    internal fun setButtonItem(
        name: String,
        item: I?,
    ) = deliver(renderer.setButtonItem(name, item))

    internal fun setItem(
        name: String,
        item: I?,
    ) = deliver(renderer.setItem(name, item))

    internal fun setButtonState(
        name: String,
        state: String,
    ) {
        deliver(renderer.setButtonState(name, state) ?: return)
    }

    internal fun setTooltip(
        name: String,
        tooltip: ButtonTooltip?,
    ) = deliver(renderer.setTooltip(name, tooltip))

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
