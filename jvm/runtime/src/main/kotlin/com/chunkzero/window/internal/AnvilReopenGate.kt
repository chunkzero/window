package com.chunkzero.window.internal

import java.util.concurrent.atomic.AtomicInteger

/**
 * Separates anvil reopen echoes from user edits, keeps at most one reopen in flight, and holds
 * reopens while the player types.
 *
 * Changing an anvil's title reopens the vanilla screen, which resets its edit box to the input
 * seed's name; the client then sends that name back as input. Each reopen is bundled with a ping,
 * so the client handles both at once and its echo arrives immediately before the pong. While a
 * reopen is in flight the latest input is held back: any later packet proves it was a user edit,
 * and the pong proves it was the echo when it matches the restored seed.
 *
 * The client sends its whole edit box, so the gate tracks the box separately from the seed. Once a
 * reopen lands, the box holds the restored seed even when later edits moved the seed on; edits
 * typed on that box are rebased onto the seed, and another reopen restores it. Title changes wait
 * for the in-flight reopen's pong and for [QUIET_TICKS] scheduler ticks without input.
 */
internal class AnvilReopenGate(
    private val scheduler: RenderScheduler,
    private val handle: InventoryHandle,
    initial: String,
    private val deliver: (String) -> Unit,
    private val reopen: () -> Unit,
) {
    /** The edit-box text the next reopen restores. */
    private var seed = initial

    /** The client's edit-box text as of the latest input. */
    private var box = initial
    private var restored = initial
    private var awaiting: Int? = null
    private var held: String? = null
    private var deferred = false
    private var quiet = 0

    /**
     * Sends a reopen: runs [action] in a bundle with a ping and returns true. While another reopen is
     * in flight or the player is typing, defers the title change and returns false without running
     * [action].
     */
    fun send(action: () -> Unit): Boolean {
        if (awaiting != null || quiet > 0) {
            deferred = true
            return false
        }
        handle.bundle {
            action()
            restored = seed
            deferred = false
            val id = PINGS.getAndIncrement()
            awaiting = id
            handle.ping(id)
        }
        return true
    }

    /** Handles client edit-box [value]. */
    fun input(value: String) {
        if (quiet == 0) scheduler.schedule(::countDown)
        quiet = QUIET_TICKS
        if (awaiting == null) {
            edit(value)
        } else {
            release()
            held = value
        }
    }

    /** Replaces the input with [value] and delivers it; a reopen restores it into the edit box. */
    fun replace(value: String) {
        release()
        val next = value.take(MAX_NAME_LENGTH)
        if (next == seed) return
        seed = next
        deliver(seed)
        deferred = true
        scheduler.schedule(::reopenIfDeferred)
    }

    /** Delivers held input before another client packet is handled, since that packet proves it was an edit. */
    fun release() {
        val value = held ?: return
        held = null
        edit(value)
    }

    /** Settles the in-flight reopen when [id] answers its ping. */
    fun pong(id: Int) {
        if (id != awaiting) return
        awaiting = null
        val last = held
        held = null
        if (last != null && last != restored) edit(last)
        box = restored
        if (seed != restored) deferred = true
        // Runs after the reactive flush the edits scheduled, which usually sends the reopen.
        scheduler.schedule(::reopenIfDeferred)
    }

    private fun countDown() {
        if (--quiet > 0) {
            scheduler.schedule(::countDown)
        } else {
            reopenIfDeferred()
        }
    }

    private fun reopenIfDeferred() {
        if (deferred && awaiting == null && quiet == 0) reopen()
    }

    private fun edit(value: String) {
        seed = rebase(box, value, seed).take(MAX_NAME_LENGTH)
        box = value
        deliver(seed)
    }

    internal companion object {
        /** Scheduler ticks without input before a held title change reopens the anvil. */
        const val QUIET_TICKS = 3

        /** The longest name vanilla's anvil edit box accepts. */
        const val MAX_NAME_LENGTH = 50

        /** Ping ids start in a range unlikely to collide with other users of the ping packet. */
        private val PINGS = AtomicInteger(0x57_49_00_00)

        /**
         * Applies the edit that turned [from] into [to] onto [target], locating it through the
         * characters [from] and [target] share. Deletions remove the shared characters they touched;
         * insertions land just before the next shared character, so after text [from] lost there.
         */
        fun rebase(
            from: String,
            to: String,
            target: String,
        ): String {
            val prefix = from.commonPrefixWith(to).length
            val suffix = from.substring(prefix).commonSuffixWith(to.substring(prefix)).length
            val end = from.length - suffix
            val inserted = to.substring(prefix, to.length - suffix)
            val matches = alignment(from, target)
            val deleted = (prefix until end).map { matches[it] }.filter { it >= 0 }
            val (start, stop) =
                if (deleted.isEmpty()) {
                    val next = (end until from.length).map { matches[it] }.firstOrNull { it >= 0 } ?: target.length
                    next to next
                } else {
                    deleted.first() to deleted.last() + 1
                }
            return target.substring(0, start) + inserted + target.substring(stop)
        }

        /** For each index of [a], the index of the matching character of [b] in a longest common subsequence, or -1. */
        private fun alignment(
            a: String,
            b: String,
        ): IntArray {
            val lengths = Array(a.length + 1) { IntArray(b.length + 1) }
            for (i in a.indices.reversed()) {
                for (j in b.indices.reversed()) {
                    lengths[i][j] =
                        if (a[i] == b[j]) lengths[i + 1][j + 1] + 1 else maxOf(lengths[i + 1][j], lengths[i][j + 1])
                }
            }
            val matches = IntArray(a.length) { -1 }
            var i = 0
            var j = 0
            while (i < a.length && j < b.length) {
                when {
                    a[i] == b[j] -> matches[i++] = j++
                    lengths[i + 1][j] >= lengths[i][j + 1] -> i++
                    else -> j++
                }
            }
            return matches
        }
    }
}
