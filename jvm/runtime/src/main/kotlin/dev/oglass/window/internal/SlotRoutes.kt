package dev.oglass.window.internal

import dev.oglass.window.Click
import dev.oglass.window.IndexedClick
import dev.oglass.window.SlotArea
import dev.oglass.window.SlotRef
import dev.oglass.window.manifest.ButtonDefault
import dev.oglass.window.manifest.SlotAreaEntry
import dev.oglass.window.manifest.SlotRefEntry
import dev.oglass.window.manifest.WindowEntry
import net.minestom.server.entity.Player

/**
 * Routes clicks on typed backing slots to the button or collection cell that receives them.
 *
 * An action collection cell takes precedence over a button covering the same slot.
 */
internal class SlotRoutes(
    private val entry: WindowEntry,
    private val bindings: WindowBindings,
    private val player: Player,
    /** Applies a `close` button default. */
    private val close: () -> Unit,
) {
    private val routes: Map<SlotRef, Route> =
        buildMap {
            for ((name, button) in entry.buttons) {
                for (slot in button.slots) put(slot.toApi(), Route.Button(name))
            }
            for ((name, collection) in entry.collections) {
                if (!collection.action) continue
                for ((index, slot) in collection.slots.withIndex()) {
                    put(slot.toApi(), Route.Collection(name, index))
                }
            }
        }

    fun dispatch(info: ClickInfo) {
        when (val route = routes[info.slot] ?: return) {
            is Route.Button -> {
                clickButton(route.name, info)
            }

            is Route.Collection -> {
                val handler = bindings.collectionHandlers[route.name] ?: return
                handler(IndexedClick(player, info.slot, route.index, info.shift, info.right))
            }
        }
    }

    /** Invokes the bound handler, or applies the manifest default when none is bound. */
    private fun clickButton(
        name: String,
        info: ClickInfo,
    ) {
        val handler = bindings.buttonHandlers[name]
        if (handler != null) {
            handler(Click(player, info.slot, info.shift, info.right))
        } else if (entry.buttons.getValue(name).default == ButtonDefault.CLOSE) {
            close()
        }
    }

    private sealed interface Route {
        data class Button(
            val name: String,
        ) : Route

        data class Collection(
            val name: String,
            val index: Int,
        ) : Route
    }
}

internal fun SlotRefEntry.toApi(): SlotRef =
    SlotRef(
        when (area) {
            SlotAreaEntry.CONTAINER -> SlotArea.CONTAINER
            SlotAreaEntry.PLAYER -> SlotArea.PLAYER
        },
        index,
    )
