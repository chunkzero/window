package com.chunkzero.window.example

import com.chunkzero.window.Click
import com.chunkzero.window.example.generated.CatalogSearchView
import net.minestom.server.entity.Player

/** Static native-anvil search over [MyShop]'s catalog, opened with the shop's [current] query. */
class CatalogSearch(
    player: Player,
    private val market: Market,
    private val current: String = "",
) : CatalogSearchView(player) {
    private var query = current

    override fun onOpen() {
        if (current.isNotEmpty()) input("query", current)
    }

    override fun onQueryChanged(value: String) {
        query = value.trim()
    }

    override fun onBack(click: Click) {
        MyShop(player, market, initialQuery = current).open()
    }

    override fun onConfirm(click: Click) {
        MyShop(player, market, initialQuery = query).open()
    }
}
