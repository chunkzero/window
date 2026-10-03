package dev.oglass.window.example

import dev.oglass.window.Click
import dev.oglass.window.Windows
import dev.oglass.window.example.generated.CatalogSearchView

/** Static native-anvil search: confirming opens [MyShop] filtered to the typed query. */
class CatalogSearch(
    private val windows: Windows,
    private val market: Market,
) : CatalogSearchView() {
    private var query = ""

    override fun onQueryChanged(value: String) {
        query = value.trim()
    }

    override fun onBack(click: Click) {
        windows.open(player, MyShop(windows, market))
    }

    override fun onConfirm(click: Click) {
        windows.open(player, MyShop(windows, market, query = query))
    }
}
