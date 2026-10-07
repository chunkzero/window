package com.chunkzero.window.example

import com.chunkzero.window.Click
import com.chunkzero.window.WindowCollection
import com.chunkzero.window.example.generated.ShopView
import com.chunkzero.window.example.generated.WindowSprite
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.TextColor
import net.minestom.server.entity.Player
import net.minestom.server.item.ItemStack

/** Interactive catalog demonstrating collections, paging, selections, toggles, and live text. */
class MyShop(
    player: Player,
    private val market: Market,
    initialQuery: String = "",
) : ShopView(player) {
    private var coins by state(0)
    private var feedback by state<String?>(null)
    private var query by state(initialQuery)

    private val catalog = list(cells = CATALOG_CELLS, key = Product::id) { visibleProducts() }

    override val products: WindowCollection<ItemStack> =
        catalog.items(Product::toItemStack) { product, _ ->
            catalog.select(product)
            feedback = null
        }

    override fun balance(): Component = Component.text(coins)

    override fun categoryLabel(index: Int): Component =
        tabLabel(CATEGORY_LABELS[index], Category.entries[index] == category)

    override fun sortLabel(index: Int): Component = tabLabel(SORT_LABELS[index], Sort.entries[index] == sort)

    override fun favoritesLampSprite(): WindowSprite = lamp(favorites)

    override fun affordableLampSprite(): WindowSprite = lamp(affordable)

    override fun canPrevious(): Boolean = catalog.canPrevious()

    override fun canNext(): Boolean = catalog.canNext()

    override fun previousLabel(): Component = actionLabel("Prev", catalog.canPrevious())

    override fun nextLabel(): Component = actionLabel("Next", catalog.canNext())

    override fun page(): Component = Component.text("${catalog.page} of ${catalog.pageCount}")

    override fun selectedName(): Component = Component.text(catalog.selected?.name ?: "No matches")

    override fun price(): Component =
        Component.text(
            catalog.selected
                ?.price
                ?.toString()
                .orEmpty(),
        )

    override fun hasPrice(): Boolean = catalog.selected != null

    override fun hasQuery(): Boolean = query.isNotEmpty()

    override fun status(): Component {
        val count = catalog.size
        return Component.text(feedback ?: if (query.isEmpty()) "$count items" else "$count matches")
    }

    override fun canBuy(): Boolean = catalog.selected?.let { it.price <= coins } == true

    override fun buyLabel(): Component {
        val product = catalog.selected ?: return actionLabel("Buy", false)
        if (product.price > coins) return actionLabel("Need ${product.price - coins}", false)
        return Component.text("Buy ${product.price}")
    }

    override fun onPrevious(click: Click) = catalog.previous()

    override fun onNext(click: Click) = catalog.next()

    override fun onSearch(click: Click) {
        CatalogSearch(player, market, query).open()
    }

    override fun onClearSearch(click: Click) {
        query = ""
        filtersChanged()
    }

    override fun onBuy(click: Click) {
        val product = catalog.selected ?: return
        val newBalance = market.purchase(player, product.price)
        if (newBalance == null) {
            feedback = "Need ${product.price - coins} more"
        } else {
            coins = newBalance
            feedback = "Purchased"
        }
    }

    override fun onCategoryChanged(value: Category) = filtersChanged()

    override fun onSortChanged(value: Sort) = filtersChanged()

    override fun onFavoritesChanged(value: Boolean) = filtersChanged()

    override fun onAffordableChanged(value: Boolean) = filtersChanged()

    override fun onOpen() {
        coins = market.balanceOf(player)
    }

    private fun visibleProducts(): List<Product> {
        var products = market.products.asSequence().filter(::matchesQuery)
        products =
            when (category) {
                Category.ALL -> products
                Category.GEAR -> products.filter { it.category == ProductCategory.GEAR }
                Category.MAGIC -> products.filter { it.category == ProductCategory.MAGIC }
            }
        if (favorites) products = products.filter(Product::featured)
        if (affordable) products = products.filter { it.price <= coins }
        return when (sort) {
            Sort.FEATURED -> products.sortedWith(compareByDescending<Product>(Product::featured).thenBy(Product::price))
            Sort.PRICE -> products.sortedBy(Product::price)
            Sort.NAME -> products.sortedBy(Product::name)
        }.toList()
    }

    private fun matchesQuery(product: Product): Boolean =
        product.name.contains(query, ignoreCase = true) ||
            product.tier.label.contains(query, ignoreCase = true) ||
            product.category.name.contains(query, ignoreCase = true)

    /** Keeps the selected product when it survives the new filters and scrolls to it; otherwise shows the start. */
    private fun filtersChanged() {
        feedback = null
        val selected = catalog.selected
        if (selected != null) catalog.select(selected) else catalog.first()
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

    private fun lamp(on: Boolean): WindowSprite = if (on) WindowSprite.LAMP_ON else WindowSprite.LAMP_OFF
}

/** The `products` collection's cells: three rows of the container. */
private const val CATALOG_CELLS = 27
private val CATEGORY_LABELS = listOf("All", "Gear", "Magic")
private val SORT_LABELS = listOf("Top", "Price", "Name")
private val DISABLED_TEXT = TextColor.color(0x5fb0d4)
