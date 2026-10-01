package dev.oglass.window

import dev.oglass.window.diagnostics.RenderFrame
import net.minestom.server.entity.Player

/**
 * Narrow observation hook for immutable expectations emitted by Window's real composition pass.
 *
 * Observers cannot alter the component sent to Minecraft. Runtime sessions isolate observer
 * failures so enabling diagnostics never changes normal window or HUD behavior.
 */
public fun interface RenderDiagnosticsObserver {
    /** Observes one composed window title or HUD frame for [player]. */
    public fun observe(
        player: Player,
        frame: RenderFrame,
    )

    public companion object {
        /** Observer used by normal runtime operation. */
        @JvmField public val NONE: RenderDiagnosticsObserver = RenderDiagnosticsObserver { _, _ -> }
    }
}
