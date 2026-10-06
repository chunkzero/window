package com.chunkzero.window.internal

import com.chunkzero.window.WindowDefinition
import com.chunkzero.window.diagnostics.RenderLayerKind
import com.chunkzero.window.diagnostics.RenderStyleTrace
import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.SpriteSlotEntry
import com.chunkzero.window.manifest.WindowEntry

/**
 * The net-zero title segments of one window, keyed by semantic id in composition order: switch case
 * art, then button state sprites, then collection selections, then sprite slots, then text slots.
 * Slots of inactive switch cases are left out of the composed title.
 */
internal class WindowTitle(
    private val definition: WindowDefinition,
    private val bindings: WindowBindings<*>,
    private val reactivity: Reactivity,
) {
    private val entry = definition.entry
    private val composer = definition.composer
    private val segments = LinkedHashMap<String, RenderedSegment>()
    private var composed: ComposedRender? = null

    /**
     * Composes the current segments; repeated calls return the same render until a segment is written or
     * [updateSwitch] re-selects a case.
     */
    fun compose(): ComposedRender =
        composed ?: run {
            val switches = bindings.switches
            val hidden =
                switches.hiddenSlots().mapTo(HashSet(), bindings.slots::semanticId) +
                    switches.hiddenSpriteSlots().map(::spriteId)
            composer.compose(definition.name, segments.filterKeys { it !in hidden }).also { composed = it }
        }

    /** Draws the art of switch [name]'s active case, re-selected under dependency capture. */
    fun updateSwitch(name: String) {
        composed = null
        val id = bindings.switches.semanticId(name)
        put(id, composer.renderStatic(id, bindings.switches.render(name).static))
    }

    /** Renders collection selections, fixed sprites, runtime sprites, and text slots once. */
    fun seedContent() {
        for (name in bindings.collectionSelections.keys) updateCollectionSelection(name)
        for ((name, slot) in entry.spriteSlots) {
            val sprite = slot.sprite ?: continue
            put(spriteId(name), spriteSegment(spriteId(name), slot, sprite))
        }
        for ((name, slot) in entry.spriteSlots) {
            if (slot.sprite == null) updateSprite(name)
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
        val id = "window/${definition.name}/selection/$name"
        val cell = index?.let(cells::getOrNull)
        put(id, spriteSegment(id, cell ?: cells.first(), cell?.sprite))
    }

    /** Reserves an empty segment for button [name]'s state sprite until a state is set. */
    fun reserveButtonVisual(name: String) {
        val button = entry.buttons.getValue(name)
        val font = button.spriteFont ?: return
        val id = buttonId(name)
        put(id, definition.emptyTitleSegment(id, RenderLayerKind.SPRITE_SLOT, button.x, button.y, font))
    }

    /** Draws button [name]'s sprite for [stateName], or nothing when the state has no sprite. */
    fun setButtonVisual(
        name: String,
        stateName: String,
    ) {
        val button = entry.buttons.getValue(name)
        val state = definition.requireButtonState(name, stateName)
        val id = buttonId(name)
        val font = button.spriteFont
        val sprite = state.sprite
        val segment =
            if (font == null || sprite == null) {
                definition.emptyTitleSegment(
                    id,
                    RenderLayerKind.SPRITE_SLOT,
                    button.x,
                    button.y,
                    font ?: definition.manifest.font,
                )
            } else {
                val slot =
                    SpriteSlotEntry(
                        x = button.x,
                        y = button.y,
                        width = button.width,
                        height = button.height,
                        align = Align.LEFT,
                        font = font,
                    )
                composer.renderSprite(id, slot, sprite)
                    ?: error("Button '$name' state '$stateName' has an empty sprite")
            }
        put(id, segment)
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

    private fun buttonId(name: String): String = "window/${definition.name}/button/$name"
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
 * Whether the title never changes after open: no text slots, bound sprites, state sprites,
 * selections, or switches.
 */
internal val WindowEntry.hasStaticTitle: Boolean
    get() =
        switches.isEmpty() &&
            slots.values.all { it.text != null } &&
            spriteSlots.values.all { it.sprite != null } &&
            buttons.values.all { it.spriteFont == null } &&
            collections.values.all { it.selection.isEmpty() }
