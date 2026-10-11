package com.chunkzero.window.example

import com.chunkzero.window.Click
import com.chunkzero.window.IndexedClick
import com.chunkzero.window.example.generated.UpgradesView
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.minestom.server.entity.Player

/** The workshop: upgrade cards bought with the [market]'s coins, in the amount the player chose. */
class MyUpgrades(
    player: Player,
    private val market: Market,
) : UpgradesView(player) {
    private var coins by state(0)
    private var levels by state(market.workshop.levelsOf(player))
    private var feedback by state<String?>(null)

    override fun onOpen() {
        coins = market.balanceOf(player)
    }

    override fun balance(): Component = Component.text(coins)

    override fun upgradeName(index: Int): Component = Component.text(Upgrade.entries[index].title)

    override fun upgradeLevel(index: Int): Component = Component.text("Lv ${levels[index]}/${Upgrade.MAX_LEVEL}")

    override fun upgradeProgress(index: Int): Float = levels[index].toFloat() / Upgrade.MAX_LEVEL

    override fun upgradeStage(index: Int): UpgradeStage =
        when (levels[index]) {
            0 -> UpgradeStage.NEW
            Upgrade.MAX_LEVEL -> UpgradeStage.MAXED
            else -> UpgradeStage.OPEN
        }

    /** The offer's price, red while the balance does not cover it. */
    override fun upgradeCost(index: Int): Component {
        val offer = offer(index) ?: return Component.empty()
        return if (offer.price <=
            coins
        ) {
            Component.text(offer.price)
        } else {
            Component.text(offer.price, NamedTextColor.RED)
        }
    }

    override fun onUpgrade(click: IndexedClick) {
        val upgrade = Upgrade.entries[click.index]
        val offer = offer(click.index) ?: return
        val newBalance = market.purchase(player, offer.price)
        if (newBalance == null) {
            feedback = "Need ${offer.price - coins} more"
        } else {
            coins = newBalance
            levels = market.workshop.raise(player, upgrade, offer.count)
            feedback = "Bought ${upgrade.title} x${offer.count}"
        }
    }

    override fun onAmountChanged(value: Amount) {
        feedback = null
    }

    override fun rankSprite(): RankSprite = RankSprite.entries[rank()]

    override fun rankName(): Component = Component.text(RANKS[rank()])

    override fun status(): Component = Component.text(feedback ?: "${levels.sum()} of $TOTAL_LEVELS levels")

    /** Whether any offer is affordable, or warn once everything is maxed, unless `/tone` pinned a tone. */
    override fun tone(): Tone {
        val pinned = pinnedTone(player)
        if (pinned != null) return Tone.entries.first { it.value == pinned }
        val offers = Upgrade.entries.indices.mapNotNull(::offer)
        return when {
            offers.isEmpty() -> Tone.WARN
            offers.any { it.price <= coins } -> Tone.OK
            else -> Tone.DANGER
        }
    }

    override fun onOpenShop(click: Click) {
        MyShop(player, market).open()
    }

    /** One rank per [Upgrade.MAX_LEVEL] levels bought. */
    private fun rank(): Int = minOf(levels.sum() / Upgrade.MAX_LEVEL, RANKS.lastIndex)

    /**
     * What a click on upgrade [index] buys, or `null` when it is maxed. `Max` buys as many levels as the balance
     * covers, and prices one level when it covers none.
     */
    private fun offer(index: Int): Offer? {
        val upgrade = Upgrade.entries[index]
        val level = levels[index]
        val left = Upgrade.MAX_LEVEL - level
        if (left == 0) return null
        var count =
            when (amount) {
                Amount.ONE -> 1
                Amount.FIVE -> minOf(5, left)
                Amount.MAX -> left
            }
        if (amount == Amount.MAX) {
            while (count > 1 && upgrade.cost(level, count) > coins) count--
        }
        return Offer(count, upgrade.cost(level, count))
    }

    private data class Offer(
        val count: Int,
        val price: Int,
    )
}

private val RANKS = listOf("Apprentice", "Journeyman", "Master")
private val TOTAL_LEVELS = Upgrade.entries.size * Upgrade.MAX_LEVEL
