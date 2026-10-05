package com.chunkzero.window

import com.chunkzero.window.internal.TitleComposer
import com.chunkzero.window.manifest.SurfaceEntry
import com.chunkzero.window.manifest.WindowEntry
import com.chunkzero.window.manifest.WindowManifest
import net.kyori.adventure.text.Component

/**
 * A resolved window: its [WindowEntry] combined with the manifest-level spacer and advance tables.
 *
 * Exposes the window [name], its [surface] metadata, and the precomputed [staticTitle] component
 * (the baked chrome with the main font applied, before any dynamic slot segments).
 */
public class WindowDefinition
    internal constructor(
        /** The window name (registry key). */
        public val name: String,
        internal val manifest: WindowManifest,
        internal val entry: WindowEntry,
    ) {
        internal val composer: TitleComposer = TitleComposer(manifest, entry)

        /** Surface (container) metadata for this window. */
        public val surface: SurfaceEntry
            get() = entry.surface

        /** The static-chrome title component, with the main font applied and no slot segments. */
        public val staticTitle: Component
            get() = composer.staticComponent
    }
