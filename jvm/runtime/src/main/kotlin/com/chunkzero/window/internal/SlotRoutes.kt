package com.chunkzero.window.internal

import com.chunkzero.window.Click
import com.chunkzero.window.IndexedClick
import com.chunkzero.window.SlotArea
import com.chunkzero.window.SlotRef
import com.chunkzero.window.manifest.ButtonDefault
import com.chunkzero.window.manifest.SlotAreaEntry
import com.chunkzero.window.manifest.SlotRefEntry
import com.chunkzero.window.manifest.WindowEntry

/**
 * Routes clicks on typed backing slots to the button or collection cell that receives them.
 *
 * An action collection cell takes precedence over a button covering the same slot.
 */
internal class SlotRoutes(
    private val entry: WindowEntry,
    private val bindings: WindowBindings<*>,
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

    fun dispatch(click: Click) {
        when (val route = routes[click.slot] ?: return) {
            is Route.Button -> {
                clickButton(route.name, click)
            }

            is Route.Collection -> {
                val handler = bindings.collectionHandlers[route.name] ?: return
                handler(IndexedClick(click.slot, route.index, click.shift, click.right))
            }
        }
    }

    /** Invokes the bound handler, or applies the manifest default when none is bound. */
    private fun clickButton(
        name: String,
        click: Click,
    ) {
        val handler = bindings.buttonHandlers[name]
        if (handler != null) {
            handler(click)
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
