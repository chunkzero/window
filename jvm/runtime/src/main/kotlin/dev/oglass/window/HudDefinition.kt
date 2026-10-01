package dev.oglass.window

import dev.oglass.window.internal.HudComposer
import dev.oglass.window.manifest.HudEntry
import dev.oglass.window.manifest.HudSurfaceEntry
import dev.oglass.window.manifest.WindowManifest
import net.kyori.adventure.text.Component

/** A resolved HUD: its [HudEntry] combined with the manifest-level spacer and advance tables. */
public class HudDefinition
    internal constructor(
        /** The HUD name (registry key). */
        public val name: String,
        internal val manifest: WindowManifest,
        internal val entry: HudEntry,
    ) {
        internal val composer: HudComposer = HudComposer(manifest, entry)

        /** Surface/channel metadata for this HUD. */
        public val surface: HudSurfaceEntry
            get() = entry.surface

        /** The baked static HUD component, with no slot segments. */
        public val staticHud: Component
            get() = composer.staticComponent
    }
