package dev.oglass.window.example

import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.TextDecoration
import net.minestom.server.component.DataComponents
import net.minestom.server.entity.Player
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

enum class ProductCategory {
    GEAR,
    MAGIC,
}

enum class ProductTier(
    val label: String,
    val color: NamedTextColor,
) {
    COMMON("Common", NamedTextColor.WHITE),
    RARE("Rare", NamedTextColor.AQUA),
    EPIC("Epic", NamedTextColor.LIGHT_PURPLE),
    LEGENDARY("Legendary", NamedTextColor.GOLD),
}

data class Product(
    val id: String,
    val name: String,
    val material: Material,
    val price: Int,
    val tier: ProductTier,
    val category: ProductCategory,
    val featured: Boolean = false,
)

fun Product.toItemStack(selected: Boolean = false): ItemStack {
    val marker = if (selected) "> " else ""
    return ItemStack
        .builder(material)
        .set(
            DataComponents.CUSTOM_NAME,
            Component.text(marker + name, tier.color).decoration(TextDecoration.ITALIC, false),
        ).set(
            DataComponents.LORE,
            listOf(
                Component
                    .text("$price credits", NamedTextColor.GOLD)
                    .decoration(TextDecoration.ITALIC, false),
                Component
                    .text("${tier.label} / ${category.name.lowercase()}", NamedTextColor.GRAY)
                    .decoration(TextDecoration.ITALIC, false),
                Component
                    .text("Click to select", NamedTextColor.DARK_GRAY)
                    .decoration(TextDecoration.ITALIC, false),
            ),
        ).set(DataComponents.MAX_STACK_SIZE, 1)
        .build()
}

/** Small in-memory catalog and wallet used by the visual example. */
class Market(
    val unitPrice: Int = 10,
    private val startingBalance: Int = 2_500,
) {
    val products: List<Product> =
        listOf(
            Product(
                "diamond",
                "Diamond Cache",
                Material.DIAMOND,
                480,
                ProductTier.EPIC,
                ProductCategory.GEAR,
                true,
            ),
            Product(
                "emerald",
                "Emerald Voucher",
                Material.EMERALD,
                320,
                ProductTier.RARE,
                ProductCategory.GEAR,
                true,
            ),
            Product(
                "gold",
                "Gilded Ingot",
                Material.GOLD_INGOT,
                180,
                ProductTier.RARE,
                ProductCategory.GEAR,
            ),
            Product(
                "iron",
                "Tempered Iron",
                Material.IRON_INGOT,
                95,
                ProductTier.COMMON,
                ProductCategory.GEAR,
            ),
            Product(
                "copper",
                "Copper Bundle",
                Material.COPPER_INGOT,
                75,
                ProductTier.COMMON,
                ProductCategory.GEAR,
            ),
            Product(
                "redstone",
                "Redstone Kit",
                Material.REDSTONE,
                110,
                ProductTier.COMMON,
                ProductCategory.GEAR,
            ),
            Product(
                "lapis",
                "Lapis Bundle",
                Material.LAPIS_LAZULI,
                125,
                ProductTier.COMMON,
                ProductCategory.MAGIC,
            ),
            Product(
                "coal",
                "Compressed Coal",
                Material.COAL,
                60,
                ProductTier.COMMON,
                ProductCategory.GEAR,
            ),
            Product(
                "quartz",
                "Quartz Prism",
                Material.QUARTZ,
                140,
                ProductTier.RARE,
                ProductCategory.MAGIC,
            ),
            Product(
                "amethyst",
                "Amethyst Song",
                Material.AMETHYST_SHARD,
                210,
                ProductTier.RARE,
                ProductCategory.MAGIC,
                true,
            ),
            Product(
                "ender_pearl",
                "Ender Pearl",
                Material.ENDER_PEARL,
                240,
                ProductTier.RARE,
                ProductCategory.MAGIC,
            ),
            Product(
                "blaze_rod",
                "Blaze Conduit",
                Material.BLAZE_ROD,
                275,
                ProductTier.RARE,
                ProductCategory.MAGIC,
            ),
            Product(
                "ghast_tear",
                "Ghast Tear",
                Material.GHAST_TEAR,
                390,
                ProductTier.EPIC,
                ProductCategory.MAGIC,
            ),
            Product(
                "prismarine",
                "Prismarine Core",
                Material.PRISMARINE_SHARD,
                225,
                ProductTier.RARE,
                ProductCategory.MAGIC,
            ),
            Product(
                "heart",
                "Heart of the Sea",
                Material.HEART_OF_THE_SEA,
                760,
                ProductTier.LEGENDARY,
                ProductCategory.MAGIC,
                true,
            ),
            Product(
                "nether_star",
                "Nether Star",
                Material.NETHER_STAR,
                1_200,
                ProductTier.LEGENDARY,
                ProductCategory.MAGIC,
                true,
            ),
            Product(
                "totem",
                "Totem Charm",
                Material.TOTEM_OF_UNDYING,
                940,
                ProductTier.LEGENDARY,
                ProductCategory.MAGIC,
            ),
            Product(
                "trident",
                "Tide Trident",
                Material.TRIDENT,
                860,
                ProductTier.LEGENDARY,
                ProductCategory.GEAR,
                true,
            ),
            Product(
                "elytra",
                "Sky Wings",
                Material.ELYTRA,
                1_150,
                ProductTier.LEGENDARY,
                ProductCategory.GEAR,
                true,
            ),
            Product(
                "rocket",
                "Rocket Crate",
                Material.FIREWORK_ROCKET,
                155,
                ProductTier.COMMON,
                ProductCategory.GEAR,
            ),
            Product(
                "golden_apple",
                "Golden Apple",
                Material.GOLDEN_APPLE,
                420,
                ProductTier.EPIC,
                ProductCategory.MAGIC,
            ),
            Product(
                "experience",
                "Experience Flask",
                Material.EXPERIENCE_BOTTLE,
                195,
                ProductTier.RARE,
                ProductCategory.MAGIC,
            ),
            Product(
                "book",
                "Arcane Manual",
                Material.ENCHANTED_BOOK,
                340,
                ProductTier.EPIC,
                ProductCategory.MAGIC,
            ),
            Product(
                "spyglass",
                "Scout Spyglass",
                Material.SPYGLASS,
                165,
                ProductTier.COMMON,
                ProductCategory.GEAR,
            ),
            Product(
                "compass",
                "Explorer Compass",
                Material.COMPASS,
                205,
                ProductTier.RARE,
                ProductCategory.GEAR,
            ),
            Product(
                "clock",
                "Chrono Dial",
                Material.CLOCK,
                260,
                ProductTier.RARE,
                ProductCategory.GEAR,
            ),
            Product(
                "name_tag",
                "Identity Tag",
                Material.NAME_TAG,
                135,
                ProductTier.COMMON,
                ProductCategory.GEAR,
            ),
            Product(
                "saddle",
                "Ranger Saddle",
                Material.SADDLE,
                285,
                ProductTier.RARE,
                ProductCategory.GEAR,
            ),
            Product(
                "shell",
                "Shulker Shell",
                Material.SHULKER_SHELL,
                510,
                ProductTier.EPIC,
                ProductCategory.GEAR,
            ),
            Product(
                "echo",
                "Echo Shard",
                Material.ECHO_SHARD,
                620,
                ProductTier.EPIC,
                ProductCategory.MAGIC,
            ),
        )

    private val balances = ConcurrentHashMap<UUID, Int>()

    fun balanceOf(player: Player): Int = balances.computeIfAbsent(player.uuid) { startingBalance }

    fun purchase(
        player: Player,
        price: Int,
    ): Int? {
        var purchased = false
        val balance =
            balances.compute(player.uuid) { _, current ->
                val available = current ?: startingBalance
                if (available >= price) {
                    purchased = true
                    available - price
                } else {
                    available
                }
            }!!
        return balance.takeIf { purchased }
    }
}
