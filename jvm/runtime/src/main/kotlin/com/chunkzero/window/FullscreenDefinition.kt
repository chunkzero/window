package com.chunkzero.window

import com.chunkzero.window.manifest.FullscreenCanvasEntry
import com.chunkzero.window.manifest.FullscreenEntry
import com.chunkzero.window.manifest.FullscreenSurfaceEntry
import com.chunkzero.window.manifest.WindowManifest

/**
 * The full-screen window [name] of [manifest], with the canvas contract its pack shares.
 *
 * @throws IllegalArgumentException if [manifest] has no full-screen window [name] (listing the known names).
 */
public class FullscreenDefinition(
    internal val manifest: WindowManifest,
    /** The full-screen window name. */
    public val name: String,
) {
    internal val entry: FullscreenEntry =
        manifest.fullscreens[name]
            ?: throw IllegalArgumentException(
                "Unknown fullscreen '$name'; known fullscreens: ${manifest.fullscreens.keys.sorted()}",
            )

    /** The canvas contract shared by every full-screen window of the pack. */
    public val canvas: FullscreenCanvasEntry =
        requireNotNull(manifest.fullscreenCanvas) { "The manifest has fullscreen '$name' but no fullscreen canvas" }

    /** Surface metadata for this full-screen window. */
    public val surface: FullscreenSurfaceEntry
        get() = entry.surface
}
