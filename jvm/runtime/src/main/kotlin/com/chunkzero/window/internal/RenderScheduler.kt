package com.chunkzero.window.internal

import com.chunkzero.window.WindowPlatform
import net.minestom.server.entity.Player

/**
 * Schedules a batched re-render task. Abstracted so tests can inject a manual scheduler instead of
 * relying on the live server tick loop.
 */
internal fun interface RenderScheduler {
    /** Schedules [task] to run once, on the next scheduler tick. */
    fun schedule(task: Runnable)

    companion object {
        /** Default scheduler: runs the task on the next tick of the server serving [player]. */
        fun nextTick(
            platform: WindowPlatform,
            player: Player,
        ): RenderScheduler = RenderScheduler { task -> platform.scheduleNextTick(player, task) }
    }
}
