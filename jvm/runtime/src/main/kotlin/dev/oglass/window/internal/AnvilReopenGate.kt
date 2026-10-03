package dev.oglass.window.internal

import java.util.concurrent.atomic.AtomicInteger

/**
 * Separates anvil reopen echoes from user edits and keeps at most one reopen in flight.
 *
 * Changing an anvil's title reopens the vanilla screen, which resets its edit box to the input
 * seed's name; the client then sends that name back as input. Each reopen is bundled with a ping,
 * so the client handles both at once and its echo arrives immediately before the pong. While a
 * reopen is in flight the latest input is held back: any later packet proves it was a user edit,
 * and the pong proves it was the echo when it matches the restored seed. Edits typed on the old
 * screen were wiped from the client's edit box, so another reopen restores them. Title changes
 * requested while a reopen is in flight wait for its pong.
 */
internal class AnvilReopenGate(
    private val scheduler: RenderScheduler,
    seed: String,
    private val ping: (Int) -> Unit,
    private val deliver: (String) -> Unit,
    private val reopen: () -> Unit,
) {
    /** The edit-box text the next reopen restores. */
    var seed: String = seed
        private set
    private var restored = seed
    private var awaiting: Int? = null
    private var held: String? = null
    private var deferred = false
    private var owed = false

    /** True when a reopen may be sent now; otherwise it is deferred until the in-flight one settles. */
    fun canReopen(): Boolean {
        if (awaiting == null) return true
        deferred = true
        return false
    }

    /** Records that a reopen restoring [seed] was sent; call inside the reopen's bundle. */
    fun sent() {
        restored = seed
        owed = false
        val id = PINGS.getAndIncrement()
        awaiting = id
        ping(id)
    }

    /** Handles client edit-box [value]. */
    fun input(value: String) {
        if (awaiting == null) {
            apply(value)
        } else {
            release()
            held = value
        }
    }

    /** Delivers held input before another client packet is handled, since that packet proves it was an edit. */
    fun release() {
        val value = held ?: return
        held = null
        apply(value)
    }

    /** Settles the in-flight reopen when [id] answers its ping. */
    fun pong(id: Int) {
        if (id != awaiting) return
        awaiting = null
        val last = held
        held = null
        if (last != null && last != restored) apply(last)
        if (deferred || seed != restored) {
            deferred = false
            owed = true
            // Runs after the reactive flush the edits scheduled, which usually sends the reopen.
            scheduler.schedule { if (owed && awaiting == null) reopen() }
        }
    }

    private fun apply(value: String) {
        seed = value
        deliver(value)
    }

    private companion object {
        /** Ping ids start in a range unlikely to collide with other users of the ping packet. */
        val PINGS = AtomicInteger(0x57_49_00_00)
    }
}
