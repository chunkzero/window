package com.chunkzero.window.internal

import kotlin.properties.ReadWriteProperty
import kotlin.reflect.KProperty

/**
 * Per-session reactive engine.
 *
 * Tracks which render keys read which [state] cells during their render lambda, marks dependent
 * keys dirty on writes, and coalesces the resulting re-renders into a single scheduled flush per
 * burst.
 *
 * The owning session drives the engine: it renders each key inside [withRendering] (so reads
 * register a dependency), and supplies a [flush] callback that re-renders dirty keys and re-sends
 * the composed output.
 */
internal class Reactivity(
    private val scheduler: RenderScheduler,
    /** Re-renders the given dirty keys. Receives a snapshot, then clears. */
    private val flush: (dirty: Set<RenderKey>) -> Unit,
) {
    /** state cell id -> keys that read it during render. */
    private val dependencies = HashMap<Long, MutableSet<RenderKey>>()

    /** Keys awaiting re-render. */
    private val dirty = LinkedHashSet<RenderKey>()

    /** Key currently being rendered, if any; reads during this register a dependency. */
    private var rendering: RenderKey? = null
    private var flushScheduled = false
    private var nextStateId = 0L

    /** Runs [block] with [key] marked as the currently-rendering key for dependency capture. */
    fun <T> withRendering(
        key: RenderKey,
        block: () -> T,
    ): T {
        val previous = rendering
        rendering = key
        try {
            return block()
        } finally {
            rendering = previous
        }
    }

    /** Marks every given key dirty and schedules a flush. Used by `refresh()`. */
    fun markAllDirty(keys: Collection<RenderKey>) {
        dirty.addAll(keys)
        scheduleFlush()
    }

    /** Creates a reactive state delegate bound to this engine. */
    fun <T> state(initial: T): ReadWriteProperty<Any?, T> = StateProperty(nextStateId++, initial)

    private fun registerRead(id: Long) {
        val key = rendering ?: return
        dependencies.getOrPut(id) { HashSet() }.add(key)
    }

    private fun onWrite(id: Long) {
        val dependents = dependencies[id] ?: return
        if (dependents.isEmpty()) return
        dirty.addAll(dependents)
        scheduleFlush()
    }

    private fun scheduleFlush() {
        if (flushScheduled) return
        flushScheduled = true
        scheduler.schedule {
            flushScheduled = false
            if (dirty.isEmpty()) return@schedule
            val snapshot = LinkedHashSet(dirty)
            dirty.clear()
            flush(snapshot)
        }
    }

    private inner class StateProperty<T>(
        private val id: Long,
        initial: T,
    ) : ReadWriteProperty<Any?, T> {
        private var value: T = initial

        override fun getValue(
            thisRef: Any?,
            property: KProperty<*>,
        ): T {
            registerRead(id)
            return value
        }

        override fun setValue(
            thisRef: Any?,
            property: KProperty<*>,
            value: T,
        ) {
            if (this.value == value) return
            this.value = value
            onWrite(id)
        }
    }
}
