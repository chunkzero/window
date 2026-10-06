package com.chunkzero.window.internal

/** Schedules a batched re-render task. */
internal fun interface RenderScheduler {
    /** Schedules [task] to run once, on the next scheduler tick. */
    fun schedule(task: Runnable)
}
