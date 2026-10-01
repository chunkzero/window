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
import dev.oglass.window.internal.ActionbarMultiplexer
import dev.oglass.window.internal.ComposedRender
import dev.oglass.window.internal.Reactivity
import dev.oglass.window.internal.RenderScheduler
import dev.oglass.window.internal.RenderedSegment
import net.kyori.adventure.bossbar.BossBar
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import net.minestom.server.scoreboard.Sidebar
import org.slf4j.LoggerFactory
import java.util.UUID
import java.util.concurrent.atomic.AtomicLong

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
        private val diagnosticsObserver: RenderDiagnosticsObserver = RenderDiagnosticsObserver.NONE,
        private val componentSender: ((Component) -> Unit)? = null,
    ) {
        private val entry = definition.entry
        private val composer = definition.composer
        private val actionbarId = ActionbarMultiplexer.nextId()

        private val slotRenders = HashMap<String, () -> Component>()
        private val slotSegments = LinkedHashMap<String, RenderedSegment>()
        private val dynamicSlotNames: Set<String> = entry.slots.filterValues { it.text == null }.keys
        private val reactivity = Reactivity(scheduler) { dirty -> flush(dirty) }
        private val renderSessionId = UUID.randomUUID().toString()
        private val nextFrameId = AtomicLong(1)

        private var bossBar: BossBar? = null
        private var sidebar: Sidebar? = null
        private var shown = false
        private var hidden = false

        internal fun show() {
            check(!shown) { "HUD session already shown" }
            val render = composeInitialRender()
            send(render.component)
            observe(render, RenderFrameReason.OPEN)

            view.invokeOnShow()
            shown = true
        }

        internal fun composeInitial(): Component = composeInitialRender().component

        private fun composeInitialRender(): ComposedRender {
            view.attach(player, this, reactivity)

            collectBindings()
            validateBindings()
            seedSegments()
            return composer.compose(definition.name, slotSegments)
        }

        private fun collectBindings() {
            view.invokeBind(
                object : HudScope {
                    override fun slot(
                        name: String,
                        render: () -> Component,
                    ) {
                        val slot =
                            entry.slots[name]
                                ?: throw IllegalArgumentException(
                                    "Unknown slot '$name' in hud '${definition.name}'; known slots: " +
                                        "${dynamicSlotNames.sorted()}",
                                )
                        require(slot.text == null) {
                            "Slot '$name' is a static label and must not be bound"
                        }
                        require(slotRenders.put(name, render) == null) {
                            "Slot '$name' bound more than once"
                        }
                    }
                },
            )
        }

        private fun validateBindings() {
            val unbound = dynamicSlotNames - slotRenders.keys
            check(unbound.isEmpty()) {
                "Unbound dynamic slots in hud '${definition.name}': ${unbound.sorted()}"
            }
        }

        private fun seedSegments() {
            for ((name, slot) in entry.slots) {
                val text = slot.text
                if (text != null) {
                    slotSegments[name] =
                        composer.renderSlot(
                            "hud/${definition.name}/slot/$name",
                            slot,
                            Component.text(text),
                        ) ?: continue
                } else {
                    slotSegments[name] = renderSegment(name)
                }
            }
        }

        private fun renderSegment(name: String): RenderedSegment {
            val render = slotRenders.getValue(name)
            val content = reactivity.withRendering(name) { render() }
            val slot = entry.slots.getValue(name)
            return composer.renderSlot("hud/${definition.name}/slot/$name", slot, content)
                ?: RenderedSegment(
                    Component.empty(),
                    RenderLayerTrace(
                        semanticId = "hud/${definition.name}/slot/$name",
                        kind = RenderLayerKind.HUD_TEXT,
                        content = "",
                        font = slot.font,
                        style = RenderStyleTrace(slot.color, slot.shadow),
                        expectedBounds = RenderBounds(slot.x, slot.y, 0, 0),
                        cursorStart = 0,
                        contentCursorStart = slot.x,
                        contentCursorEnd = slot.x,
                        cursorEnd = 0,
                        advance = 0,
                        visualWidth = 0,
                        netCursorDelta = 0,
                    ),
                )
        }

        private fun flush(dirty: Set<String>) {
            if (hidden) return
            for (name in dirty) {
                if (name in slotRenders) {
                    slotSegments[name] = renderSegment(name)
                }
            }
            val render = composer.compose(definition.name, slotSegments)
            send(render.component)
            observe(render, RenderFrameReason.REACTIVE_UPDATE)
        }

        private fun observe(
            render: ComposedRender,
            reason: RenderFrameReason,
        ) {
            val frame =
                RenderFrame(
                    frameId = nextFrameId.getAndIncrement(),
                    reason = reason,
                    correlation =
                        RenderCorrelation(
                            surfaceKind = RenderSurfaceKind.HUD,
                            semanticId = definition.name,
                            renderSessionId = renderSessionId,
                            hudChannel = entry.surface.channel,
                        ),
                    cursorConvention = RenderCursorConvention.FIXED_WIDTH_COMPOSITION,
                    cursorStart = 0,
                    cursorEnd = entry.surface.width,
                    netCursorDelta = entry.surface.width,
                    layers = render.layers,
                )
            try {
                diagnosticsObserver.observe(player, frame)
            } catch (error: RuntimeException) {
                LOGGER.warn("Window HUD diagnostics observer failed", error)
            }
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
            refreshAll()
        }

        /** Marks all dynamic slots dirty and schedules a single re-render. */
        internal fun refreshAll() {
            reactivity.markAllDirty(dynamicSlotNames)
        }

        private companion object {
            val LOGGER = LoggerFactory.getLogger(HudSession::class.java)
        }
    }
