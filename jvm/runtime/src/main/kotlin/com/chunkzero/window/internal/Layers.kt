package com.chunkzero.window.internal

import com.chunkzero.window.manifest.HudEntry
import com.chunkzero.window.manifest.LayerEntry
import com.chunkzero.window.manifest.LayerKind
import com.chunkzero.window.manifest.WindowEntry

/** Checks that this window's layers list every slot, sprite slot, switch, and selectable collection once. */
internal fun WindowEntry.requireLayers(name: String) =
    requireLayers(
        "window '$name'",
        layers,
        mapOf(
            LayerKind.SLOT to slots.keys,
            LayerKind.SPRITE_SLOT to spriteSlots.keys,
            LayerKind.SWITCH to switches.keys,
            LayerKind.COLLECTION to collections.filterValues { it.selection.isNotEmpty() }.keys,
        ),
    )

/** Checks that this HUD's layers list every slot and switch once. */
internal fun HudEntry.requireLayers(name: String) =
    requireLayers("hud '$name'", layers, mapOf(LayerKind.SLOT to slots.keys, LayerKind.SWITCH to switches.keys))

private fun requireLayers(
    owner: String,
    layers: List<LayerEntry>,
    entries: Map<LayerKind, Set<String>>,
) {
    val expected = entries.flatMap { (kind, names) -> names.map { LayerEntry(kind, it) } }.toSet()
    val listed = layers.toSet()
    val missing = expected - listed
    val unexpected = listed - expected
    val duplicated =
        layers
            .groupingBy { it }
            .eachCount()
            .filterValues { it > 1 }
            .keys
    require(missing.isEmpty() && unexpected.isEmpty() && duplicated.isEmpty()) {
        fun describe(layers: Set<LayerEntry>) = layers.map { "${it.kind.name.lowercase()} '${it.name}'" }
        "The layers of $owner must list each of its slots, sprite slots, switches, and selectable collections " +
            "once; missing ${describe(missing)}, unexpected ${describe(unexpected)}, " +
            "duplicated ${describe(duplicated)}"
    }
}
