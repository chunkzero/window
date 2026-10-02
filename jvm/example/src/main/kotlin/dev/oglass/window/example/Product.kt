package dev.oglass.window.example

import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.TextDecoration
import net.minestom.server.component.DataComponents
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material

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
