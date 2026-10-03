package dev.oglass.window.internal

import java.util.concurrent.atomic.AtomicInteger

/**
 * Separates anvil reopen echoes from user edits and keeps at most one reopen in flight.
 *
 * Changing an anvil's title reopens the vanilla screen, which resets its edit box to the input
 * seed's name; the client then sends that name back as input. Each reopen is followed by a ping, so
 * the client's pong marks where the echo ends: input received before the pong is buffered, its
 * last value is the echo when it matches the seed the reopen restored, and the rest are edits typed
 * on the old screen. Those edits are delivered on the pong, and since the reopen wiped them from the
 * client's edit box, another reopen restores them. Title changes requested while a reopen is in
 * flight wait for its pong.
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
    private val buffered = mutableListOf<String>()
    private var deferred = false
    private var owed = false

    /** True when a reopen may be sent now; otherwise it is deferred until the in-flight one settles. */
    fun canReopen(): Boolean {
        if (awaiting == null) return true
        deferred = true
        return false
    }

    /** Records that a reopen restoring [seed] was sent, and pings behind it. */
    fun sent() {
        restored = seed
        owed = false
        val id = PINGS.getAndIncrement()
        awaiting = id
        ping(id)
    }

    /** Handles client edit-box [value], buffering it while a reopen is in flight. */
    fun input(value: String) {
        if (awaiting != null) buffered += value else apply(value)
    }

    /** Settles the in-flight reopen when [id] answers its ping. */
    fun pong(id: Int) {
        if (id != awaiting) return
        awaiting = null
        val edits = if (buffered.lastOrNull() == restored) buffered.dropLast(1) else buffered.toList()
        buffered.clear()
        edits.forEach(::apply)
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
