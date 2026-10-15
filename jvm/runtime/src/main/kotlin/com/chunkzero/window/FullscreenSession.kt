package com.chunkzero.window

import com.chunkzero.window.host.FullscreenHost
import com.chunkzero.window.host.FullscreenListener
import com.chunkzero.window.host.OpenFullscreen
import com.chunkzero.window.internal.CanvasPoint
import com.chunkzero.window.internal.FullscreenMapping
import com.chunkzero.window.internal.HudLayers
import com.chunkzero.window.internal.Reactivity
import com.chunkzero.window.internal.RenderKey
import com.chunkzero.window.internal.RenderScheduler
import com.chunkzero.window.internal.RuntimeAction
import net.kyori.adventure.text.Component

/** A live full-screen window opened by [FullscreenView.open]. */
public sealed class FullscreenSession {
    /** The view driving this session. */
    public abstract val view: FullscreenView

    /** Closes this window: invokes [FullscreenView.onClose] and restores the player. */
    public abstract fun close()
}

/**
 * A full-screen window shown through [host]: maps the player's rotation to the canvas cursor, hit-tests clicks, draws
 * the hovered control's outline, and re-sends the runs when reactive state or the hovered control changes.
 *
 * A player turned more than a degree past a canvas edge is turned back to it, at most once per round trip: rotations
 * the client sent before applying a correction still carry the overshoot.
 */
internal class FullscreenWindowSession(
    private val definition: FullscreenDefinition,
    override val view: FullscreenView,
    private val host: FullscreenHost,
    private val nanoTime: () -> Long = System::nanoTime,
) : FullscreenSession() {
    private val entry = definition.entry
    private val mapping = FullscreenMapping(definition.canvas)
    private val reactivity = Reactivity(RenderScheduler(host::scheduleNextTick)) { dirty -> flush(dirty) }
    private val layers = HudLayers("fullscreen", definition.name, definition.layers, definition.composer, reactivity)
    private val bindings = Bindings()

    private lateinit var screen: OpenFullscreen
    private lateinit var layout: Component
    private var sent: Component? = null
    private var hovered: String? = null
    private var yaw = 0f
    private var pitch = 0f
    private var pullBackAfter = Long.MIN_VALUE
    private var closed = false

    private val listener =
        object : FullscreenListener {
            override fun onLook(
                yaw: Float,
                pitch: Float,
            ) = look(yaw, pitch)

            override fun onClick() = click()

            override fun onClose() {
                if (closed) return
                closed = true
                view.invokeOnClose()
            }
        }

    fun open() {
        try {
            view.attach(this, reactivity)
            view.invokeBind(bindings)
            bindings.sealed = true
            bindings.validate()
            layout = layers.seed().component
            val content = content()
            screen = host.open(definition, content, listener)
            sent = content
        } catch (failure: Throwable) {
            closed = true
            throw failure
        }
        view.invokeOnOpen()
    }

    override fun close() {
        if (closed) return
        closed = true
        view.invokeOnClose()
        screen.close()
    }

    fun refreshAll() {
        reactivity.markAllDirty(layers.renderKeys())
    }

    private fun flush(dirty: Set<RenderKey>) {
        if (closed) return
        if (layers.update(dirty)) layout = layers.compose().component
        hovered = hover(cursor())
        show()
    }

    private fun look(
        yaw: Float,
        pitch: Float,
    ) {
        if (closed) return
        this.yaw = yaw
        this.pitch = pitch
        mapping.pullBack(yaw, pitch)?.let { turn ->
            val now = nanoTime()
            if (now < pullBackAfter) return@let
            screen.turn(turn.yaw, turn.pitch)
            this.yaw += turn.yaw
            this.pitch += turn.pitch
            pullBackAfter = now + (screen.latencyMillis + PULL_BACK_COOLDOWN_MILLIS) * 1_000_000L
        }
        val next = hover(cursor())
        if (next == hovered) return
        hovered = next
        show()
    }

    private fun click() {
        if (closed) return
        val point = cursor()
        val current = layers.switches.current()
        val actions =
            entry.controls.filter { (key, control) ->
                control.action != null && layers.switches.regionActive(key, current)
            }
        val control =
            actions.values.firstOrNull { mapping.hits(it, point) }
                ?: actions.values.firstOrNull { mapping.hits(it, point, definition.canvas.hitMargin) }
                ?: return
        val handler = bindings.buttons[control.action]
        if (handler != null) {
            handler(FullscreenClick(point.x, point.y))
        } else {
            when (control.defaultAction?.let(RuntimeAction::of)) {
                RuntimeAction.CLOSE -> close()
                null -> Unit
            }
        }
    }

    private fun cursor(): CanvasPoint = mapping.cursor(yaw, pitch)

    /** The first active control under [point], as the shader draws it. */
    private fun hover(point: CanvasPoint): String? =
        entry.controls.entries
            .firstOrNull { (key, control) -> layers.switches.regionActive(key) && mapping.hits(control, point) }
            ?.key

    /** The backdrop, the window's layers, the hovered control's outline, and the cursor. */
    private fun content(): Component =
        Component
            .text()
            .append(
                listOfNotNull(definition.backdrop, layout, hovered?.let(definition.outlines::get), definition.cursor),
            ).build()

    private fun show() {
        val content = content()
        if (content == sent) return
        screen.show(content)
        sent = content
    }

    /** The [FullscreenScope] passed to `bind`, and the check that every control requiring a binding received one. */
    private inner class Bindings : FullscreenScope {
        var sealed = false
        val buttons = HashMap<String, (FullscreenClick) -> Unit>()

        /** The action ids controls name that a handler can bind; runtime actions are run by the runtime. */
        private val actions: Set<String> =
            entry.controls.values
                .mapNotNull { it.action }
                .filterTo(HashSet()) { RuntimeAction.of(it) == null }

        override fun slot(
            name: String,
            render: () -> Component,
        ) {
            requireBindable(name)
            layers.slots.bind(name, render)
        }

        override fun switch(
            name: String,
            render: () -> String,
        ) {
            requireBindable(name)
            layers.switches.bind(name, render)
        }

        override fun tint(
            name: String,
            render: () -> String,
        ) {
            requireBindable(name)
            layers.tints.bind(name, render)
        }

        override fun bar(
            name: String,
            render: () -> Float,
        ) {
            requireBindable(name)
            layers.bars.bind(name, render)
        }

        override fun button(
            name: String,
            handler: (FullscreenClick) -> Unit,
        ) {
            requireBindable(name)
            require(name in actions) {
                "Unknown button '$name' in fullscreen '${definition.name}'; known buttons: ${actions.sorted()}"
            }
            require(buttons.put(name, handler) == null) { "Button '$name' bound more than once" }
        }

        fun validate() {
            layers.validate()
            val controls = entry.controls.values
            val unknown = controls.mapNotNull { it.defaultAction }.filterTo(HashSet()) { RuntimeAction.of(it) == null }
            check(
                unknown.isEmpty(),
            ) { "Unknown runtime actions in fullscreen '${definition.name}': ${unknown.sorted()}" }
            val unbound =
                controls.filter { it.defaultAction == null }.mapNotNullTo(HashSet()) { it.action } - buttons.keys
            check(unbound.isEmpty()) {
                "Buttons in fullscreen '${definition.name}' have neither a handler nor a default: ${unbound.sorted()}"
            }
        }

        private fun requireBindable(name: String) {
            check(!sealed) { "Bindings are fixed once bind() returns; declare '$name' in bind()" }
        }
    }

    private companion object {
        /** Added to the player's latency to space edge pull-backs one round trip apart. */
        const val PULL_BACK_COOLDOWN_MILLIS = 100
    }
}
