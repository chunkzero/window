package dev.oglass.window.internal

import dev.oglass.window.WindowDefinition
import dev.oglass.window.diagnostics.RenderLayerKind
import dev.oglass.window.diagnostics.RenderStyleTrace
import dev.oglass.window.manifest.Align
import dev.oglass.window.manifest.SpriteSlotEntry

/**
 * The net-zero title segments of one window, keyed by semantic id in composition order: button
 * state sprites, then sprite slots, then text slots.
 */
internal class WindowTitle(
    private val definition: WindowDefinition,
    private val bindings: WindowBindings,
    private val reactivity: Reactivity,
) {
    private val entry = definition.entry
    private val composer = definition.composer
    private val segments = LinkedHashMap<String, RenderedSegment>()

    fun compose(): ComposedRender = composer.compose(definition.name, segments)

    /** Renders fixed sprites, runtime sprites, and text slots once. */
    fun seedContent() {
        for ((name, slot) in entry.spriteSlots) {
            val sprite = slot.sprite ?: continue
            segments[spriteId(name)] = spriteSegment(name, slot, sprite)
        }
        for ((name, slot) in entry.spriteSlots) {
            if (slot.sprite == null) updateSprite(name)
        }
        bindings.slots.seed { name, segment -> segments[bindings.slots.semanticId(name)] = segment }
    }

    fun updateSlot(name: String) {
        if (!bindings.slots.isBound(name)) return
        segments[bindings.slots.semanticId(name)] = bindings.slots.render(name)
    }

    /** Re-renders a runtime sprite slot under dependency capture. */
    fun updateSprite(name: String) {
        val render = bindings.sprites[name] ?: return
        val sprite = reactivity.withRendering(RenderKey.Sprite(name)) { render() }
        segments[spriteId(name)] = spriteSegment(name, entry.spriteSlots.getValue(name), sprite)
    }

    /** Reserves an empty segment for button [name]'s state sprite until a state is set. */
    fun reserveButtonVisual(name: String) {
        val button = entry.buttons.getValue(name)
        val font = button.spriteFont ?: return
        val id = buttonId(name)
        segments[id] = definition.emptyTitleSegment(id, RenderLayerKind.SPRITE_SLOT, button.x, button.y, font)
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
        segments[id] =
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
    }

    private fun spriteSegment(
        name: String,
        slot: SpriteSlotEntry,
        sprite: String?,
    ): RenderedSegment {
        val id = spriteId(name)
        return composer.renderSprite(id, slot, sprite)
            ?: definition.emptyTitleSegment(id, RenderLayerKind.SPRITE_SLOT, slot.x, slot.y, slot.font)
    }

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
