package com.chunkzero.window

import com.chunkzero.window.diagnostics.RenderCorrelation
import com.chunkzero.window.diagnostics.RenderCursorConvention
import com.chunkzero.window.diagnostics.RenderFrameReason
import com.chunkzero.window.diagnostics.RenderLayerKind
import com.chunkzero.window.diagnostics.RenderStyleTrace
import com.chunkzero.window.diagnostics.RenderSurfaceKind
import com.chunkzero.window.internal.ComposedRender
import com.chunkzero.window.internal.DynamicSlots
import com.chunkzero.window.internal.FrameCursor
import com.chunkzero.window.internal.RenderedSegment
import com.chunkzero.window.internal.SessionFrames
import com.chunkzero.window.internal.Switches
import com.chunkzero.window.internal.emptySegment
import com.chunkzero.window.manifest.SwitchCaseEntry
import net.kyori.adventure.text.Component
import org.slf4j.LoggerFactory

/**
 * A user-defined view over a HUD: declares the providers of its slots and switches, and composes them on [render].
 *
 * A view only produces content. The caller decides when to render, sends the result on [channel], and resends it as
 * needed. A view is not thread-safe; render it from one thread at a time.
 */
public abstract class HudView(
    private val definition: HudDefinition,
) {
    private val entry = definition.entry
    private val switches = Switches("hud", definition.name, entry.switches, null)
    private val slots =
        DynamicSlots("hud", definition.name, entry.slots, null, definition.composer::renderSlot) { id, slot ->
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
            { observer, frame -> observer.observeHud(this, frame) },
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

    private val contents = HashMap<String, Component>()
    private val segments = LinkedHashMap<String, RenderedSegment>()
    private val cases = HashMap<String, SwitchCaseEntry>()
    private val caseArt = LinkedHashMap<String, String>()
    private var current: Component? = null
    private var bound = false
    private var unpublished = false

    /** The channel this HUD's layout was composed for; send [render]'s result only on this channel. */
    public val channel: HudChannel
        get() = definition.channel

    /**
     * Evaluates every slot and switch provider and returns the composed HUD.
     *
     * The first call runs [bind]. Slots whose provider returns an equal component, and switches that keep their case,
     * reuse their previous layout.
     *
     * @throws IllegalStateException on the first call if a dynamic slot or switch is unbound.
     * @throws IllegalArgumentException for unknown names referenced in [bind], or an unknown switch case.
     */
    public fun render(): Component {
        val previous = current ?: return publish(composeInitialRender(), RenderFrameReason.OPEN)
        for (name in switches.names) unpublished = updateSwitch(name) || unpublished
        for (name in slots.names) {
            val content = slots.content(name)
            if (contents[name] == content) continue
            contents[name] = content
            segments[name] = slots.segment(name, content)
            unpublished = true
        }
        return if (unpublished) publish(compose(), RenderFrameReason.REACTIVE_UPDATE) else previous
    }

    /** Registers the providers of this HUD's slots and switches; runs once, on the first [render]. */
    protected abstract fun HudScope.bind()

    private fun composeInitialRender(): ComposedRender {
        if (!bound) {
            val scope =
                object : HudScope {
                    override fun slot(
                        name: String,
                        render: () -> Component,
                    ) = slots.bind(name, render)

                    override fun switch(
                        name: String,
                        render: () -> String,
                    ) = switches.bind(name, render)
                }
            scope.bind()
            slots.validate()
            switches.validate()
            bound = true
        }
        for (name in switches.names) updateSwitch(name)
        slots.seed(::seedSlot) { name, segment -> segments[name] = segment }
        return compose()
    }

    private fun seedSlot(name: String): RenderedSegment {
        val content = slots.content(name)
        contents[name] = content
        return slots.segment(name, content)
    }

    /** Selects switch [name]'s case; returns whether it changed. */
    private fun updateSwitch(name: String): Boolean {
        val case = switches.render(name)
        if (cases.put(name, case) === case) return false
        caseArt[switches.semanticId(name)] = case.static
        return true
    }

    private fun publish(
        render: ComposedRender,
        reason: RenderFrameReason,
    ): Component {
        current = render.component
        unpublished = false
        frames.observe(render, reason)
        return render.component
    }

    /** Composes the HUD without the slots of inactive switch cases. */
    private fun compose(): ComposedRender {
        val hiddenSlots = switches.hiddenSlots()
        return definition.composer.compose(definition.name, segments.filterKeys { it !in hiddenSlots }, caseArt)
    }

    private companion object {
        val LOGGER = LoggerFactory.getLogger(HudView::class.java)
    }
}
