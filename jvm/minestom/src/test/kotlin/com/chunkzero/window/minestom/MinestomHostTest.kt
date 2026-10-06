package com.chunkzero.window.minestom

import com.chunkzero.window.ButtonTooltip
import com.chunkzero.window.Click
import com.chunkzero.window.SlotArea
import com.chunkzero.window.SlotRef
import com.chunkzero.window.host.ContainerKind
import com.chunkzero.window.host.ContainerListener
import com.chunkzero.window.host.WindowItem
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.types.shouldBeSameInstanceAs
import io.kotest.matchers.types.shouldNotBeSameInstanceAs
import net.kyori.adventure.key.Key
import net.kyori.adventure.nbt.CompoundBinaryTag
import net.kyori.adventure.text.Component
import net.minestom.server.component.DataComponents
import net.minestom.server.event.EventDispatcher
import net.minestom.server.event.inventory.InventoryPreClickEvent
import net.minestom.server.item.component.CustomData
import net.minestom.server.item.component.TooltipDisplay
import net.minestom.server.inventory.click.Click as MinestomClick

class MinestomHostTest :
    StringSpec({
        "every view and HUD of a player shares one host" {
            val player = MinestomFixture().player

            MinestomHost.of(player) shouldBeSameInstanceAs MinestomHost.of(player)
            MinestomHost.of(MinestomFixture().player) shouldNotBeSameInstanceAs MinestomHost.of(player)
        }

        "pre-clicks on the player region of the opened inventory are normalised by container size" {
            val fixture = MinestomFixture()
            val clicks = mutableListOf<Click>()
            val container = fixture.host.open(ContainerKind.CHEST_3_ROW, Component.text("w"), Clicks(clicks))

            EventDispatcher.call(
                InventoryPreClickEvent(
                    fixture.player.openInventory,
                    fixture.player,
                    MinestomClick.RightShift(container.size + 18),
                ),
            )

            clicks shouldBe listOf(Click(SlotRef(SlotArea.PLAYER, 18), shift = true, right = true))
        }

        "anvil seeds hide their tooltip, hold no enchantments, and carry their revision" {
            val host = MinestomFixture().host
            val seed = host.item(WindowItem.AnvilSeed(Key.key("window", "gui/hitbox"), "maps", revision = 4))

            seed.has(DataComponents.ENCHANTMENTS) shouldBe false
            seed.get(DataComponents.CUSTOM_NAME) shouldBe Component.text("maps")
            seed.get(DataComponents.ITEM_MODEL) shouldBe "window:gui/hitbox"
            seed.get(DataComponents.TOOLTIP_DISPLAY) shouldBe TooltipDisplay(true, emptySet())
            seed.get(DataComponents.CUSTOM_DATA) shouldBe
                CustomData(CompoundBinaryTag.builder().putInt("window_input_revision", 4).build())
            host
                .item(
                    WindowItem.AnvilSeed(Key.key("window", "gui/hitbox"), "maps"),
                ).has(DataComponents.CUSTOM_DATA) shouldBe
                false
        }

        "hitboxes use their tooltip verbatim, or hide it" {
            val host = MinestomFixture().host
            val model = Key.key("window", "gui/hitbox")
            val tooltip = ButtonTooltip(Component.text("Home"), listOf(Component.text("Go home")))

            val shown = host.item(WindowItem.Hitbox(model, tooltip))
            val hidden = host.item(WindowItem.Hitbox(model, null))

            shown.get(DataComponents.CUSTOM_NAME) shouldBe tooltip.title
            shown.get(DataComponents.LORE) shouldBe tooltip.lines
            shown.get(DataComponents.MAX_STACK_SIZE) shouldBe 1
            hidden.get(DataComponents.TOOLTIP_DISPLAY) shouldBe TooltipDisplay(true, emptySet())
            hidden.get(DataComponents.MAX_STACK_SIZE) shouldBe 1
        }
    })

private class Clicks(
    private val clicks: MutableList<Click>,
) : ContainerListener {
    override fun onClick(
        slot: SlotRef,
        shift: Boolean,
        right: Boolean,
    ) {
        clicks += Click(slot, shift, right)
    }

    override fun onClose() = Unit

    override fun onAnvilInput(text: String) = Unit

    override fun onPong(id: Int) = Unit
}
