package com.chunkzero.window

import com.chunkzero.window.internal.Containers
import com.chunkzero.window.internal.TitleComposer
import com.chunkzero.window.internal.requireLayers
import com.chunkzero.window.manifest.SurfaceEntry
import com.chunkzero.window.manifest.WindowEntry
import com.chunkzero.window.manifest.WindowManifest
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component

/**
 * The window [name] of [manifest], combined with the manifest-level spacer and advance tables.
 *
 * Exposes the window [name], its [surface] metadata, and the precomputed [staticTitle] component
 * (the baked chrome with the main font applied, before any dynamic slot segments).
 *
 * @throws IllegalArgumentException if [manifest] has no window [name] (listing the known names), or
 *   its container kind is not supported, or its layers do not list each slot, sprite slot, switch, and
 *   selectable collection exactly once.
 */
public class WindowDefinition(
    internal val manifest: WindowManifest,
    /** The window name. */
    public val name: String,
) {
    internal val entry: WindowEntry =
        manifest.windows[name]
            ?: throw IllegalArgumentException(
                "Unknown window '$name'; known windows: ${manifest.windows.keys.sorted()}",
            )

    init {
        entry.requireLayers(name)
    }

    internal val kind = Containers.kind(entry.surface.container)

    internal val composer: TitleComposer = TitleComposer(manifest, entry)

    /** The item model of this window's invisible hitbox items. */
    internal val hitboxModel: Key = Key.key("${manifest.namespace}:gui/hitbox")

    /** Surface (container) metadata for this window. */
    public val surface: SurfaceEntry
        get() = entry.surface

    /** The static-chrome title component, with the main font applied and no slot segments. */
    public val staticTitle: Component
        get() = composer.staticComponent
}
