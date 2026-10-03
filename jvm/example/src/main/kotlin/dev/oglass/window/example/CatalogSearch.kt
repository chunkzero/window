package dev.oglass.window.example

import dev.oglass.window.Click
import dev.oglass.window.IndexedClick
import dev.oglass.window.Windows
import dev.oglass.window.example.generated.CatalogSearchView
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.TextColor
import net.minestom.server.item.ItemStack

/** Native-anvil search over the same catalog used by [MyShop]. */
class CatalogSearch(
    private val windows: Windows,
    private val market: Market,
) : CatalogSearchView() {
    private var input by state("")
    private val query get() = input.trim()

    /** Window art covers the vanilla edit box, so this draws the typed text with a cursor at its end. */
    override fun queryText(): Component =
        if (input.isEmpty()) Component.text("Item name", PLACEHOLDER_TEXT) else Component.text(input + CURSOR)

    override fun resultCount(): Component {
        val count = results().size
        return Component.text(
            when {
                count == 0 -> "No matches"
                count > RESULT_CELLS -> "$RESULT_CELLS of $count shown"
                else -> "$count found"
            },
        )
    }

    override fun resetLabel(): Component =
        if (query.isEmpty()) Component.text("Clear", PLACEHOLDER_TEXT) else Component.text("Clear")

    override fun resultsItem(index: Int): ItemStack? = results().getOrNull(index)?.toItemStack()

    override fun onResults(click: IndexedClick) {
        val product = results().getOrNull(click.index) ?: return
        windows.open(player, MyShop(windows, market, product.id))
    }

    override fun onQueryChanged(value: String) {
        input = value
        syncResetState()
    }

    override fun onOpen() = syncResetState()

    override fun onBack(click: Click) {
        windows.open(player, MyShop(windows, market))
    }

    override fun onReset(click: Click) {
        windows.open(player, CatalogSearch(windows, market))
    }

    private fun syncResetState() = buttonState("reset", if (query.isEmpty()) "disabled" else "enabled")

    private fun results(): List<Product> {
        if (query.isBlank()) return market.products
        return market.products.filter { product ->
            product.name.contains(query, ignoreCase = true) ||
                product.tier.label.contains(query, ignoreCase = true) ||
                product.category.name.contains(query, ignoreCase = true)
        }
    }
}

private const val RESULT_CELLS = 27

private const val CURSOR = "_"

private val PLACEHOLDER_TEXT = TextColor.color(0x5fb0d4)
