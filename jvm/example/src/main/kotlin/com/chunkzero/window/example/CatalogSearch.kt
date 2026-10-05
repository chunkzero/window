package com.chunkzero.window.example

import com.chunkzero.window.Click
import com.chunkzero.window.Windows
import com.chunkzero.window.example.generated.CatalogSearchView

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
        windows.open(player, MyShop(windows, market, initialQuery = current))
    }

    override fun onConfirm(click: Click) {
        windows.open(player, MyShop(windows, market, initialQuery = query))
    }
}
