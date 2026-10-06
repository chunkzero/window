package com.chunkzero.window

import com.chunkzero.window.diagnostics.RenderCorrelation
import com.chunkzero.window.diagnostics.RenderCursorConvention
import com.chunkzero.window.diagnostics.RenderFrameReason
import com.chunkzero.window.diagnostics.RenderLayerKind
import com.chunkzero.window.diagnostics.RenderStyleTrace
import com.chunkzero.window.diagnostics.RenderSurfaceKind
import com.chunkzero.window.host.HudChannel
import com.chunkzero.window.host.HudDescriptor
import com.chunkzero.window.host.HudOutput
import com.chunkzero.window.host.WindowHost
import com.chunkzero.window.internal.ActionbarMultiplexer
import com.chunkzero.window.internal.ComposedRender
import com.chunkzero.window.internal.DynamicSlots
import com.chunkzero.window.internal.FrameCursor
import com.chunkzero.window.internal.Reactivity
import com.chunkzero.window.internal.RenderKey
import com.chunkzero.window.internal.RenderScheduler
import com.chunkzero.window.internal.RenderedSegment
import com.chunkzero.window.internal.SessionFrames
import com.chunkzero.window.internal.Switches
import com.chunkzero.window.internal.emptySegment
import net.kyori.adventure.text.Component
import org.slf4j.LoggerFactory

/**
 * A live HUD shown by [HudView.show].
 *
 * Owns the HUD's channel output, drives reactive slot re-renders, and manages HUD lifecycle.
 */
public class HudSession
    internal constructor(
        private val definition: HudDefinition,
        /** The view driving this session. */
        public val view: HudView,
        private val host: WindowHost<*>,
    ) {
        private val entry = definition.entry
        private val composer = definition.composer
        private val actionbarId = ActionbarMultiplexer.nextId()
        private val reactivity = Reactivity(RenderScheduler(host::scheduleNextTick)) { dirty -> flush(dirty) }
        private val segments = LinkedHashMap<String, RenderedSegment>()
        private val switches = Switches("hud", definition.name, entry.switches, reactivity)
        private val caseArt = LinkedHashMap<String, String>()

        private val slots =
            DynamicSlots("hud", definition.name, entry.slots, reactivity, composer::renderSlot) { id, slot ->
                emptySegment(
                    id,
                    RenderLayerKind.HUD_TEXT,
                    slot.x,
                    slot.y,
                    slot.font,
                    RenderStyleTrace(slot.color, slot.shadow),
                    cursor = 0,
                )
            }

        private val frames =
            SessionFrames(
                host,
                FrameCursor(RenderCursorConvention.FIXED_WIDTH_COMPOSITION, 0, entry.surface.width),
                LOGGER,
                "Window HUD diagnostics observer failed",
            ) { renderSessionId ->
                RenderCorrelation(
                    surfaceKind = RenderSurfaceKind.HUD,
                    semanticId = definition.name,
                    renderSessionId = renderSessionId,
                    hudChannel = entry.surface.channel,
                )
            }

        private var output: HudOutput? = null
        private var hidden = false

        internal fun show() {
            val render = composeInitialRender()
            send(render.component)
            frames.observe(render, RenderFrameReason.OPEN)
            view.invokeOnShow()
        }

        private fun composeInitialRender(): ComposedRender {
            view.attach(this, reactivity)
            view.invokeBind(
                object : HudScope {
                    override fun slot(
                        name: String,
                        render: () -> Component,
                    ) = slots.bind(name, render)

                    override fun switch(
                        name: String,
                        render: () -> String,
                    ) = switches.bind(name, render)
                },
            )
            slots.validate()
            switches.validate()
            for (name in switches.names) updateSwitch(name)
            slots.seed { name, segment -> segments[name] = segment }
            return compose()
        }

        private fun flush(dirty: Set<RenderKey>) {
            if (hidden) return
            for (key in dirty) {
                if (key is RenderKey.Slot && slots.isBound(key.name)) {
                    segments[key.name] = slots.render(key.name)
                } else if (key is RenderKey.Switch) {
                    updateSwitch(key.name)
                }
            }
            val render = compose()
            send(render.component)
            frames.observe(render, RenderFrameReason.REACTIVE_UPDATE)
        }

        private fun updateSwitch(name: String) {
            caseArt[switches.semanticId(name)] = switches.render(name).static
        }

        /** Composes the HUD without the slots of inactive switch cases. */
        private fun compose(): ComposedRender {
            val hiddenSlots = switches.hiddenSlots()
            return composer.compose(definition.name, segments.filterKeys { it !in hiddenSlots }, caseArt)
        }

        private fun send(component: Component) {
            if (definition.channel == HudChannel.ACTION_BAR) {
                ActionbarMultiplexer.send(host, actionbarId, component)
                return
            }
            val current = output
            if (current == null) {
                output = host.showHud(HudDescriptor(definition.channel, component))
            } else {
                current.update(component)
            }
        }

        /** Hides this HUD and removes it from its channel. */
        public fun hide() {
            if (hidden) return
            hidden = true
            view.invokeOnHide()
            if (definition.channel == HudChannel.ACTION_BAR) {
                ActionbarMultiplexer.hide(host, actionbarId)
            } else {
                output?.hide()
                output = null
            }
        }

        /** Forces all dynamic slots to re-render on the next scheduler tick. */
        public fun refresh() {
            reactivity.markAllDirty(slots.names.map(RenderKey::Slot) + switches.names.map(RenderKey::Switch))
        }

        private companion object {
            val LOGGER = LoggerFactory.getLogger(HudSession::class.java)
        }
    }
