package dev.oglass.window.example

import net.minestom.server.entity.Player
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

/** Small in-memory catalog and wallet used by the visual example. */
class Market(
    val unitPrice: Int = 10,
    private val startingBalance: Int = 2_500,
) {
    val products: List<Product> = MARKET_CATALOG

    private val balances = ConcurrentHashMap<UUID, Int>()

    fun balanceOf(player: Player): Int = balances.computeIfAbsent(player.uuid) { startingBalance }

    fun purchase(
        player: Player,
        price: Int,
    ): Int? {
        var purchased = false
        val balance =
            balances.compute(player.uuid) { _, current ->
                val available = current ?: startingBalance
                if (available >= price) {
                    purchased = true
                    available - price
                } else {
                    available
                }
            }!!
        return balance.takeIf { purchased }
    }
}
