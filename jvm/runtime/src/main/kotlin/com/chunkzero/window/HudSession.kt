package com.chunkzero.window

import com.chunkzero.window.diagnostics.RenderCorrelation
import com.chunkzero.window.diagnostics.RenderCursorConvention
import com.chunkzero.window.diagnostics.RenderFrameReason
import com.chunkzero.window.diagnostics.RenderLayerKind
import com.chunkzero.window.diagnostics.RenderStyleTrace
import com.chunkzero.window.diagnostics.RenderSurfaceKind
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
import net.kyori.adventure.bossbar.BossBar
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import net.minestom.server.scoreboard.Sidebar
import org.slf4j.LoggerFactory

/**
 * A live HUD instance bound to a player.
 *
 * Owns the vanilla fallback channel, drives reactive slot re-renders, and manages HUD lifecycle.
 * Created via [Windows.show]; not constructed directly.
 */
public class HudSession
    internal constructor(
        private val definition: HudDefinition,
        /** The view driving this session. */
        public val view: HudView,
        private val player: Player,
        scheduler: RenderScheduler,
        diagnosticsObserver: RenderDiagnosticsObserver = RenderDiagnosticsObserver.NONE,
        private val componentSender: ((Component) -> Unit)? = null,
    ) {
        private val entry = definition.entry
        private val composer = definition.composer
        private val actionbarId = ActionbarMultiplexer.nextId()
        private val reactivity = Reactivity(scheduler) { dirty -> flush(dirty) }
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
                player,
                diagnosticsObserver,
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

        private var bossBar: BossBar? = null
        private var sidebar: Sidebar? = null
        private var shown = false
        private var hidden = false

        internal fun show() {
            check(!shown) { "HUD session already shown" }
            val render = composeInitialRender()
            send(render.component)
            frames.observe(render, RenderFrameReason.OPEN)

            view.invokeOnShow()
            shown = true
        }

        internal fun composeInitial(): Component = composeInitialRender().component

        private fun composeInitialRender(): ComposedRender {
            view.attach(player, this, reactivity)
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
            componentSender?.let {
                it(component)
                return
            }
            when (entry.surface.channel) {
                "actionbar" -> {
                    ActionbarMultiplexer.send(player, actionbarId, component)
                }

                "bossbar" -> {
                    sendBossBar(component)
                }

                "sidebar" -> {
                    sendSidebar(component)
                }

                else -> {
                    throw IllegalArgumentException(
                        "Unsupported HUD channel '${entry.surface.channel}' in hud '${definition.name}'",
                    )
                }
            }
        }

        private fun sendBossBar(component: Component) {
            val existing = bossBar
            if (existing == null) {
                val created =
                    BossBar.bossBar(component, 0.0f, BossBar.Color.WHITE, BossBar.Overlay.PROGRESS)
                bossBar = created
                player.showBossBar(created)
            } else {
                existing.name(component)
            }
        }

        private fun sendSidebar(component: Component) {
            val existing = sidebar
            if (existing == null) {
                val created = Sidebar(component)
                sidebar = created
                created.addViewer(player)
            } else {
                existing.setTitle(component)
            }
        }

        /** Hides this HUD and tears down any persistent fallback channel. */
        public fun hide() {
            if (hidden) return
            hidden = true
            view.invokeOnHide()
            bossBar?.let(player::hideBossBar)
            sidebar?.removeViewer(player)
            if (entry.surface.channel == "actionbar") {
                ActionbarMultiplexer.hide(player, actionbarId)
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
