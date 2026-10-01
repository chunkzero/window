package dev.oglass.window.example

import dev.oglass.window.Click
import dev.oglass.window.IndexedClick
import dev.oglass.window.WindowPager
import dev.oglass.window.Windows
import dev.oglass.window.example.generated.ShopView
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.minestom.server.item.ItemStack

private enum class CatalogCategory {
    ALL,
    GEAR,
    MAGIC,
}

private enum class CatalogSort {
    FEATURED,
    PRICE,
    NAME,
}

/** Interactive catalog demonstrating collections, paging, choices, toggles, and live text. */
class MyShop(
    private val windows: Windows,
    private val market: Market,
    initialSelection: String? = null,
) : ShopView() {
    private val pager = WindowPager(cellCount = 21)

    private var coins by state(0)
    private var category by state(CatalogCategory.ALL)
    private var sort by state(CatalogSort.FEATURED)
    private var favoritesOnly by state(false)
    private var affordableOnly by state(false)
    private var offset by state(0)
    private var selectedId by state(initialSelection)
    private var feedback by state("SELECT ITEM")

    override fun balance(): Component = Component.text("$coins C")

    override fun categoryAllLabel(): Component = tabLabel("ALL", category == CatalogCategory.ALL)

    override fun categoryGearLabel(): Component = tabLabel("GEAR", category == CatalogCategory.GEAR)

    override fun categoryMagicLabel(): Component = tabLabel("MAGIC", category == CatalogCategory.MAGIC)

    override fun sortFeaturedLabel(): Component = tabLabel("FEATURED", sort == CatalogSort.FEATURED)

    override fun sortPriceLabel(): Component = tabLabel("PRICE", sort == CatalogSort.PRICE)

    override fun sortNameLabel(): Component = tabLabel("NAME", sort == CatalogSort.NAME)

    override fun favoritesLabel(): Component = Component.text(if (favoritesOnly) "FAVS: ON" else "FAVS: OFF")

    override fun affordableLabel(): Component = Component.text(if (affordableOnly) "BUDGET: ON" else "BUDGET: OFF")

    override fun previousLabel(): Component = Component.text(if (pager.canPrevious(offset)) "PREV" else "-")

    override fun nextLabel(): Component {
        val products = visibleProducts()
        return Component.text(if (pager.canNext(offset, products.size)) "NEXT" else "-")
    }

    override fun page(): Component {
        val products = visibleProducts()
        return Component.text(
            "${pager.page(offset, products.size)} / ${pager.pageCount(products.size)}",
        )
    }

    override fun selection(): Component {
        val selected = selectedProduct()
        return Component.text(selected?.name ?: "NO MATCHES")
    }

    override fun status(): Component = Component.text(feedback)

    override fun buyLabel(): Component = Component.text(selectedProduct()?.let { "BUY ${it.price} C" } ?: "BUY")

    override fun productsItem(index: Int): ItemStack? {
        val products = visibleProducts()
        val absolute = pager.itemIndex(offset, index, products.size) ?: return null
        val product = products[absolute]
        return product.toItemStack(product.id == selectedId)
    }

    override fun onProducts(click: IndexedClick) {
        val products = visibleProducts()
        val absolute = pager.itemIndex(offset, click.index, products.size) ?: return
        val product = products[absolute]
        selectedId = product.id
        feedback = "SELECTED"
        syncButtonStates()
    }

    override fun onCategoryAll(click: Click) = selectCategory(CatalogCategory.ALL)

    override fun onCategoryGear(click: Click) = selectCategory(CatalogCategory.GEAR)

    override fun onCategoryMagic(click: Click) = selectCategory(CatalogCategory.MAGIC)

    override fun onSortFeatured(click: Click) = selectSort(CatalogSort.FEATURED)

    override fun onSortPrice(click: Click) = selectSort(CatalogSort.PRICE)

    override fun onSortName(click: Click) = selectSort(CatalogSort.NAME)

    override fun onFavorites(click: Click) {
        favoritesOnly = !favoritesOnly
        resetPageAndSelection()
        feedback = if (favoritesOnly) "FAVORITES ON" else "FAVORITES OFF"
        syncButtonStates()
    }

    override fun onAffordable(click: Click) {
        affordableOnly = !affordableOnly
        resetPageAndSelection()
        feedback = if (affordableOnly) "BUDGET ON" else "BUDGET OFF"
        syncButtonStates()
    }

    override fun onPrevious(click: Click) {
        offset = pager.previous(offset, visibleProducts().size)
        ensureSelection()
        syncButtonStates()
    }

    override fun onNext(click: Click) {
        offset = pager.next(offset, visibleProducts().size)
        ensureSelection()
        syncButtonStates()
    }

    override fun onSearch(click: Click) {
        windows.open(player, CatalogSearch(windows, market))
    }

    override fun onBuy(click: Click) {
        val product = selectedProduct() ?: return
        val newBalance = market.purchase(player, product.price)
        if (newBalance == null) {
            feedback = "NEED ${product.price - coins} C"
        } else {
            coins = newBalance
            feedback = "PURCHASED"
        }
        syncButtonStates()
    }

    override fun onOpen() {
        coins = market.balanceOf(player)
        ensureSelection()
        syncButtonStates()
    }

    private fun visibleProducts(): List<Product> {
        var products = market.products.asSequence()
        products =
            when (category) {
                CatalogCategory.ALL -> products
                CatalogCategory.GEAR -> products.filter { it.category == ProductCategory.GEAR }
                CatalogCategory.MAGIC -> products.filter { it.category == ProductCategory.MAGIC }
            }
        if (favoritesOnly) products = products.filter(Product::featured)
        if (affordableOnly) products = products.filter { it.price <= coins }
        return when (sort) {
            CatalogSort.FEATURED -> {
                products.sortedWith(
                    compareByDescending<Product>(Product::featured).thenBy(Product::price),
                )
            }

            CatalogSort.PRICE -> {
                products.sortedBy(Product::price)
            }

            CatalogSort.NAME -> {
                products.sortedBy(Product::name)
            }
        }.toList()
    }

    private fun selectedProduct(): Product? {
        val products = visibleProducts()
        return products.firstOrNull { it.id == selectedId } ?: products.firstOrNull()
    }

    private fun selectCategory(value: CatalogCategory) {
        category = value
        resetPageAndSelection()
        feedback = "FILTER: ${value.name}"
        syncButtonStates()
    }

    private fun selectSort(value: CatalogSort) {
        sort = value
        resetPageAndSelection()
        feedback = "SORT: ${value.name}"
        syncButtonStates()
    }

    private fun resetPageAndSelection() {
        offset = 0
        selectedId = visibleProducts().firstOrNull()?.id
    }

    private fun ensureSelection() {
        val products = visibleProducts()
        offset = pager.clamp(offset, products.size)
        if (products.none { it.id == selectedId }) selectedId = products.firstOrNull()?.id
    }

    private fun syncButtonStates() {
        val products = visibleProducts()
        buttonState("category_all", selectedState(category == CatalogCategory.ALL))
        buttonState("category_gear", selectedState(category == CatalogCategory.GEAR))
        buttonState("category_magic", selectedState(category == CatalogCategory.MAGIC))
        buttonState("sort_featured", selectedState(sort == CatalogSort.FEATURED))
        buttonState("sort_price", selectedState(sort == CatalogSort.PRICE))
        buttonState("sort_name", selectedState(sort == CatalogSort.NAME))
        buttonState("favorites", if (favoritesOnly) "on" else "off")
        buttonState("affordable", if (affordableOnly) "on" else "off")
        buttonState("previous", enabledState(pager.canPrevious(offset)))
        buttonState("next", enabledState(pager.canNext(offset, products.size)))
        buttonState("buy", enabledState(selectedProduct()?.price?.let { it <= coins } == true))
    }

    private fun tabLabel(
        label: String,
        selected: Boolean,
    ): Component = Component.text(label, if (selected) NamedTextColor.GOLD else NamedTextColor.WHITE)

    private fun selectedState(selected: Boolean): String = if (selected) "selected" else "unselected"

    private fun enabledState(enabled: Boolean): String = if (enabled) "enabled" else "disabled"
}
