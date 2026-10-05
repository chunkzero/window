package com.chunkzero.window.internal

/** A reactive render target: the unit a state read depends on and a flush re-renders. */
internal sealed interface RenderKey {
    /** A dynamic text slot in the title or HUD. */
    data class Slot(
        val name: String,
    ) : RenderKey

    /** A runtime sprite slot in the title. */
    data class Sprite(
        val name: String,
    ) : RenderKey

    /** The inventory item bound to a button or hotspot. */
    data class ButtonItem(
        val name: String,
    ) : RenderKey

    /** The named state of a button, driving both its item and its title sprite. */
    data class ButtonState(
        val name: String,
    ) : RenderKey

    /** A dynamic item region. */
    data class Item(
        val name: String,
    ) : RenderKey

    /** One cell of a repeated item collection. */
    data class CollectionCell(
        val name: String,
        val index: Int,
    ) : RenderKey

    /** The selected-cell sprite of a repeated item collection. */
    data class CollectionSelection(
        val name: String,
    ) : RenderKey

    /** The active case of a switch in the title or HUD. */
    data class Switch(
        val name: String,
    ) : RenderKey
}
