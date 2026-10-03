package dev.oglass.window.example

import dev.oglass.window.Click
import dev.oglass.window.Windows
import dev.oglass.window.example.generated.CatalogSearchView

/** Static native-anvil search over [MyShop]'s catalog, opened with the shop's [current] query. */
class CatalogSearch(
    private val windows: Windows,
    private val market: Market,
    private val current: String = "",
) : CatalogSearchView() {
    private var query = current

    override fun onOpen() {
        if (current.isNotEmpty()) input("query", current)
    }

    override fun onQueryChanged(value: String) {
        query = value.trim()
    }

    override fun onBack(click: Click) {
        windows.open(player, MyShop(windows, market, query = current))
    }

    override fun onClear(click: Click) = input("query", "")

    override fun onConfirm(click: Click) {
        windows.open(player, MyShop(windows, market, query = query))
    }
}
