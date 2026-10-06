package com.chunkzero.window.host.testkit

import com.chunkzero.window.ButtonTooltip
import com.chunkzero.window.Click
import com.chunkzero.window.SlotArea
import com.chunkzero.window.SlotRef
import com.chunkzero.window.host.ContainerKind
import com.chunkzero.window.host.ContainerListener
import com.chunkzero.window.host.HudChannel
import com.chunkzero.window.host.HudDescriptor
import com.chunkzero.window.host.WindowItem
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.collections.shouldBeEmpty
import io.kotest.matchers.collections.shouldContain
import io.kotest.matchers.collections.shouldContainExactlyInAnyOrder
import io.kotest.matchers.nulls.shouldNotBeNull
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component

/**
 * The behaviour every [com.chunkzero.window.host.WindowHost] must show, run against a fresh [fixture] per test.
 *
 * Extend it in a host's tests with that host's [HostFixture].
 */
public abstract class HostConformance<I : Any>(
    createFixture: () -> HostFixture<I>,
) : StringSpec({
        val fixtures = mutableListOf<HostFixture<I>>()

        fun fixture(): HostFixture<I> = createFixture().also(fixtures::add)

        afterEach {
            fixtures.forEach(HostFixture<I>::close)
            fixtures.clear()
        }

        "every container kind opens with its slot count and title" {
            for (kind in ContainerKind.entries) {
                val client = fixture()
                val container = client.host.open(kind, Component.text(kind.name), RecordingListener())

                container.size shouldBe kind.size
                client.updates.screen()?.title shouldBe Component.text(kind.name)
            }
        }

        "container clicks route to container slots with their modifiers" {
            val client = fixture()
            val listener = RecordingListener()
            client.host.open(ContainerKind.CHEST_3_ROW, TITLE, listener)

            client.click(0)
            client.click(5, right = true)
            client.click(13, shift = true)
            client.click(26, shift = true, right = true)

            listener.clicks shouldBe
                listOf(
                    Click(container(0), shift = false, right = false),
                    Click(container(5), shift = false, right = true),
                    Click(container(13), shift = true, right = false),
                    Click(container(26), shift = true, right = true),
                )
        }

        "player-region clicks route to player inventory slots" {
            val client = fixture()
            val listener = RecordingListener()
            val size = client.host.open(ContainerKind.CHEST_3_ROW, TITLE, listener).size

            client.click(size)
            client.click(size + 9, right = true)
            client.click(size + 26)
            client.click(size + 27, shift = true)
            client.click(size + 35)

            listener.clicks shouldBe
                listOf(
                    Click(player(9), shift = false, right = false),
                    Click(player(18), shift = false, right = true),
                    Click(player(35), shift = false, right = false),
                    Click(player(0), shift = true, right = false),
                    Click(player(8), shift = false, right = false),
                )
        }

        "anvil hotbar cells route to player hotbar slots" {
            val client = fixture()
            val listener = RecordingListener()
            val size = client.host.open(ContainerKind.ANVIL, TITLE, listener).size

            for (hotbar in listOf(0, 4, 8)) client.click(size + 27 + hotbar)

            listener.clicks.map { it.slot } shouldBe listOf(player(0), player(4), player(8))
        }

        "player inventory clicks route even after an earlier listener cancelled them" {
            val client = fixture()
            client.guardPlayerInventory()
            val listener = RecordingListener()
            val size = client.host.open(ContainerKind.CHEST_3_ROW, TITLE, listener).size

            client.click(size)

            listener.clicks.map { it.slot } shouldBe listOf(player(9))
        }

        "clicks never move items" {
            val client = fixture()
            val container = client.host.open(ContainerKind.CHEST_1_ROW, TITLE, RecordingListener())
            val hitbox = client.hitbox("hitbox")
            container.setItem(container(0), hitbox)

            client.click(0, shift = true)
            client.click(0)

            client.updates
                .screen()
                .shouldNotBeNull()
                .slots[0] shouldBe hitbox
            (0 until PLAYER_SLOTS).mapNotNull(client::playerItem).shouldBeEmpty()
        }

        "a later listener that un-cancels clicks cannot move container or leased player items" {
            val client = fixture()
            client.allowClicksGlobally()
            val container = client.host.open(ContainerKind.CHEST_3_ROW, TITLE, RecordingListener())
            val hitbox = client.hitbox("hitbox")
            container.setItem(container(0), hitbox)
            container.setItem(player(9), hitbox)

            client.click(0)
            client.click(container.size)
            client.click(0, shift = true)
            client.click(container.size, shift = true)

            val slots =
                client.updates
                    .screen()
                    .shouldNotBeNull()
                    .slots
            slots[0] shouldBe hitbox
            slots[27] shouldBe hitbox
            client.playerItem(9) shouldBe hitbox
        }

        "a later listener that un-cancels clicks cannot move anvil items" {
            val client = fixture()
            client.allowClicksGlobally()
            val anvil = client.host.open(ContainerKind.ANVIL, TITLE, RecordingListener())
            val hitbox = client.hitbox("hitbox")
            anvil.setItem(container(0), hitbox)

            client.click(0)

            client.updates
                .screen()
                .shouldNotBeNull()
                .slots[0] shouldBe hitbox
        }

        "player slots are cleared, mirrored to the open screen, and restored" {
            val client = fixture()
            val diamond = client.hitbox("diamond")
            val emerald = client.hitbox("emerald")
            val gold = client.hitbox("gold")
            val iron = client.hitbox("iron")
            client.setPlayerItem(0, diamond)
            client.setPlayerItem(8, emerald)
            client.setPlayerItem(9, gold)
            client.setPlayerItem(35, iron)
            val container = client.host.open(ContainerKind.CHEST_3_ROW, TITLE, RecordingListener())
            val hitbox = client.hitbox("hitbox")

            container.setItem(player(9), hitbox)

            client.playerItem(0) shouldBe null
            client.playerItem(8) shouldBe null
            client.playerItem(9) shouldBe hitbox
            client.playerItem(35) shouldBe null
            client.updates shouldContain ClientUpdate.SetSlot(54, null)
            client.updates
                .screen()
                .shouldNotBeNull()
                .slots[27] shouldBe hitbox

            container.restorePlayerSlots()

            client.playerItem(9) shouldBe gold
            val restored =
                client.updates
                    .screen()
                    .shouldNotBeNull()
                    .slots
            listOf(restored[54], restored[62], restored[27], restored[53]) shouldBe listOf(diamond, emerald, gold, iron)

            container.setItem(player(0), hitbox)
            container.close()

            listOf(0, 8, 9, 35).map(client::playerItem) shouldBe listOf(diamond, emerald, gold, iron)
        }

        "a client close reports onClose once and restores player slots" {
            val client = fixture()
            val diamond = client.hitbox("diamond")
            client.setPlayerItem(0, diamond)
            val listener = RecordingListener()
            val container = client.host.open(ContainerKind.CHEST_3_ROW, TITLE, listener)
            container.setItem(player(0), client.hitbox("hitbox"))

            client.closeScreen()
            container.close()
            client.tick()
            client.tick()

            listener.closes shouldBe 1
            client.playerItem(0) shouldBe diamond
        }

        "a server close reports no onClose, restores player slots, and stops listening" {
            val client = fixture()
            val diamond = client.hitbox("diamond")
            client.setPlayerItem(0, diamond)
            val listener = RecordingListener()
            val container = client.host.open(ContainerKind.CHEST_3_ROW, TITLE, listener)
            container.setItem(player(0), client.hitbox("hitbox"))

            container.close()
            client.pong(1)
            container.close()

            listener.closes shouldBe 0
            listener.pongs.shouldBeEmpty()
            client.updates.screen() shouldBe null
            client.playerItem(0) shouldBe diamond
        }

        "opening another container from a click replaces the first, reporting its onClose once" {
            val client = fixture()
            val diamond = client.hitbox("diamond")
            client.setPlayerItem(0, diamond)
            val events = mutableListOf<String>()
            val second =
                RecordingListener(afterClose = { events += "second:close" }) { events += "second:click" }
            val first =
                RecordingListener(afterClose = { events += "first:close" }) {
                    events += "first:click"
                    client.host.open(ContainerKind.CHEST_1_ROW, Component.text("second"), second)
                }
            val firstContainer = client.host.open(ContainerKind.CHEST_3_ROW, Component.text("first"), first)
            firstContainer.setItem(player(0), client.hitbox("hitbox"))

            client.click(firstContainer.size + 27)
            client.tick()
            client.click(ContainerKind.CHEST_1_ROW.size + 27)
            client.closeScreen()
            client.tick()

            events shouldBe listOf("first:click", "first:close", "second:click", "second:close")
            client.playerItem(0) shouldBe diamond
        }

        "closing a replaced container leaves its replacement open" {
            val client = fixture()
            val first = RecordingListener()
            val second = RecordingListener()
            val firstContainer = client.host.open(ContainerKind.CHEST_3_ROW, Component.text("first"), first)
            client.host.open(ContainerKind.CHEST_1_ROW, Component.text("second"), second)

            firstContainer.close()
            client.click(0)
            client.tick()

            first.closes shouldBe 1
            first.clicks.shouldBeEmpty()
            second.clicks.map { it.slot } shouldBe listOf(container(0))
            client.updates.screen()?.title shouldBe Component.text("second")
        }

        "a replacement returns a cursor item to the restored inventory instead of erasing it" {
            val client = fixture()
            client.cancelDropsGlobally()
            val diamond = client.hitbox("diamond")
            val emerald = client.hitbox("emerald")
            client.setPlayerItem(0, diamond)
            val first = client.host.open(ContainerKind.CHEST_3_ROW, Component.text("first"), RecordingListener())
            first.setItem(player(0), client.hitbox("hitbox"))
            client.setCursorItem(emerald)

            client.host.open(ContainerKind.CHEST_1_ROW, Component.text("second"), RecordingListener())

            (0 until PLAYER_SLOTS).mapNotNull(client::playerItem) shouldContainExactlyInAnyOrder
                listOf(diamond, emerald)
            client.cursorItem() shouldBe null
        }

        "a client close returns a cursor item to the restored inventory instead of erasing it" {
            val client = fixture()
            client.cancelDropsGlobally()
            val diamond = client.hitbox("diamond")
            val emerald = client.hitbox("emerald")
            client.setPlayerItem(0, diamond)
            val container = client.host.open(ContainerKind.CHEST_3_ROW, TITLE, RecordingListener())
            container.setItem(player(0), client.hitbox("hitbox"))
            client.setCursorItem(emerald)

            client.closeScreen()

            (0 until PLAYER_SLOTS).mapNotNull(client::playerItem) shouldContainExactlyInAnyOrder
                listOf(diamond, emerald)
        }

        "a window opened from a replaced container's onClose replaces its replacement" {
            val client = fixture()
            val third = RecordingListener()
            val second = RecordingListener()
            val first =
                RecordingListener(afterClose = {
                    client.host.open(ContainerKind.CHEST_3_ROW, Component.text("third"), third)
                })
            client.host.open(ContainerKind.CHEST_3_ROW, Component.text("first"), first)
            client.host.open(ContainerKind.CHEST_3_ROW, Component.text("second"), second)

            client.tick()
            client.tick()
            client.click(ContainerKind.CHEST_3_ROW.size)

            first.closes shouldBe 1
            second.closes shouldBe 1
            second.clicks.shouldBeEmpty()
            third.clicks.map { it.slot } shouldBe listOf(player(9))
            client.updates.screen()?.title shouldBe Component.text("third")
        }

        "clicks on the player's own inventory window never reach the player inventory or the listener" {
            val client = fixture()
            client.allowClicksGlobally()
            val listener = RecordingListener()
            val container = client.host.open(ContainerKind.CHEST_3_ROW, TITLE, listener)
            val hitbox = client.hitbox("hitbox")
            container.setItem(player(0), hitbox)

            client.clickPlayerWindow(HOTBAR_WINDOW_SLOT)

            client.playerItem(0) shouldBe hitbox
            client.cursorItem() shouldBe null
            listener.clicks.shouldBeEmpty()
        }

        "a replacement that an open listener cancels leaves the current container open and listening" {
            val client = fixture()
            val first = RecordingListener()
            val firstContainer = client.host.open(ContainerKind.CHEST_3_ROW, Component.text("first"), first)
            client.cancelOpensGlobally()

            shouldThrow<IllegalStateException> {
                client.host.open(ContainerKind.CHEST_1_ROW, Component.text("second"), RecordingListener())
            }
            client.click(0)
            firstContainer.close()

            first.closes shouldBe 0
            first.clicks.map { it.slot } shouldBe listOf(container(0))
            client.updates.screen() shouldBe null
        }

        "an open that a listener cancels fails without listening or touching player slots" {
            val client = fixture()
            val diamond = client.hitbox("diamond")
            client.setPlayerItem(0, diamond)
            client.cancelOpensGlobally()
            val listener = RecordingListener()

            shouldThrow<IllegalStateException> { client.host.open(ContainerKind.CHEST_3_ROW, TITLE, listener) }
            client.pong(1)

            listener.pongs.shouldBeEmpty()
            listener.closes shouldBe 0
            client.updates.screen() shouldBe null
            client.playerItem(0) shouldBe diamond
        }

        "setTitle retitles the open screen without onClose" {
            val client = fixture()
            val listener = RecordingListener()
            val container = client.host.open(ContainerKind.CHEST_1_ROW, TITLE, listener)

            container.setTitle(Component.text("renamed"))
            client.click(0)

            client.updates.screen()?.title shouldBe Component.text("renamed")
            listener.closes shouldBe 0
            listener.clicks.map { it.slot } shouldBe listOf(container(0))
        }

        "an anvil setTitle reopens with the staged items and without onClose" {
            val client = fixture()
            val listener = RecordingListener()
            val anvil = client.host.open(ContainerKind.ANVIL, TITLE, listener)
            val seed = client.host.item(WindowItem.AnvilSeed(MODEL, "seed", revision = 1))

            anvil.stageItem(0, seed)

            client.updates
                .screen()
                .shouldNotBeNull()
                .slots[0] shouldBe null

            anvil.setTitle(Component.text("reopened"))
            client.click(2)

            val screen = client.updates.screen().shouldNotBeNull()
            screen.title shouldBe Component.text("reopened")
            screen.slots[0] shouldBe seed
            listener.closes shouldBe 0
            listener.clicks.map { it.slot } shouldBe listOf(container(2))
        }

        "every anvil text change reaches the listener" {
            val client = fixture()
            val listener = RecordingListener()
            client.host.open(ContainerKind.ANVIL, TITLE, listener)

            client.typeInAnvil("abc")
            client.typeInAnvil("abc")
            client.typeInAnvil("")

            listener.inputs shouldBe listOf("abc", "abc", "")
        }

        "a batch sends its packets as one bundle, in order" {
            val client = fixture()
            val container = client.host.open(ContainerKind.CHEST_1_ROW, TITLE, RecordingListener())
            val first = client.hitbox("first")
            val second = client.hitbox("second")
            val before = client.updates.size

            container.batch {
                container.setItem(container(0), first)
                container.ping(7)
                container.setItem(container(1), second)
            }

            client.updates.drop(before) shouldBe
                listOf(
                    ClientUpdate.BundleDelimiter,
                    ClientUpdate.SetSlot(0, first),
                    ClientUpdate.Ping(7),
                    ClientUpdate.SetSlot(1, second),
                    ClientUpdate.BundleDelimiter,
                )
        }

        "every pong reaches the listener in order" {
            val client = fixture()
            val listener = RecordingListener()
            val container = client.host.open(ContainerKind.ANVIL, TITLE, listener)

            container.ping(1)
            container.ping(2)
            client.pong(1)
            client.pong(2)
            client.pong(99)

            listener.pongs shouldBe listOf(1, 2, 99)
        }

        "a scheduled task runs once, on the next tick" {
            val client = fixture()
            var runs = 0

            client.host.scheduleNextTick { runs++ }
            runs shouldBe 0
            client.tick()
            client.tick()

            runs shouldBe 1
        }

        for (channel in HudChannel.entries) {
            "a $channel HUD shows, updates and hides" {
                val client = fixture()

                val output = client.host.showHud(HudDescriptor(channel, Component.text("shown")))
                client.updates.hud(channel) shouldBe Component.text("shown")

                output.update(Component.text("updated"))
                client.updates.hud(channel) shouldBe Component.text("updated")

                output.hide()
                client.updates.hud(channel) shouldBe null
            }
        }

        "equal item descriptions build equal items" {
            val host = fixture().host
            val tooltip = ButtonTooltip(Component.text("title"), listOf(Component.text("line")))

            host.item(WindowItem.Hitbox(MODEL, tooltip)) shouldBe host.item(WindowItem.Hitbox(MODEL, tooltip))
            host.item(WindowItem.AnvilSeed(MODEL, "text", 3)) shouldBe host.item(WindowItem.AnvilSeed(MODEL, "text", 3))
        }

        "hitboxes differ by model and tooltip" {
            val host = fixture().host
            val tooltip = ButtonTooltip(Component.text("title"))

            host.item(WindowItem.Hitbox(MODEL, null)) shouldNotBe
                host.item(WindowItem.Hitbox(Key.key("test", "other"), null))
            host.item(WindowItem.Hitbox(MODEL, null)) shouldNotBe host.item(WindowItem.Hitbox(MODEL, tooltip))
        }

        "anvil seeds differ by text and revision" {
            val host = fixture().host
            val seeds =
                listOf(
                    WindowItem.AnvilSeed(MODEL, "text"),
                    WindowItem.AnvilSeed(MODEL, "text", 1),
                    WindowItem.AnvilSeed(MODEL, "text", 2),
                    WindowItem.AnvilSeed(MODEL, "other", 2),
                ).map(host::item)

            seeds.toSet().size shouldBe seeds.size
        }
    })

private val TITLE = Component.text("window")
private val MODEL = Key.key("window", "gui/hitbox")
private const val PLAYER_SLOTS = 36

/** The first hotbar slot in the player inventory window. */
private const val HOTBAR_WINDOW_SLOT = 36

private fun container(index: Int) = SlotRef(SlotArea.CONTAINER, index)

private fun player(index: Int) = SlotRef(SlotArea.PLAYER, index)

/** A distinct item named [name]. */
private fun <I : Any> HostFixture<I>.hitbox(name: String): I =
    host.item(WindowItem.Hitbox(Key.key("test", name), ButtonTooltip(Component.text(name))))

/** What the client shows in its open screen. */
private data class Screen<I : Any>(
    val title: Component,
    val slots: Map<Int, I>,
)

/** The screen the client shows after these updates, or `null` when none is open. */
private fun <I : Any> List<ClientUpdate<I>>.screen(): Screen<I>? =
    fold(null as Screen<I>?) { screen, update ->
        when (update) {
            is ClientUpdate.OpenScreen -> Screen(update.title, emptyMap())
            is ClientUpdate.SetSlot -> screen?.copy(slots = screen.slots.with(update.slot, update.item))
            ClientUpdate.CloseScreen -> null
            else -> screen
        }
    }

private fun <I : Any> Map<Int, I>.with(
    slot: Int,
    item: I?,
): Map<Int, I> = if (item == null) this - slot else this + (slot to item)

/** The content the client shows on [channel] after these updates, or `null` when hidden. */
private fun List<ClientUpdate<*>>.hud(channel: HudChannel): Component? =
    filterIsInstance<ClientUpdate.Hud>().lastOrNull { it.channel == channel }?.content

private class RecordingListener(
    private val afterClose: () -> Unit = {},
    private val afterClick: (Click) -> Unit = {},
) : ContainerListener {
    val clicks = mutableListOf<Click>()
    var closes = 0
        private set
    val inputs = mutableListOf<String>()
    val pongs = mutableListOf<Int>()

    override fun onClick(
        slot: SlotRef,
        shift: Boolean,
        right: Boolean,
    ) {
        val click = Click(slot, shift, right)
        clicks += click
        afterClick(click)
    }

    override fun onClose() {
        closes++
        afterClose()
    }

    override fun onAnvilInput(text: String) {
        inputs += text
    }

    override fun onPong(id: Int) {
        pongs += id
    }
}
