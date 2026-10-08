package com.chunkzero.window.internal

import com.chunkzero.window.WindowDefinition
import com.chunkzero.window.diagnostics.RenderLayerKind
import com.chunkzero.window.diagnostics.RenderStyleTrace
import com.chunkzero.window.manifest.LayerEntry
import com.chunkzero.window.manifest.LayerKind
import com.chunkzero.window.manifest.SpriteSlotEntry
import com.chunkzero.window.manifest.WindowEntry

/**
 * The net-zero title segments of one window. They compose above the static chrome in the order of the
 * window's layers, leaving out entries of inactive switch cases; a switch layer draws its active
 * case's art.
 */
internal class WindowTitle(
    private val definition: WindowDefinition,
    private val bindings: WindowBindings<*>,
    private val reactivity: Reactivity,
) {
    private val entry = definition.entry
    private val composer = definition.composer
    private val switches = bindings.switches
    private val segments = HashMap<String, RenderedSegment>()
    private var composed: ComposedRender? = null

    /**
     * Composes the current segments; repeated calls return the same render until a segment is written or
     * [updateCases] reports a case change.
     */
    fun compose(): ComposedRender =
        composed ?: composer.compose(definition.name, entry.layers.mapNotNull(::layer)).also { composed = it }

    /** Marks the title for recomposition after the active switch cases changed. */
    fun updateCases() {
        composed = null
    }

    /** Renders collection selections, sprite slots, and text slots once. */
    fun seed() {
        for (name in bindings.collectionSelections.keys) updateCollectionSelection(name)
        for ((name, slot) in entry.spriteSlots) {
            val sprite = slot.sprite
            if (sprite == null) updateSprite(name) else put(spriteId(name), spriteSegment(spriteId(name), slot, sprite))
        }
        bindings.slots.seed { name, segment -> put(bindings.slots.semanticId(name), segment) }
    }

    fun updateSlot(name: String) {
        if (!bindings.slots.isBound(name)) return
        put(bindings.slots.semanticId(name), bindings.slots.render(name))
    }

    /** Re-renders a runtime sprite slot under dependency capture. */
    fun updateSprite(name: String) {
        val render = bindings.sprites[name] ?: return
        val sprite = reactivity.withRendering(RenderKey.Sprite(name)) { render() }
        val id = spriteId(name)
        put(id, spriteSegment(id, entry.spriteSlots.getValue(name), sprite))
    }

    /** Re-renders collection [name]'s selected-cell sprite under dependency capture. */
    fun updateCollectionSelection(name: String) {
        val render = bindings.collectionSelections[name] ?: return
        val cells = entry.collections.getValue(name).selection
        val index = reactivity.withRendering(RenderKey.CollectionSelection(name)) { render() }
        val id = selectionId(name)
        val cell = index?.let(cells::getOrNull)
        put(id, spriteSegment(id, cell ?: cells.first(), cell?.sprite))
    }

    /** The segment [layer] draws now, labeled with the authored element it comes from. */
    private fun layer(layer: LayerEntry): RenderedSegment? {
        val name = layer.name
        val segment =
            when (layer.kind) {
                LayerKind.SLOT -> {
                    segments[bindings.slots.semanticId(name)]?.takeIf { switches.slotActive(name) }
                }

                LayerKind.SPRITE_SLOT -> {
                    segments[spriteId(name)]?.takeIf { switches.spriteSlotActive(name) }
                }

                LayerKind.COLLECTION -> {
                    return segments[selectionId(name)]?.takeIf { switches.collectionActive(name) }
                }

                LayerKind.SWITCH -> {
                    val art = switches.activeCase(name)?.static?.takeIf { it.isNotEmpty() } ?: return null
                    return composer.renderStatic(switches.semanticId(name), art).sourced(switches.source(name))
                }
            }
        val source =
            when (layer.kind) {
                LayerKind.SLOT -> entry.slots[name]?.source
                else -> entry.spriteSlots[name]?.source
            }
        return segment?.sourced(source ?: switches.entrySource(name))
    }

    private fun put(
        id: String,
        segment: RenderedSegment,
    ) {
        segments[id] = segment
        composed = null
    }

    private fun spriteSegment(
        id: String,
        slot: SpriteSlotEntry,
        sprite: String?,
    ): RenderedSegment =
        composer.renderSprite(id, slot, sprite)
            ?: definition.emptyTitleSegment(id, RenderLayerKind.SPRITE_SLOT, slot.x, slot.y, slot.font)

    private fun spriteId(name: String): String = "window/${definition.name}/sprite/$name"

    private fun selectionId(name: String): String = "window/${definition.name}/selection/$name"
}

private val EMPTY_STYLE = RenderStyleTrace(color = "#ffffff", shadow = false)

/** A title segment that draws nothing, with its cursor reset at the title origin. */
internal fun WindowDefinition.emptyTitleSegment(
    semanticId: String,
    kind: RenderLayerKind,
    x: Int,
    y: Int,
    font: String,
): RenderedSegment = emptySegment(semanticId, kind, x, y, font, EMPTY_STYLE, entry.surface.titleOrigin[0])

/** This window's text slots, rendered as title segments. */
internal fun WindowDefinition.titleSlots(reactivity: Reactivity): DynamicSlots =
    DynamicSlots("window", name, entry.slots, reactivity, composer::renderSlot) { id, slot ->
        emptyTitleSegment(id, RenderLayerKind.TEXT_SLOT, slot.x, slot.y, slot.font)
    }

/**
 * Whether the title never changes after open: no text slots, bound sprites, or selections, and no
 * switch case with art, slots, or sprite slots. Switches that only swap regions keep the title.
 */
internal val WindowEntry.hasStaticTitle: Boolean
    get() =
        slots.values.all { it.text != null } &&
            spriteSlots.values.all { it.sprite != null } &&
            collections.values.all { it.selection.isEmpty() } &&
            switches.values.all { switch ->
                switch.cases.all { it.static.isEmpty() && it.slots.isEmpty() && it.spriteSlots.isEmpty() }
            }
