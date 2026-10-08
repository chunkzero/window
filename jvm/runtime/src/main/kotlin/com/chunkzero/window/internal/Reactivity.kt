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

    /** State ids read by the memo currently computing, if any. */
    private var capture: MutableSet<Long>? = null

    /** state cell id -> memos whose last computation read it. */
    private val memoSources = HashMap<Long, MutableSet<Memo<*>>>()
    private val memos = ArrayList<Memo<*>>()

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
        memos.forEach { it.invalidate() }
        dirty.addAll(keys)
        scheduleFlush()
    }

    /** Creates a reactive state delegate bound to this engine. */
    fun <T> state(initial: T): ReadWriteProperty<Any?, T> = StateProperty(nextStateId++, initial)

    /** Creates a cached computation bound to this engine. */
    fun <T> memo(compute: () -> T): Memo<T> = Memo(compute).also { memos += it }

    private fun registerRead(id: Long) {
        capture?.add(id)
        val key = rendering ?: return
        dependencies.getOrPut(id) { HashSet() }.add(key)
    }

    private fun onWrite(id: Long) {
        memoSources[id]?.forEach { it.invalidate() }
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

    /**
     * A cached result of `compute`, recomputed on the first [get] after a state it read changes or a
     * full refresh. Each [get] registers the states the computation read as dependencies of the
     * reader, so a render reading a memo re-renders when those states change.
     */
    inner class Memo<T>(
        private val compute: () -> T,
    ) {
        private var stale = true
        private var value: T? = null
        private var reads: Set<Long> = emptySet()

        fun get(): T {
            if (stale) {
                val previous = capture
                val captured = HashSet<Long>()
                capture = captured
                try {
                    value = compute()
                } finally {
                    capture = previous
                }
                for (id in reads) memoSources[id]?.remove(this)
                reads = captured
                stale = false
                for (id in captured) memoSources.getOrPut(id) { HashSet() }.add(this)
            }
            reads.forEach(::registerRead)
            @Suppress("UNCHECKED_CAST")
            return value as T
        }

        fun invalidate() {
            stale = true
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
