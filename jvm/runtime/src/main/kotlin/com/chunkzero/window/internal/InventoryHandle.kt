package com.chunkzero.window.internal

import com.chunkzero.window.SlotRef
import net.kyori.adventure.text.Component
import net.minestom.server.item.ItemStack

/**
 * Abstraction over the inventory-facing side effects of a session, isolating the live Minestom
 * `Inventory`/`Player`/event wiring so the session's binding, click-routing, and reactive logic can
 * be unit-tested headlessly.
 */
internal interface InventoryHandle {
    /** Unsigned Minecraft container id once opened, when the handle has one. */
    val containerId: Int?
        get() = null

    /** Builds the inventory with the given composed [title] and opens it for the player. */
    fun open(title: Component)

    /** Updates the open inventory's title (re-render path). */
    fun setTitle(title: Component)

    /** Sets an item in a typed backing inventory slot. */
    fun setItem(
        slot: SlotRef,
        item: ItemStack,
    )

    /** Stores an item in a container slot without sending it; the next title change delivers it. */
    fun stageItem(
        slot: SlotRef,
        item: ItemStack,
    )

    /** Sends the player a ping whose pong reaches [registerListeners]' `onPong` in packet order. */
    fun ping(id: Int)

    /** Runs [action] with the packets it sends delivered as one bundle the client handles at once. */
    fun bundle(action: () -> Unit)

    /** Registers click, close, native text-input, and pong listeners. */
    fun registerListeners(
        onClick: (ClickInfo) -> Unit,
        onClose: () -> Unit,
        onInput: (String) -> Unit,
        onPong: (Int) -> Unit,
    )

    /** Closes the inventory for the player and tears down listeners. */
    fun close()

    /** Removes listeners without closing (used after a client-initiated close). */
    fun teardownListeners()
}

/** A normalised click: the typed slot and the derived modifiers. */
internal data class ClickInfo(
    val slot: SlotRef,
    val shift: Boolean,
    val right: Boolean,
)
