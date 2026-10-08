package com.chunkzero.window.internal

import com.chunkzero.window.Click
import com.chunkzero.window.IndexedClick
import com.chunkzero.window.SlotArea
import com.chunkzero.window.SlotRef
import com.chunkzero.window.manifest.SlotAreaEntry
import com.chunkzero.window.manifest.SlotRefEntry
import com.chunkzero.window.manifest.WindowEntry

/**
 * Routes clicks on typed backing slots to the active region or the collection cell that receives them.
 *
 * An action collection cell takes precedence over a region covering the same slot. Regions and collections in
 * mutually exclusive cases may share slots; a click routes to the one whose case is active. Each switch's case is
 * read at click time, from its state provider or binding, so a click between a change and the next render routes
 * by the current case.
 */
internal class SlotRoutes(
    private val entry: WindowEntry,
    private val bindings: WindowBindings<*>,
    /** Runs a region's runtime action when no handler is bound to its action. */
    private val run: (RuntimeAction) -> Unit,
) {
    private val regions: Map<SlotRef, List<String>> =
        buildMap<SlotRef, MutableList<String>> {
            for ((name, region) in entry.regions) {
                if (region.action == null) continue
                for (slot in region.slots) getOrPut(slot.toApi(), ::ArrayList) += name
            }
        }

    private val cells: Map<SlotRef, List<Cell>> =
        buildMap<SlotRef, MutableList<Cell>> {
            for ((name, collection) in entry.collections) {
                if (!collection.action) continue
                for ((index, slot) in collection.slots.withIndex()) {
                    getOrPut(slot.toApi(), ::ArrayList) +=
                        Cell(name, index)
                }
            }
        }

    fun dispatch(click: Click) {
        val switches = bindings.switches
        val current = switches.current { key -> bindings.buttonStates[key]?.invoke() }
        val cell = cells[click.slot]?.firstOrNull { switches.collectionActive(it.name, current) }
        if (cell != null) {
            val handler = bindings.collectionHandlers[cell.name] ?: return
            handler(IndexedClick(click.slot, cell.index, click.shift, click.right))
            return
        }
        val name = regions[click.slot]?.firstOrNull { switches.regionActive(it, current) } ?: return
        val region = entry.regions.getValue(name)
        val handler = bindings.buttonHandlers[region.action]
        if (handler != null) {
            handler(click)
        } else {
            region.defaultAction?.let(RuntimeAction::of)?.let(run)
        }
    }

    private data class Cell(
        val name: String,
        val index: Int,
    )
}

internal fun SlotRefEntry.toApi(): SlotRef =
    SlotRef(
        when (area) {
            SlotAreaEntry.CONTAINER -> SlotArea.CONTAINER
            SlotAreaEntry.PLAYER -> SlotArea.PLAYER
        },
        index,
    )
