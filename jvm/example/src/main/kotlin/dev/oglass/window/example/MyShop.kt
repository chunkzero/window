package dev.oglass.window.example

import dev.oglass.window.Click
import dev.oglass.window.IndexedClick
import dev.oglass.window.WindowPager
import dev.oglass.window.Windows
import dev.oglass.window.example.generated.ShopView
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.TextColor
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
    private val pager = WindowPager(cellCount = 27)

    private var coins by state(0)
    private var category by state(CatalogCategory.ALL)
    private var sort by state(CatalogSort.FEATURED)
    private var favoritesOnly by state(false)
    private var affordableOnly by state(false)
    private var offset by state(0)
    private var selectedId by state(initialSelection)
    private var feedback by state<String?>(null)

    override fun balance(): Component = Component.text(coins)

    override fun categoryAllLabel(): Component = tabLabel("All", category == CatalogCategory.ALL)

    override fun categoryGearLabel(): Component = tabLabel("Gear", category == CatalogCategory.GEAR)

    override fun categoryMagicLabel(): Component = tabLabel("Magic", category == CatalogCategory.MAGIC)

    override fun sortFeaturedLabel(): Component = tabLabel("Top", sort == CatalogSort.FEATURED)

    override fun sortPriceLabel(): Component = tabLabel("Price", sort == CatalogSort.PRICE)

    override fun sortNameLabel(): Component = tabLabel("Name", sort == CatalogSort.NAME)

    override fun favoritesLabel(): Component = Component.text("Favs")

    override fun favoritesLampSprite(): String = lamp(favoritesOnly)

    override fun affordableLabel(): Component = Component.text("Afford")

    override fun affordableLampSprite(): String = lamp(affordableOnly)

    override fun previousLabel(): Component = actionLabel("Prev", pager.canPrevious(offset))

    override fun nextLabel(): Component = actionLabel("Next", pager.canNext(offset, visibleProducts().size))

    override fun page(): Component {
        val products = visibleProducts()
        return Component.text(
            "${pager.page(offset, products.size)} of ${pager.pageCount(products.size)}",
        )
    }

    override fun selection(): Component = Component.text(selectedProduct()?.name ?: "No matches")

    override fun price(): Component = Component.text(selectedProduct()?.price?.toString().orEmpty())

    override fun priceCoinSprite(): String? = selectedProduct()?.let { "coin" }

    override fun status(): Component = Component.text(feedback ?: "${visibleProducts().size} items")

    override fun buyLabel(): Component {
        val product = selectedProduct() ?: return actionLabel("Buy", false)
        if (product.price > coins) return actionLabel("Need ${product.price - coins}", false)
        return Component.text("Buy ${product.price}")
    }

    override fun productsItem(index: Int): ItemStack? {
        val products = visibleProducts()
        val absolute = pager.itemIndex(offset, index, products.size) ?: return null
        return products[absolute].toItemStack()
    }

    override fun productsSelected(): Int? {
        val products = visibleProducts()
        val absolute = products.indexOf(selectedProduct() ?: return null)
        return absolute - pager.clamp(offset, products.size)
    }

    override fun onProducts(click: IndexedClick) {
        val products = visibleProducts()
        val absolute = pager.itemIndex(offset, click.index, products.size) ?: return
        val product = products[absolute]
        selectedId = product.id
        feedback = null
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
        feedback = null
        syncButtonStates()
    }

    override fun onAffordable(click: Click) {
        affordableOnly = !affordableOnly
        resetPageAndSelection()
        feedback = null
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
            feedback = "Need ${product.price - coins} more"
        } else {
            coins = newBalance
            feedback = "Purchased"
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
        feedback = null
        syncButtonStates()
    }

    private fun selectSort(value: CatalogSort) {
        sort = value
        resetPageAndSelection()
        feedback = null
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

    /** Disabled labels fade toward the disabled button fill; enabled ones keep the slot's authored color. */
    private fun actionLabel(
        label: String,
        enabled: Boolean,
    ): Component = if (enabled) Component.text(label, NamedTextColor.WHITE) else Component.text(label, DISABLED_TEXT)

    private fun lamp(on: Boolean): String = if (on) "lamp_on" else "lamp_off"

    private fun selectedState(selected: Boolean): String = if (selected) "selected" else "unselected"

    private fun enabledState(enabled: Boolean): String = if (enabled) "enabled" else "disabled"
}

private val DISABLED_TEXT = TextColor.color(0x5fb0d4)
