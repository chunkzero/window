package dev.oglass.window.internal

import net.minestom.server.MinecraftServer

/**
 * Schedules a batched re-render task. Abstracted so tests can inject a manual scheduler instead of
 * relying on the live server tick loop.
 */
internal fun interface RenderScheduler {
    /** Schedules [task] to run once, on the next scheduler tick. */
    fun schedule(task: Runnable)

    companion object {
        /** Default scheduler: runs the task on the next Minestom server tick. */
        val NEXT_TICK: RenderScheduler =
            RenderScheduler { task ->
                MinecraftServer.getSchedulerManager().scheduleNextTick(task)
            }
    }
}
