package com.chunkzero.window.host

import com.chunkzero.window.SlotRef
import net.kyori.adventure.text.Component

/** A container a [WindowHost] opened for its player. */
public interface OpenContainer<I : Any> {
    /** Number of container slots (excluding the player's inventory). */
    public val size: Int

    /**
     * Changes the title. For [ContainerKind.ANVIL] this reopens the screen on the same container, delivering staged
     * items with it, without reporting [ContainerListener.onClose]; the client's text box resets to the seed name.
     */
    public fun setTitle(title: Component)

    /** Sets a container or player slot; `null` clears it. Player slots are leased until [restorePlayerSlots]. */
    public fun setItem(
        slot: SlotRef,
        item: I?,
    )

    /** Stores a container item without sending it; the next [setTitle] delivers it. */
    public fun stageItem(
        slot: Int,
        item: I?,
    )

    /** Restores the player's real inventory after Window used player slots. */
    public fun restorePlayerSlots()

    /** Sends the packets produced by [action] as one bundle, in order, that the client applies at once. */
    public fun batch(action: () -> Unit)

    /** Sends a ping whose pong reaches [ContainerListener.onPong] in packet order. */
    public fun ping(id: Int)

    /** Server-initiated close: stops listening, restores player slots, closes the screen. Does not call onClose. */
    public fun close()
}

/** Input for an open container, already normalised to Window's slot model. */
public interface ContainerListener {
    /** A click on [slot]; delivered even when another listener already cancelled the server's click event. */
    public fun onClick(
        slot: SlotRef,
        shift: Boolean,
        right: Boolean,
    )

    /**
     * The client closed the container, or another inventory replaced it. Listening has stopped and the player's slots
     * are restored. It runs while the server is still closing or replacing the inventory (including when the player
     * disconnects), so opening another container from it must be deferred, e.g. to the next tick.
     */
    public fun onClose()

    /** The anvil text box changed (including echoes after a reopen; the core filters those). */
    public fun onAnvilInput(text: String)

    /** The client answered ping [id]. */
    public fun onPong(id: Int)
}

/** The kinds of container Window opens, with their container slot counts. */
public enum class ContainerKind(
    public val size: Int,
) {
    CHEST_1_ROW(9),
    CHEST_2_ROW(18),
    CHEST_3_ROW(27),
    CHEST_4_ROW(36),
    CHEST_5_ROW(45),
    CHEST_6_ROW(54),
    ANVIL(3),
}
