package com.chunkzero.window

import com.chunkzero.window.diagnostics.RenderFrame
import com.chunkzero.window.host.WindowHost

/**
 * Narrow observation hook for immutable expectations emitted by Window's real composition pass.
 *
 * Observers cannot alter the component sent to Minecraft. Runtime sessions isolate observer
 * failures so enabling diagnostics never changes normal window or HUD behavior.
 */
public fun interface RenderDiagnosticsObserver {
    /** Observes one composed window title or HUD frame shown through [host]. */
    public fun observe(
        host: WindowHost<*>,
        frame: RenderFrame,
    )

    public companion object {
        /** Observer used by normal runtime operation. */
        @JvmField public val NONE: RenderDiagnosticsObserver = RenderDiagnosticsObserver { _, _ -> }
    }
}

/** Global diagnostics registration for every window and HUD session. */
public object WindowDiagnostics {
    /** The observer every session reports its frames to; [RenderDiagnosticsObserver.NONE] by default. */
    @Volatile
    @JvmStatic
    public var observer: RenderDiagnosticsObserver = RenderDiagnosticsObserver.NONE
}
