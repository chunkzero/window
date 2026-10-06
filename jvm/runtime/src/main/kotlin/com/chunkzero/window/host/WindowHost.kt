package com.chunkzero.window.host

import net.kyori.adventure.text.Component

/**
 * The server side of Window for one player: opens containers, shows HUDs, builds items, and schedules work.
 *
 * Every view and HUD shown to a player must use the same host instance; the core keys per-player state (such as
 * merged action bars) by host identity. Hosts hold no core state themselves.
 *
 * @param I the server's native item type.
 */
public interface WindowHost<I : Any> {
    /**
     * Opens [kind] titled [title] for this host's player; reports its input to [listener] until closed.
     *
     * The container must be the player's open screen when this returns. If the server refuses to show it (an open
     * listener cancelled or redirected the request), nothing may be left behind: no listener registered and no player
     * slot touched. The host then throws [IllegalStateException]. A cancelled open leaves the player's current screen
     * untouched; a redirected one has already replaced it, and that screen reported `onClose`.
     *
     * @throws IllegalStateException if the container could not be opened.
     */
    public fun open(
        kind: ContainerKind,
        title: Component,
        listener: ContainerListener,
    ): OpenContainer<I>

    /**
     * Shows [hud] to this host's player until the returned output is hidden.
     *
     * The core keeps action-bar outputs in a map keyed by this host, so an output must not strongly reference this
     * host or anything that does (such as the player holding it); hold the player weakly instead.
     */
    public fun showHud(hud: HudDescriptor): HudOutput

    /** Builds an item Window renders itself. */
    public fun item(item: WindowItem): I

    /** Runs [task] once on the next tick that serves this host's player. */
    public fun scheduleNextTick(task: Runnable)
}
