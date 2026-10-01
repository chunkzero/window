package dev.oglass.window.example

import dev.oglass.window.Click
import dev.oglass.window.IndexedClick
import dev.oglass.window.Windows
import dev.oglass.window.example.generated.CatalogSearchView
import net.kyori.adventure.text.Component
import net.minestom.server.item.ItemStack

/** Native-anvil search over the same catalog used by [MyShop]. */
class CatalogSearch(
    private val windows: Windows,
    private val market: Market,
) : CatalogSearchView() {
    private var query by state("")

    override fun queryText(): Component = Component.text(if (query.isEmpty()) "ITEM NAME" else query.uppercase())

    override fun resultCount(): Component = Component.text("${results().size} FOUND")

    override fun resultsItem(index: Int): ItemStack? = results().getOrNull(index)?.toItemStack()

    override fun onResults(click: IndexedClick) {
        val product = results().getOrNull(click.index) ?: return
        windows.open(player, MyShop(windows, market, product.id))
    }

    override fun onQueryChanged(value: String) {
        query = value.trim()
    }

    override fun onBack(click: Click) {
        windows.open(player, MyShop(windows, market))
    }

    override fun onReset(click: Click) {
        windows.open(player, CatalogSearch(windows, market))
    }

    private fun results(): List<Product> {
        if (query.isBlank()) return market.products
        return market.products.filter { product ->
            product.name.contains(query, ignoreCase = true) ||
                product.tier.label.contains(query, ignoreCase = true) ||
                product.category.name.contains(query, ignoreCase = true)
        }
    }
}
