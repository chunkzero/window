package com.chunkzero.window

import com.chunkzero.window.internal.Containers
import com.chunkzero.window.internal.HudComposer
import com.chunkzero.window.internal.requireLayers
import com.chunkzero.window.manifest.HudEntry
import com.chunkzero.window.manifest.HudSurfaceEntry
import com.chunkzero.window.manifest.WindowManifest
import net.kyori.adventure.text.Component

/**
 * The HUD [name] of [manifest], combined with the manifest-level spacer and advance tables.
 *
 * @throws IllegalArgumentException if [manifest] has no HUD [name] (listing the known names), or its
 *   channel is not supported, or its layers do not list each slot and switch exactly once.
 */
public class HudDefinition(
    internal val manifest: WindowManifest,
    /** The HUD name. */
    public val name: String,
) {
    internal val entry: HudEntry =
        manifest.huds[name]
            ?: throw IllegalArgumentException("Unknown hud '$name'; known huds: ${manifest.huds.keys.sorted()}")

    init {
        entry.requireLayers(name)
    }

    internal val channel = Containers.channel(entry.surface.channel)

    internal val composer: HudComposer = HudComposer(manifest, entry)

    /** Surface/channel metadata for this HUD. */
    public val surface: HudSurfaceEntry
        get() = entry.surface

    /** The baked static HUD component, with no slot segments. */
    public val staticHud: Component
        get() = composer.staticComponent
}
