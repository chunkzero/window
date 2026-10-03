package dev.oglass.window.internal

/**
 * Keeps at most one anvil reopen in flight.
 *
 * Changing an anvil's title reopens the vanilla screen, which resets its edit box to the input
 * seed's name, and the client then sends that name back as input. A second reopen sent before that
 * echo arrives would restore stale text that the client echoes again, cycling between old values.
 * So a title change waits until the in-flight reopen is echoed, or [TIMEOUT_TICKS] pass without
 * one, and the echo itself is not a user edit.
 */
internal class AnvilReopenGate(
    private val scheduler: RenderScheduler,
    /** The edit-box text the next reopen restores. */
    var seed: String,
    private val reopen: () -> Unit,
) {
    private var awaiting: String? = null
    private var deferred = false
    private var generation = 0

    /** True when a reopen may be sent now; otherwise it is deferred until the in-flight one settles. */
    fun canReopen(): Boolean {
        if (awaiting == null) return true
        deferred = true
        return false
    }

    /** Records that a reopen restoring [seed] was sent. */
    fun sent() {
        awaiting = seed
        countdown(++generation, TIMEOUT_TICKS)
    }

    /** Returns true, settling the in-flight reopen, when [value] is its echo. */
    fun consumeEcho(value: String): Boolean {
        if (awaiting == null || value != awaiting) return false
        settle()
        return true
    }

    private fun countdown(
        token: Int,
        ticks: Int,
    ) {
        scheduler.schedule {
            if (token != generation) return@schedule
            if (ticks > 1) countdown(token, ticks - 1) else settle()
        }
    }

    private fun settle() {
        awaiting = null
        generation++
        if (deferred) {
            deferred = false
            reopen()
        }
    }

    private companion object {
        const val TIMEOUT_TICKS = 20
    }
}
