package com.chunkzero.window.internal

import com.chunkzero.window.manifest.SlotEntry
import net.kyori.adventure.text.Component

/**
 * The text slots of one window or HUD: binds render lambdas to dynamic slots, validates that each
 * is bound, and renders static labels and dynamic slots into segments.
 */
internal class DynamicSlots(
    /** The surface name used in semantic ids and messages, `window` or `hud`. */
    private val surface: String,
    private val ownerName: String,
    private val slots: Map<String, SlotEntry>,
    /** Captures state reads per slot; `null` for surfaces that render without reactivity. */
    private val reactivity: Reactivity?,
    private val renderSlot: (semanticId: String, slot: SlotEntry, content: Component) -> RenderedSegment?,
    private val emptySlot: (semanticId: String, slot: SlotEntry) -> RenderedSegment,
) {
    /** Slots without a static `text` label. */
    val names: Set<String> = slots.filterValues { it.text == null }.keys

    /** The dynamic slot keys of each binding name; a binding shared across switch cases has one per case. */
    private val bindings: Map<String, List<String>> = names.groupBy { slots.getValue(it).binding ?: it }

    private val renders = HashMap<String, () -> Component>()

    /** Binds [render] to binding [name], which renders every slot copy that shares it. */
    fun bind(
        name: String,
        render: () -> Component,
    ) {
        val keys =
            bindings[name]
                ?: if (slots[name]?.text != null) {
                    throw IllegalArgumentException("Slot '$name' is a static label and must not be bound")
                } else {
                    throw IllegalArgumentException(
                        "Unknown slot '$name' in $surface '$ownerName'; known slots: ${bindings.keys.sorted()}",
                    )
                }
        require(keys.none { it in renders }) { "Slot '$name' bound more than once" }
        for (key in keys) renders[key] = render
    }

    fun validate() {
        val unbound = bindings.filterValues { keys -> keys.any { it !in renders } }.keys
        check(unbound.isEmpty()) {
            "Unbound dynamic slots in $surface '$ownerName': ${unbound.sorted()}"
        }
    }

    fun isBound(name: String): Boolean = name in renders

    fun semanticId(name: String): String = "$surface/$ownerName/slot/$name"

    /**
     * Renders every slot in definition order, dynamic ones through [dynamic], skipping static labels that draw
     * nothing.
     */
    fun seed(
        dynamic: (name: String) -> RenderedSegment = ::render,
        put: (name: String, segment: RenderedSegment) -> Unit,
    ) {
        for ((name, slot) in slots) {
            val text = slot.text
            val segment =
                if (text == null) {
                    dynamic(name)
                } else {
                    renderSlot(semanticId(name), slot, Component.text(text)) ?: continue
                }
            put(name, segment)
        }
    }

    /** Renders one bound dynamic slot under dependency capture. */
    fun render(name: String): RenderedSegment = segment(name, content(name))

    /** Evaluates bound dynamic slot [name]'s render lambda under dependency capture. */
    fun content(name: String): Component {
        val render = renders.getValue(name)
        return if (reactivity == null) render() else reactivity.withRendering(RenderKey.Slot(name), render)
    }

    /** Lays out [content] as slot [name]'s segment. */
    fun segment(
        name: String,
        content: Component,
    ): RenderedSegment {
        val slot = slots.getValue(name)
        val id = semanticId(name)
        return renderSlot(id, slot, content) ?: emptySlot(id, slot)
    }
}
