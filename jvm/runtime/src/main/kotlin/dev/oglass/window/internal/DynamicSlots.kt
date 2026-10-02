package dev.oglass.window.internal

import dev.oglass.window.manifest.SlotEntry
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
    private val reactivity: Reactivity,
    private val renderSlot: (semanticId: String, slot: SlotEntry, content: Component) -> RenderedSegment?,
    private val emptySlot: (semanticId: String, slot: SlotEntry) -> RenderedSegment,
) {
    /** Slots without a static `text` label. */
    val names: Set<String> = slots.filterValues { it.text == null }.keys

    private val renders = HashMap<String, () -> Component>()

    fun bind(
        name: String,
        render: () -> Component,
    ) {
        val slot =
            slots[name]
                ?: throw IllegalArgumentException(
                    "Unknown slot '$name' in $surface '$ownerName'; known slots: ${names.sorted()}",
                )
        require(slot.text == null) { "Slot '$name' is a static label and must not be bound" }
        require(renders.put(name, render) == null) { "Slot '$name' bound more than once" }
    }

    fun validate() {
        val unbound = names - renders.keys
        check(unbound.isEmpty()) {
            "Unbound dynamic slots in $surface '$ownerName': ${unbound.sorted()}"
        }
    }

    fun isBound(name: String): Boolean = name in renders

    fun semanticId(name: String): String = "$surface/$ownerName/slot/$name"

    /** Renders every slot in definition order, skipping static labels that draw nothing. */
    fun seed(put: (name: String, segment: RenderedSegment) -> Unit) {
        for ((name, slot) in slots) {
            val text = slot.text
            val segment =
                if (text == null) {
                    render(name)
                } else {
                    renderSlot(semanticId(name), slot, Component.text(text)) ?: continue
                }
            put(name, segment)
        }
    }

    /** Renders one bound dynamic slot under dependency capture. */
    fun render(name: String): RenderedSegment {
        val render = renders.getValue(name)
        val content = reactivity.withRendering(RenderKey.Slot(name)) { render() }
        val slot = slots.getValue(name)
        val id = semanticId(name)
        return renderSlot(id, slot, content) ?: emptySlot(id, slot)
    }
}
