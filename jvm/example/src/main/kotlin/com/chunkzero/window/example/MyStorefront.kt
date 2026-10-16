package com.chunkzero.window.example

import com.chunkzero.window.ExperimentalFullscreenApi
import com.chunkzero.window.FullscreenClick
import com.chunkzero.window.example.generated.StorefrontFullscreen
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import net.minestom.server.timer.Task
import net.minestom.server.timer.TaskSchedule
import kotlin.math.abs
import kotlin.math.ceil

/**
 * A web-style storefront drawn over the live world: category tabs and a search box typed into in place filter a product
 * grid, a detail pane buys or adds to a cart, and the grid and the cart each scroll with the wheel and their arrows.
 * The wheel moves the pane last clicked; right click flips the side pane between the details and the cart.
 */
class MyStorefront(
    player: Player,
    private val market: Market,
) : StorefrontFullscreen(player) {
    private var coins by state(0)
    private var query by state("")
    private var searching by state(false)
    private var sort by state(Sort.FEATURED)
    private var amount by state(1)

    /** Quantities by product id, in the order they were first added. */
    private var cart by state(mapOf<String, Int>())
    private var wheelOnCatalog by state(true)
    private var toast by state<Pair<String, Tone>?>(null)
    private var toastTask: Task? = null

    private val catalog =
        list(cells = CARD_NAME_SIZE, key = { it: Product? ->
            it?.id
        }, step = COLUMNS) { visibleProducts().inWholeRows() }
    private val lines =
        list(cells = CART_NAME_SIZE, key = { it: Pair<Product, Int> -> it.first.id }, step = 1) { cartLines() }

    override fun onOpen() {
        coins = market.balanceOf(player)
    }

    override fun onClose() {
        toastTask?.cancel()
    }

    // Navigation bar

    override fun storeSearch(): Component =
        if (searching) editing(query) else fit("store_search", Component.text(query))

    override fun storeHasSearch(): Boolean = searching || query.isNotEmpty()

    /** Lets the player type the query in place; the grid filters as they type, until they press Escape. */
    @OptIn(ExperimentalFullscreenApi::class)
    override fun onStoreFind(click: FullscreenClick) {
        searching = true
        promptText(query, onEdit = ::search) { text ->
            searching = false
            text?.let(::search)
        }
    }

    override fun onStoreClearSearch(click: FullscreenClick) = search("")

    override fun storeCartCount(): Component = Component.text(cart.values.sum())

    override fun onStoreCategoryChanged(value: StoreCategory) = catalogChanged()

    override fun onOpenDetails(click: FullscreenClick) = showPane(StorePane.DETAILS)

    override fun onOpenCart(click: FullscreenClick) = showPane(StorePane.CART)

    // Product grid

    override fun storeResults(): Component {
        val count = visibleProducts().size
        val label = if (query.isBlank()) CATEGORY_LABELS.getValue(storeCategory) else "\"${query.trim()}\""
        return Component.text(if (count == 0) "$label: no matches" else "$label: ${plural(count, "item")}")
    }

    override fun storeSortLabel(): Component = Component.text("Sort: ${sort.label}")

    override fun onStoreSort(click: FullscreenClick) {
        sort = Sort.entries[(sort.ordinal + 1) % Sort.entries.size]
        catalogChanged()
    }

    override fun cardIcon(index: Int): CardIcon =
        catalog.at(index)?.let { CardIcon.valueOf(it.id.uppercase()) } ?: CardIcon.NONE

    override fun cardName(index: Int): Component = Component.text(catalog.at(index)?.name.orEmpty())

    override fun cardPrice(index: Int): Component =
        Component.text(
            catalog
                .at(index)
                ?.price
                ?.toString()
                .orEmpty(),
        )

    override fun cardFilled(index: Int): Boolean = catalog.at(index) != null

    override fun cardFeatured(index: Int): Boolean = catalog.at(index)?.featured == true

    override fun cardSelected(index: Int): Boolean = catalog.isSelected(index)

    override fun onPickCard(
        index: Int,
        click: FullscreenClick,
    ) {
        wheelOnCatalog = true
        if (catalog.selectAt(index) == null) return
        amount = 1
        storePane = StorePane.DETAILS
    }

    override fun catalogFocused(): Boolean = wheelOnCatalog

    override fun catalogThumb(index: Int): Boolean =
        thumb(
            index,
            CATALOG_THUMB_SIZE,
            catalog.visible.first / COLUMNS,
            CARD_NAME_SIZE / COLUMNS,
            catalog.size / COLUMNS,
        )

    override fun onCatalogUp(click: FullscreenClick) {
        wheelOnCatalog = true
        catalog.previous()
    }

    override fun onCatalogDown(click: FullscreenClick) {
        wheelOnCatalog = true
        catalog.next()
    }

    // Detail pane

    private val selected: Product? get() = catalog.selected

    override fun detailIcon(): DetailIcon = selected?.let { DetailIcon.valueOf(it.id.uppercase()) } ?: DetailIcon.NONE

    override fun detailName(): Component = Component.text(selected?.name ?: "Nothing selected")

    override fun detailTier(): DetailTier = DetailTier.valueOf(selected?.tier?.name ?: "COMMON")

    override fun detailKind(): Component =
        Component.text(selected?.let { "${it.tier.label} ${it.category.name.lowercase()}" }.orEmpty())

    override fun detailPrice(): Component = Component.text(selected?.price?.toString().orEmpty())

    override fun detailBlurb(): Component = Component.text(selected?.let { BLURBS[it.id] }.orEmpty())

    override fun detailInCart(): Component {
        val count = selected?.let { cart[it.id] } ?: return Component.text("Not in your cart yet")
        return Component.text("$count already in your cart")
    }

    override fun quantity(): Component = Component.text(amount)

    override fun canLess(): Boolean = amount > 1

    override fun canMore(): Boolean = amount < MAX_AMOUNT

    override fun onLess(click: FullscreenClick) {
        amount = (amount - 1).coerceAtLeast(1)
    }

    override fun onMore(click: FullscreenClick) {
        amount = (amount + 1).coerceAtMost(MAX_AMOUNT)
    }

    override fun canBuyNow(): Boolean = selected?.let { it.price * amount <= coins } == true

    override fun buyNowLabel(): Component {
        val product = selected ?: return Component.text("Buy")
        val total = product.price * amount
        return Component.text(if (total <= coins) "Buy $total" else "Need ${total - coins}")
    }

    override fun onAddToCart(click: FullscreenClick) {
        val product = selected ?: return
        cart = cart + (product.id to (cart[product.id] ?: 0) + amount)
        show("Added ${amountOf(product)} to your cart", Tone.OK)
    }

    override fun onBuyNow(click: FullscreenClick) {
        val product = selected ?: return
        if (pay(product.price * amount)) show("Bought ${amountOf(product)}", Tone.OK)
    }

    // Cart pane

    override fun cartEmpty(): Boolean = cart.isEmpty()

    override fun cartName(index: Int): Component =
        Component.text(
            lines
                .at(index)
                ?.first
                ?.name
                .orEmpty(),
        )

    override fun cartQuantity(index: Int): Component =
        Component.text(lines.at(index)?.let { "x${it.second}" }.orEmpty())

    override fun cartPrice(index: Int): Component =
        Component.text(
            lines
                .at(index)
                ?.let { (product, count) -> product.price * count }
                ?.toString()
                .orEmpty(),
        )

    override fun cartThumb(index: Int): Boolean =
        thumb(index, CART_THUMB_SIZE, lines.visible.first, CART_NAME_SIZE, lines.size)

    override fun cartTotal(): Component = Component.text(cartTotalPrice())

    override fun canCheckout(): Boolean = cart.isNotEmpty() && cartTotalPrice() <= coins

    override fun onRemoveLine(
        index: Int,
        click: FullscreenClick,
    ) {
        wheelOnCatalog = false
        val (product, count) = lines.at(index) ?: return
        cart = if (count > 1) cart + (product.id to count - 1) else cart - product.id
        show("Removed one ${product.name}", Tone.WARN)
    }

    override fun onCartUp(click: FullscreenClick) {
        wheelOnCatalog = false
        lines.previous()
    }

    override fun onCartDown(click: FullscreenClick) {
        wheelOnCatalog = false
        lines.next()
    }

    override fun onCheckout(click: FullscreenClick) {
        val count = cart.values.sum()
        if (!pay(cartTotalPrice())) return
        cart = emptyMap()
        show("Checked out ${plural(count, "item")}", Tone.OK)
    }

    override fun onClearCart(click: FullscreenClick) {
        cart = emptyMap()
        show("Cart cleared", Tone.WARN)
    }

    // Footer and input

    override fun storeBalance(): Component = Component.text(coins)

    override fun storeToast(): Component = Component.text(toast?.first ?: "Welcome to the Foundry Market")

    override fun tone(): Tone = toast?.second ?: Tone.OK

    override fun onScroll(steps: Int) {
        val target = if (wheelOnCatalog) catalog else lines
        repeat(abs(steps)) { if (steps > 0) target.next() else target.previous() }
    }

    override fun onRightClick(click: FullscreenClick) {
        showPane(if (storePane == StorePane.DETAILS) StorePane.CART else StorePane.DETAILS)
    }

    private fun showPane(pane: StorePane) {
        storePane = pane
        wheelOnCatalog = pane == StorePane.DETAILS
    }

    private fun visibleProducts(): List<Product> {
        val products =
            market.products.filter { product ->
                product.name.contains(query.trim(), ignoreCase = true) &&
                    when (storeCategory) {
                        StoreCategory.ALL -> true
                        StoreCategory.GEAR -> product.category == ProductCategory.GEAR
                        StoreCategory.MAGIC -> product.category == ProductCategory.MAGIC
                        StoreCategory.DEALS -> product.featured
                    }
            }
        return products.sortedWith(sort.order)
    }

    /** Pads the grid with empty entries so every row is full and the list scrolls a whole row at a time. */
    private fun List<Product>.inWholeRows(): List<Product?> = this + List((-size).mod(COLUMNS)) { null }

    private fun cartLines(): List<Pair<Product, Int>> =
        cart.mapNotNull { (id, count) -> market.products.find { it.id == id }?.let { it to count } }

    private fun cartTotalPrice(): Int = cartLines().sumOf { (product, count) -> product.price * count }

    private fun search(text: String) {
        query = text
        catalogChanged()
    }

    /** Shows the top of the grid; the selection stays while it survives the change. */
    private fun catalogChanged() {
        catalog.first()
        wheelOnCatalog = true
    }

    /** Pays [price] from the wallet, or explains the shortfall. */
    private fun pay(price: Int): Boolean {
        val balance = market.purchase(player, price)
        if (balance == null) {
            show("Need ${plural(price - coins, "more coin")}", Tone.DANGER)
            return false
        }
        coins = balance
        return true
    }

    private fun amountOf(product: Product): String = if (amount == 1) product.name else "${product.name} x$amount"

    /** Shows [message] in the footer for a few seconds. */
    private fun show(
        message: String,
        tone: Tone,
    ) {
        toast = message to tone
        toastTask?.cancel()
        toastTask =
            player
                .scheduler()
                .buildTask { toast = null }
                .delay(TaskSchedule.seconds(4))
                .schedule()
    }

    /** [text] and a caret, dropping its start once it outgrows the field so the end stays in view. */
    private fun editing(text: String): Component =
        (0..text.length)
            .asSequence()
            .map { Component.text(text.substring(it) + "_") }
            .first { fit("store_search", it) == it }

    private enum class Sort(
        val label: String,
        val order: Comparator<Product>,
    ) {
        FEATURED("Top", compareByDescending(Product::featured).thenBy(Product::price)),
        CHEAPEST("Cheap", compareBy(Product::price)),
        PRICIEST("Pricey", compareByDescending(Product::price)),
        NAME("A-Z", compareBy(Product::name)),
    }
}

private const val COLUMNS = 3
private const val MAX_AMOUNT = 9

private val CATEGORY_LABELS =
    mapOf(
        StorefrontFullscreen.StoreCategory.ALL to "All",
        StorefrontFullscreen.StoreCategory.GEAR to "Gear",
        StorefrontFullscreen.StoreCategory.MAGIC to "Magic",
        StorefrontFullscreen.StoreCategory.DEALS to "Deals",
    )

private fun plural(
    count: Int,
    noun: String,
): String = if (count == 1) "1 $noun" else "$count ${noun}s"

/**
 * Whether scrollbar segment [index] of [segments] is lit: whether its stretch of a [total]-row list overlaps the
 * [visible] rows starting at [first]. A list that fits lights every segment.
 */
private fun thumb(
    index: Int,
    segments: Int,
    first: Int,
    visible: Int,
    total: Int,
): Boolean {
    if (total <= visible) return true
    val start = first * segments / total
    val end = ceil((first + visible) * segments / total.toDouble()).toInt()
    return index in start until end
}

private val BLURBS =
    mapOf(
        "diamond" to "A sealed crate of flawless diamonds, cut at the deep mines of Ironvale.",
        "emerald" to "Redeemable at any villager stall for a fair trade, no haggling needed.",
        "gold" to "Gilded bars from the Emberly vaults, stamped with the foundry seal.",
        "iron" to "Quench-hardened iron that holds an edge through a long night of mining.",
        "copper" to "Bright copper bundled for wiring, rods, and the odd lightning trap.",
        "redstone" to "Dust, repeaters, and a page of circuits to get a contraption running.",
        "lapis" to "Fine lapis for the enchanting table, ground and bottled for travel.",
        "coal" to "Packed coal that burns three times as long in any furnace.",
        "quartz" to "A nether quartz prism that splits torchlight into seven colours.",
        "amethyst" to "Shards that hum a low note when the moon is full. Staff favourite.",
        "ender_pearl" to "One throw and you are there. Mind the fall and the endermen.",
        "blaze_rod" to "A rod still warm from the fortress, ready to fire a brewing stand.",
        "ghast_tear" to "Gathered gently from a sleeping ghast. Brews a strong regeneration.",
        "prismarine" to "The glowing heart of an ocean monument, pried loose with care.",
        "heart" to "Breathe and see beneath the waves once it sits in a conduit frame.",
        "nether_star" to "Taken from the wither itself. Powers a beacon for a whole base.",
        "totem" to "Holds death back once, then crumbles. Keep it in your off hand.",
        "trident" to "Forged in the drowned halls. Returns to your hand and calls lightning in a storm.",
        "elytra" to "Wings from an end ship. Pair with rockets for a long glide home.",
        "rocket" to "A crate of flight rockets, three gunpowder strong. No stars, no boom.",
        "golden_apple" to "Crisp and gilded. Absorbs a hit or two and heals on the go.",
        "experience" to "A flask of bottled experience, for when the anvil says too expensive.",
        "book" to "A manual of rare enchantments, bound in leather and lapis ink.",
        "spyglass" to "Copper and amethyst lenses that pick out a creeper a mile off.",
        "compass" to "Points home, or to a lodestone, through rain, night, and caves.",
        "clock" to "Tells the hour underground, so you know when to head back up.",
        "name_tag" to "Name a pet, a cow, or a very patient zombie. Ink included.",
        "saddle" to "A ranger's saddle that fits horses, pigs, and striders alike.",
        "shell" to "A shulker shell, half of a box that carries a whole chest.",
        "echo" to "A shard from the deep dark that still answers when you call it.",
    )
