package com.chunkzero.window.minestom.internal

import com.chunkzero.window.host.WindowItem
import net.kyori.adventure.nbt.CompoundBinaryTag
import net.kyori.adventure.text.Component
import net.minestom.server.component.DataComponents
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material
import net.minestom.server.item.component.CustomData
import net.minestom.server.item.component.TooltipDisplay

private val HIDDEN_TOOLTIP = TooltipDisplay(true, emptySet())

/** Translates [item] to the paper item Window renders with its model. */
internal fun itemStack(item: WindowItem): ItemStack =
    when (item) {
        is WindowItem.Hitbox -> {
            val builder = paper(item.model.asString())
            val tooltip = item.tooltip
            if (tooltip == null) {
                builder.set(DataComponents.TOOLTIP_DISPLAY, HIDDEN_TOOLTIP)
            } else {
                builder.set(DataComponents.CUSTOM_NAME, tooltip.title).set(DataComponents.LORE, tooltip.lines)
            }
            builder.build()
        }

        is WindowItem.AnvilSeed -> {
            val builder =
                paper(item.model.asString())
                    .set(DataComponents.CUSTOM_NAME, Component.text(item.text))
                    .set(DataComponents.TOOLTIP_DISPLAY, HIDDEN_TOOLTIP)
                    .remove(DataComponents.ENCHANTMENTS)
            if (item.revision != 0) {
                val revision = CompoundBinaryTag.builder().putInt("window_input_revision", item.revision).build()
                builder.set(DataComponents.CUSTOM_DATA, CustomData(revision))
            }
            builder.build()
        }
    }

private fun paper(model: String): ItemStack.Builder =
    ItemStack
        .builder(Material.PAPER)
        .set(DataComponents.ITEM_MODEL, model)
        .set(DataComponents.MAX_STACK_SIZE, 1)
