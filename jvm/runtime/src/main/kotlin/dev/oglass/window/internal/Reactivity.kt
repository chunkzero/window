package dev.oglass.window.internal

import kotlin.properties.ReadWriteProperty
import kotlin.reflect.KProperty

/**
 * Per-session reactive engine.
 *
 * Tracks which slots read which [state] cells during their render lambda, marks dependent slots
 * dirty on writes, and coalesces the resulting re-renders into a single scheduled flush per burst.
 *
 * The owning session drives the engine: it pushes a slot name onto the rendering context before
 * invoking that slot's render lambda (so reads register a dependency), and supplies a [flush]
 * callback that recomputes dirty slots and re-sends the title.
 */
internal class Reactivity(
    private val scheduler: RenderScheduler,
    /**
     * Recomputes the given dirty slots and re-sends the title. Receives a snapshot, then clears.
     */
    private val flush: (dirty: Set<String>) -> Unit,
) {
    /** state cell id -> set of slot names that read it during render. */
    private val dependencies = HashMap<Long, MutableSet<String>>()

    /** Slots awaiting re-render. */
    private val dirty = LinkedHashSet<String>()

    /** Slot currently being rendered, if any; reads during this register a dependency. */
    private var renderingSlot: String? = null
    private var flushScheduled = false
    private var nextStateId = 0L

    /** Runs [block] with [slot] marked as the currently-rendering slot for dependency capture. */
    fun <T> withRendering(
        slot: String,
        block: () -> T,
    ): T {
        val previous = renderingSlot
        renderingSlot = slot
        try {
            return block()
        } finally {
            renderingSlot = previous
        }
    }

    /** Marks every named slot dirty and schedules a flush. Used by `refresh()`. */
    fun markAllDirty(slots: Collection<String>) {
        dirty.addAll(slots)
        scheduleFlush()
    }

    /** Creates a reactive state delegate bound to this engine. */
    fun <T> state(initial: T): ReadWriteProperty<Any?, T> = StateProperty(nextStateId++, initial)

    private fun registerRead(id: Long) {
        val slot = renderingSlot ?: return
        dependencies.getOrPut(id) { HashSet() }.add(slot)
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
