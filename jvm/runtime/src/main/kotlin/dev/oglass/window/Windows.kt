package dev.oglass.window

import dev.oglass.window.internal.Containers
import dev.oglass.window.internal.LiveInventoryHandle
import dev.oglass.window.internal.RenderScheduler
import dev.oglass.window.manifest.WindowManifest
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import java.nio.file.Files
import java.nio.file.Path

/**
 * Registry of windows loaded from a compiled Window pack definition.
 *
 * Load once at startup via [load], then [open] a [WindowView] for a player. Window definitions are
 * resolved by name with [get].
 */
public class Windows
    private constructor(
        private val manifest: WindowManifest,
        private val diagnosticsObserver: RenderDiagnosticsObserver,
    ) {
        private val definitions: Map<String, WindowDefinition> =
            manifest.windows.mapValues { (name, entry) -> WindowDefinition(name, manifest, entry) }
        private val hudDefinitions: Map<String, HudDefinition> =
            manifest.huds.mapValues { (name, entry) -> HudDefinition(name, manifest, entry) }

        public companion object {
            /** Loads a registry from a legacy JSON definition file at [path]. */
            public fun load(path: Path): Windows = load(Files.readString(path))

            /** Loads a registry from a legacy JSON definition document. */
            public fun load(json: String): Windows = Windows(WindowManifest.parse(json), RenderDiagnosticsObserver.NONE)

            /** Loads a registry from a compiled Window pack definition. */
            public fun load(manifest: WindowManifest): Windows = Windows(manifest, RenderDiagnosticsObserver.NONE)

            /** Loads a registry and enables immutable render-frame observation. */
            @JvmStatic
            public fun load(
                manifest: WindowManifest,
                diagnosticsObserver: RenderDiagnosticsObserver,
            ): Windows = Windows(manifest, diagnosticsObserver)
        }

        /**
         * Returns the [WindowDefinition] named [name].
         *
         * @throws IllegalArgumentException if no such window exists (listing the known names).
         */
        public operator fun get(name: String): WindowDefinition =
            definitions[name]
                ?: throw IllegalArgumentException(
                    "Unknown window '$name'; known windows: ${definitions.keys.sorted()}",
                )

        /**
         * Returns the [HudDefinition] named [name].
         *
         * @throws IllegalArgumentException if no such HUD exists (listing the known names).
         */
        public fun hud(name: String): HudDefinition =
            hudDefinitions[name]
                ?: throw IllegalArgumentException(
                    "Unknown hud '$name'; known huds: ${hudDefinitions.keys.sorted()}",
                )

        /**
         * Opens [view]'s window for [player], building the inventory, composing the title, running the
         * view's bindings, and registering listeners. Returns the live [WindowSession].
         *
         * @throws IllegalArgumentException for unknown slot/button names referenced in the view.
         * @throws IllegalStateException for unbound dynamic slots or unhandled buttons.
         */
        public fun open(
            player: Player,
            view: WindowView,
        ): WindowSession {
            val definition = get(view.windowName)
            val type = Containers.inventoryType(definition.surface.container)
            val handle = LiveInventoryHandle(player, type)
            val session =
                windowSession(
                    definition,
                    view,
                    player,
                    RenderScheduler.NEXT_TICK,
                    handle,
                    diagnosticsObserver,
                )
            session.open()
            return session
        }

        /**
         * Shows [view]'s HUD for [player] on its manifest fallback channel.
         *
         * @throws IllegalArgumentException for unknown slot names referenced in the view.
         * @throws IllegalStateException for unbound dynamic slots.
         */
        public fun show(
            player: Player,
            view: HudView,
        ): HudSession {
            val definition = hud(view.hudName)
            val session =
                HudSession(definition, view, player, RenderScheduler.NEXT_TICK, diagnosticsObserver)
            session.show()
            return session
        }

        /**
         * Renders [view]'s initial HUD component without sending it to the fallback channel.
         *
         * Intended for diagnostics and previews; the supplied [view] is attached and should not be
         * reused with [show].
         */
        public fun renderHud(
            player: Player,
            view: HudView,
        ): Component {
            val definition = hud(view.hudName)
            val session = HudSession(definition, view, player, RenderScheduler.NEXT_TICK)
            return session.composeInitial()
        }
    }
