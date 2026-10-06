package com.chunkzero.window.host.testkit

import com.chunkzero.window.host.HudChannel
import com.chunkzero.window.host.WindowHost
import net.kyori.adventure.text.Component

/**
 * One fresh player on a headless server, plus a simulated client for [HostConformance].
 *
 * The fixture drives client input through the server's real packet handling where it can, and records what the client
 * receives as [updates]. Closing the fixture removes every listener it added to the server.
 */
public interface HostFixture<I : Any> : AutoCloseable {
    /** The host of the fixture's player. */
    public val host: WindowHost<I>

    /** What the client received (or did itself, such as closing its screen), in order. */
    public val updates: List<ClientUpdate<I>>

    /** Clicks raw protocol slot [windowSlot] of the open screen: shift sends a quick move, right uses button 1. */
    public fun click(
        windowSlot: Int,
        shift: Boolean = false,
        right: Boolean = false,
    )

    /** Clicks raw slot [windowSlot] of the player's own inventory window (id 0), as if no other screen were open. */
    public fun clickPlayerWindow(windowSlot: Int)

    /** Closes the open screen from the client. */
    public fun closeScreen()

    /** Disconnects the player: the server removes it while its screen is open. */
    public fun disconnect()

    /** Types [text] into the open anvil's text box. */
    public fun typeInAnvil(text: String)

    /** Answers ping [id]. */
    public fun pong(id: Int)

    /** Runs one tick of the scheduler that serves the player. */
    public fun tick()

    /** Adds a listener that runs before the host's and cancels every click on the player's own inventory. */
    public fun guardPlayerInventory()

    /** Adds a server-wide listener that runs after the host's and un-cancels every click of the player. */
    public fun allowClicksGlobally()

    /** Adds a server-wide listener that runs after the host's and cancels every inventory open of the player. */
    public fun cancelOpensGlobally()

    /** Adds a server-wide listener that runs after the host's and cancels every item drop of the player. */
    public fun cancelDropsGlobally()

    /** Puts [item] on the player's cursor; `null` clears it. */
    public fun setCursorItem(item: I?)

    /** The item on the player's cursor, or `null` when it is empty. */
    public fun cursorItem(): I?

    /** Puts [item] in the player's real inventory slot [slot]; `null` clears it. */
    public fun setPlayerItem(
        slot: Int,
        item: I?,
    )

    /** The item in the player's real inventory slot [slot], or `null` when it is empty. */
    public fun playerItem(slot: Int): I?
}

/** A change the simulated client saw. */
public sealed interface ClientUpdate<out I : Any> {
    /** A screen opened, or the open one was reopened, titled [title]. */
    public data class OpenScreen(
        val title: Component,
    ) : ClientUpdate<Nothing>

    /** Raw slot [slot] of the open screen now shows [item]; `null` is empty. */
    public data class SetSlot<out I : Any>(
        val slot: Int,
        val item: I?,
    ) : ClientUpdate<I>

    /** The open screen closed. */
    public data object CloseScreen : ClientUpdate<Nothing>

    /** A bundle delimiter: the packets between two delimiters apply at once. */
    public data object BundleDelimiter : ClientUpdate<Nothing>

    /** A ping the client answers with pong [id]. */
    public data class Ping(
        val id: Int,
    ) : ClientUpdate<Nothing>

    /** The HUD on [channel] now shows [content]; `null` hides it. */
    public data class Hud(
        val channel: HudChannel,
        val content: Component?,
    ) : ClientUpdate<Nothing>
}
