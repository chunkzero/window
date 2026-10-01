package dev.oglass.window.internal

import net.minestom.server.inventory.InventoryType

/** Maps window-core container kind ids to Minestom [InventoryType]s. */
internal object Containers {
    private val byKind =
        mapOf(
            "generic_9x1" to InventoryType.CHEST_1_ROW,
            "generic_9x2" to InventoryType.CHEST_2_ROW,
            "generic_9x3" to InventoryType.CHEST_3_ROW,
            "generic_9x4" to InventoryType.CHEST_4_ROW,
            "generic_9x5" to InventoryType.CHEST_5_ROW,
            "generic_9x6" to InventoryType.CHEST_6_ROW,
            "anvil" to InventoryType.ANVIL,
        )

    /**
     * Resolves a container kind id to its [InventoryType].
     *
     * @throws IllegalArgumentException if the kind is not a supported container.
     */
    fun inventoryType(container: String): InventoryType =
        byKind[container]
            ?: throw IllegalArgumentException(
                "Unsupported container kind '$container'; supported: ${byKind.keys.sorted()}",
            )
}
