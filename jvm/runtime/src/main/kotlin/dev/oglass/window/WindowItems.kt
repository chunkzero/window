package dev.oglass.window

import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.format.TextDecoration
import net.minestom.server.component.DataComponents
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material
import net.minestom.server.item.component.TooltipDisplay

/** Item helpers for Window button/hotspot hitboxes. */
public object WindowItems {
    /**
     * Builds the seed item that enables a vanilla anvil rename field. It has no enchantments
     * component, so the client never predicts a repair result or draws its cost label, and it shows
     * no tooltip.
     */
    public fun anvilInput(
        initial: String = "",
        itemModel: String = "window:gui/hitbox",
    ): ItemStack =
        ItemStack
            .builder(Material.PAPER)
            .set(DataComponents.ITEM_MODEL, itemModel)
            .set(DataComponents.CUSTOM_NAME, Component.text(initial))
            .set(DataComponents.MAX_STACK_SIZE, 1)
            .set(DataComponents.TOOLTIP_DISPLAY, TooltipDisplay(true, emptySet()))
            .remove(DataComponents.ENCHANTMENTS)
            .build()

    /** Builds an invisible Window hitbox item with [tooltip]. */
    public fun hitbox(
        tooltip: ButtonTooltip,
        itemModel: String = "window:gui/hitbox",
    ): ItemStack =
        ItemStack
            .builder(Material.PAPER)
            .set(DataComponents.ITEM_MODEL, itemModel)
            .set(DataComponents.CUSTOM_NAME, clean(tooltip.title, NamedTextColor.WHITE))
            .set(DataComponents.LORE, tooltip.lines.map { clean(it, NamedTextColor.GRAY) })
            .set(DataComponents.MAX_STACK_SIZE, 1)
            .build()

    private fun clean(
        component: Component,
        defaultColor: TextColor,
    ): Component {
        val colored = if (component.color() == null) component.color(defaultColor) else component
        return colored.decoration(TextDecoration.ITALIC, false)
    }
}
