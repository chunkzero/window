package com.chunkzero.window.example

import net.minestom.server.entity.Player
import net.minestom.server.item.Material
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

/** A foundry upgrade; each level costs [baseCost] more than the one before. */
enum class Upgrade(
    val title: String,
    val material: Material,
    private val baseCost: Int,
) {
    DRILL("Drill", Material.PISTON, 60),
    FURNACE("Furnace", Material.BLAST_FURNACE, 90),
    CONVEYOR("Conveyor", Material.POWERED_RAIL, 120),
    ;

    /** What the next [count] levels above [level] cost together. */
    fun cost(
        level: Int,
        count: Int,
    ): Int = (1..count).sumOf { baseCost * (level + it) }

    companion object {
        const val MAX_LEVEL = 5
    }
}

/** Each player's upgrade levels, kept for the server's lifetime. */
class Workshop {
    private val levels = ConcurrentHashMap<UUID, List<Int>>()

    /** The level of every [Upgrade], in declaration order. */
    fun levelsOf(player: Player): List<Int> = levels[player.uuid] ?: Upgrade.entries.map { 0 }

    fun raise(
        player: Player,
        upgrade: Upgrade,
        count: Int,
    ): List<Int> {
        val raised = levelsOf(player).toMutableList()
        raised[upgrade.ordinal] = (raised[upgrade.ordinal] + count).coerceAtMost(Upgrade.MAX_LEVEL)
        levels[player.uuid] = raised
        return raised
    }
}
