package com.chunkzero.window

import com.chunkzero.window.diagnostics.RenderCorrelation
import com.chunkzero.window.diagnostics.RenderCursorConvention
import com.chunkzero.window.diagnostics.RenderFrameReason
import com.chunkzero.window.diagnostics.RenderSurfaceKind
import com.chunkzero.window.internal.ComposedRender
import com.chunkzero.window.internal.FrameCursor
import com.chunkzero.window.internal.HudLayers
import com.chunkzero.window.internal.SessionFrames
import com.chunkzero.window.internal.dynamicSlot
import net.kyori.adventure.text.Component
import org.slf4j.LoggerFactory

/**
 * A user-defined view over a HUD: declares the providers of its slots, switches, tints, and bars, and composes them
 * on [render].
 *
 * A view only produces content. The caller decides when to render, sends the result on [channel], and resends it as
 * needed. A view is not thread-safe; render it from one thread at a time.
 */
public abstract class HudView(
    private val definition: HudDefinition,
) {
    private val entry = definition.entry
    private val layers = HudLayers("hud", definition.name, entry, definition.composer, null)

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

    private var current: Component? = null
    private var bound = false

    /** The channel this HUD's layout was composed for; send [render]'s result only on this channel. */
    public val channel: HudChannel
        get() = definition.channel

    /**
     * Evaluates every slot, switch, tint, and bar provider and returns the composed HUD.
     *
     * The first call runs [bind]. Slots whose provider returns an equal component, switches and tints that keep
     * their case or palette entry, and bars that keep their columns reuse their previous layout.
     *
     * @throws IllegalStateException on the first call if a dynamic slot, switch, tint, or bar is unbound.
     * @throws IllegalArgumentException for unknown names referenced in [bind], or an unknown switch case or palette
     *   entry.
     */
    public fun render(): Component {
        val previous = current ?: return publish(composeInitialRender(), RenderFrameReason.OPEN)
        return if (layers.update(null)) publish(layers.compose(), RenderFrameReason.REACTIVE_UPDATE) else previous
    }

    /** Registers the providers of this HUD's slots, switches, tints, and bars; runs once, on the first [render]. */
    protected abstract fun HudScope.bind()

    /**
     * [value] shortened with an ellipsis so that it, followed by [suffix], fits the width of slot [slot] when drawn in
     * its font and style; followed by [suffix], which is kept even when it alone is wider. Content that already fits is returned unchanged. Use it to keep a
     * suffix, such as a score, while shortening the text before it.
     *
     * [slot] is the slot's entry name, such as `name[2]` for one entry of an indexed handle.
     *
     * @throws IllegalArgumentException if [slot] is not a dynamic text slot of this HUD.
     */
    protected fun fit(
        slot: String,
        value: Component,
        suffix: Component = Component.empty(),
    ): Component =
        definition.composer.fit(
            entry.slots.dynamicSlot(slot, "hud", definition.name),
            value,
            suffix,
        )

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
                        layers.slots.bind(name, render)
                    }

                    override fun switch(
                        name: String,
                        render: () -> String,
                    ) {
                        check(!sealed) { "Bindings are fixed once bind() returns; declare '$name' in bind()" }
                        layers.switches.bind(name, render)
                    }

                    override fun tint(
                        name: String,
                        render: () -> String,
                    ) {
                        check(!sealed) { "Bindings are fixed once bind() returns; declare '$name' in bind()" }
                        layers.tints.bind(name, render)
                    }

                    override fun bar(
                        name: String,
                        render: () -> Float,
                    ) {
                        check(!sealed) { "Bindings are fixed once bind() returns; declare '$name' in bind()" }
                        layers.bars.bind(name, render)
                    }
                }
            scope.bind()
            sealed = true
            layers.validate()
            bound = true
        }
        return layers.seed()
    }

    private fun publish(
        render: ComposedRender,
        reason: RenderFrameReason,
    ): Component {
        current = render.component
        frames.observe(render, reason)
        return render.component
    }

    private companion object {
        val LOGGER = LoggerFactory.getLogger(HudView::class.java)
    }
}
