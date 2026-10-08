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
import com.chunkzero.window.internal.HudLayer
import com.chunkzero.window.internal.RenderedSegment
import com.chunkzero.window.internal.SessionFrames
import com.chunkzero.window.internal.Switches
import com.chunkzero.window.internal.bindingSlot
import com.chunkzero.window.internal.emptySegment
import com.chunkzero.window.internal.sourced
import com.chunkzero.window.manifest.LayerKind
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
        for (name in switches.names) unpublished = switches.update(name) || unpublished
        for (name in slots.names) {
            val content = slots.content(name)
            if (contents[name] == content) continue
            segments[name] = slots.segment(name, content)
            contents[name] = content
            unpublished = true
        }
        return if (unpublished) publish(compose(), RenderFrameReason.REACTIVE_UPDATE) else previous
    }

    /** Registers the providers of this HUD's slots and switches; runs once, on the first [render]. */
    protected abstract fun HudScope.bind()

    /**
     * [value] shortened with an ellipsis so that it, followed by [suffix], fits the width of slot [slot] when drawn in
     * its font and style; followed by [suffix], which is kept even when it alone is wider. Content that already fits is returned unchanged. Use it to keep a
     * suffix, such as a score, while shortening the text before it.
     *
     * [slot] is the slot's authored name, or its flattened entry name such as `name[2]` for an indexed binding. A
     * binding shared across a switch's cases measures with its copy in the first case that has one.
     *
     * @throws IllegalArgumentException if [slot] is not a dynamic text slot of this HUD.
     */
    protected fun fit(
        slot: String,
        value: Component,
        suffix: Component = Component.empty(),
    ): Component {
        val entry = definition.entry
        return definition.composer.fit(
            entry.slots.bindingSlot(slot, entry.switches, "hud", definition.name),
            value,
            suffix,
        )
    }

    private fun composeInitialRender(): ComposedRender {
        if (!bound) {
            var sealed = false
            val scope =
                object : HudScope {
                    override fun slot(
                        name: String,
                        render: () -> Component,
                    ) {
                        check(!sealed) { "Bindings are fixed once bind() returns; declare '$name' in bind()" }
                        slots.bind(name, render)
                    }

                    override fun switch(
                        name: String,
                        render: () -> String,
                    ) {
                        check(!sealed) { "Bindings are fixed once bind() returns; declare '$name' in bind()" }
                        switches.bind(name, render)
                    }
                }
            scope.bind()
            sealed = true
            slots.validate()
            switches.validate()
            bound = true
        }
        for (name in switches.names) switches.update(name)
        slots.seed(::seedSlot) { name, segment -> segments[name] = segment }
        return compose()
    }

    private fun seedSlot(name: String): RenderedSegment {
        val content = slots.content(name)
        val segment = slots.segment(name, content)
        contents[name] = content
        return segment
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

    /** Composes the HUD's layers in order, leaving out the entries of inactive switch cases. */
    private fun compose(): ComposedRender {
        val layers =
            entry.layers.mapNotNull { layer ->
                val name = layer.name
                when (layer.kind) {
                    LayerKind.SLOT -> {
                        val segment = segments[name]?.takeIf { switches.slotActive(name) }
                        segment?.let { HudLayer.Slot(it.sourced(switches.entrySource(name))) }
                    }

                    LayerKind.SWITCH -> {
                        val art = switches.activeCase(name)?.static?.takeIf { it.isNotEmpty() }
                        art?.let { HudLayer.Art(switches.semanticId(name), it, switches.source(name)) }
                    }

                    LayerKind.SPRITE_SLOT, LayerKind.COLLECTION -> {
                        null
                    }
                }
            }
        return definition.composer.compose(definition.name, layers)
    }

    private companion object {
        val LOGGER = LoggerFactory.getLogger(HudView::class.java)
    }
}
