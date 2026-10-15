package com.chunkzero.window

import com.chunkzero.window.host.FullscreenHost
import com.chunkzero.window.host.FullscreenListener
import com.chunkzero.window.host.OpenFullscreen
import com.chunkzero.window.internal.Turn
import com.chunkzero.window.manifest.FullscreenMode
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.collections.shouldContain
import io.kotest.matchers.collections.shouldContainExactly
import io.kotest.matchers.collections.shouldNotContain
import io.kotest.matchers.ints.shouldBeGreaterThan
import io.kotest.matchers.shouldBe
import io.kotest.matchers.string.shouldContain
import io.kotest.matchers.string.shouldNotContain
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.TextComponent
import net.kyori.adventure.text.format.ShadowColor
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer

class FullscreenSessionTest :
    StringSpec({
        fun text(component: Component) = PlainTextComponentSerializer.plainText().serialize(component)

        "the window opens with its backdrop, layers, and cursor, each in its marker and without shadows" {
            val host = FakeFullscreenHost()
            Menu(host).open()

            val content = host.shown.single()
            text(content) shouldContain "BACKDROP"
            text(content) shouldContain "CURSOR"
            val runs = runs(content)
            runs.getValue("STATIC").color shouldBe TextColor.fromHexString(FullscreenManifests.STATIC_MARKER)
            runs.getValue("N0").color shouldBe TextColor.fromHexString(FullscreenManifests.COUNT_MARKER)
            runs.values.forEach { it.shadow shouldBe ShadowColor.none() }
        }

        "a click hits the control under the cursor, or one within the hit margin" {
            val host = FakeFullscreenHost()
            val menu = Menu(host).also { it.open() }

            host.look(110, 110)
            host.listener.onClick()
            host.look(99, 120)
            host.listener.onClick()
            host.look(98, 110)
            host.listener.onClick()

            menu.plays shouldBe 2
        }

        "a control in an inactive switch case is not hit" {
            val host = FakeFullscreenHost()
            val menu = Menu(host).also { it.open() }

            host.look(110, 170)
            host.listener.onClick()
            menu.mode = "on"
            host.listener.onClick()

            menu.secrets shouldBe 1
        }

        "a window:close control closes the window" {
            val host = FakeFullscreenHost()
            val menu = Menu(host).also { it.open() }

            host.look(110, 140)
            host.listener.onClick()
            host.listener.onClick()

            host.closes shouldBe 1
            menu.closes shouldBe 1
        }

        "the player leaving runs onClose without closing the window again" {
            val host = FakeFullscreenHost()
            val menu = Menu(host).also { it.open() }

            host.listener.onClose()
            host.look(110, 110)
            host.listener.onClick()

            menu.closes shouldBe 1
            menu.plays shouldBe 0
            host.closes shouldBe 0
        }

        "the runs are re-sent only when the hovered control's outline changes" {
            val host = FakeFullscreenHost()
            Menu(host).open()

            host.look(200, 200)
            host.look(110, 110)
            host.look(120, 115)
            host.look(110, 140)
            host.look(200, 200)

            host.shown.map { "OUTLINE" in text(it) } shouldContainExactly listOf(false, true, false)
            runs(host.shown[1]).getValue("OUTLINE").color shouldBe
                TextColor.fromHexString(FullscreenManifests.OVERLAY_MARKER)
        }

        "repeater cells are hit and hovered per cell, and the hovered cell's tooltip is sent above the layers" {
            val host = FakeFullscreenHost()
            val menu = Menu(host).also { it.open() }

            host.look(110, 200)
            host.listener.onClick()
            host.look(160, 200)
            host.listener.onClick()
            host.look(200, 250)

            menu.picked shouldContainExactly listOf("a", "b")
            host.shown.map { "Tip" in text(it) } shouldContainExactly listOf(false, true, false)
            val runs = runs(host.shown[1])
            runs.getValue("PANEL").color shouldBe TextColor.fromHexString(FullscreenManifests.TOOLTIP_MARKER)
            runs.getValue("Tip").color shouldBe TextColor.fromHexString(FullscreenManifests.TOOLTIP_TEXT_MARKER)
            val shown = text(host.shown[1])
            shown.indexOf("PANEL") shouldBeGreaterThan shown.indexOf("N0")
            shown.indexOf("CURSOR") shouldBeGreaterThan shown.indexOf("Tip")
        }

        "a list pages its items through the repeater's cells" {
            val host = FakeFullscreenHost()
            val menu = Menu(host).also { it.open() }

            host.look(110, 230)
            host.listener.onClick()
            host.scheduler.tick()
            host.look(110, 200)
            host.listener.onClick()

            runs(host.shown.last()).keys shouldContain "c"
            runs(host.shown.last()).keys shouldNotContain "a"
            menu.picked shouldContainExactly listOf("c")
        }

        "state changes re-send the runs on the next tick" {
            val host = FakeFullscreenHost()
            Menu(host).open()

            host.look(110, 110)
            host.listener.onClick()
            text(host.shown.last()) shouldNotContain "N1"
            host.scheduler.tick()

            text(host.shown.last()) shouldContain "N1"
        }

        "turning past an edge is pulled back at most once per round trip" {
            val host = FakeFullscreenHost()
            var now = 0L
            FullscreenWindowSession(FullscreenDefinition(FullscreenManifests.manifest(), "menu"), Menu(host), host) {
                now
            }.open()

            host.listener.onLook(50f, 0f)
            host.listener.onLook(50f, 0f)
            now += 149_000_000L
            host.listener.onLook(50f, 0f)
            now += 1_000_000L
            host.listener.onLook(50f, 0f)
            host.listener.onLook(45.5f, 0f)

            host.turns shouldContainExactly listOf(Turn(-5f, 0f), Turn(-5f, 0f))
        }

        "horse mode is refused" {
            val host = FakeFullscreenHost()
            shouldThrow<IllegalStateException> { Menu(host, FullscreenMode.HORSE).open() }
            host.shown shouldBe emptyList()
        }
    })

private class Menu(
    host: FullscreenHost,
    mode: FullscreenMode = FullscreenMode.UNLOCKED,
) : FullscreenView(FullscreenDefinition(FullscreenManifests.manifest(mode), "menu"), host) {
    var count by state(0)
    var mode by state("off")
    var plays = 0
    var secrets = 0
    var closes = 0
    val picked = mutableListOf<String>()
    private val names = list(2, { it: String -> it }) { listOf("a", "b", "c") }

    override fun FullscreenScope.bind() {
        slot("count") { Component.text("N$count") }
        switch("mode") { mode }
        button("play") {
            plays++
            count++
        }
        button("secret") { secrets++ }
        for (cell in 0 until 2) {
            slot("cell[$cell]") { Component.text(names.at(cell).orEmpty()) }
            button("pick[$cell]") { names.selectAt(cell)?.let(picked::add) }
        }
        button("next") { names.next() }
    }

    override fun onClose() {
        closes++
    }
}

private class FakeFullscreenHost :
    FullscreenHost,
    OpenFullscreen {
    val scheduler = ManualScheduler()
    val shown = mutableListOf<Component>()
    val turns = mutableListOf<Turn>()
    var closes = 0
    lateinit var listener: FullscreenListener

    override val latencyMillis = 50

    override fun open(
        definition: FullscreenDefinition,
        content: Component,
        listener: FullscreenListener,
    ): OpenFullscreen {
        this.listener = listener
        shown += content
        return this
    }

    override fun scheduleNextTick(task: Runnable) = scheduler.schedule(task)

    override fun show(content: Component) {
        shown += content
    }

    override fun turn(
        yaw: Float,
        pitch: Float,
    ) {
        turns += Turn(yaw, pitch)
    }

    override fun close() {
        closes++
    }

    /** Turns the player so the cursor lands on canvas pixel ([x], [y]). */
    fun look(
        x: Int,
        y: Int,
    ) {
        val (yaw, pitch) = FullscreenManifests.rotation(x, y)
        listener.onLook(yaw, pitch)
    }
}

private data class Run(
    val color: TextColor?,
    val shadow: ShadowColor?,
)

/** The non-blank text runs of [component] by content, with their inherited color and shadow. */
private fun runs(component: Component): Map<String, Run> =
    buildMap {
        fun visit(
            node: Component,
            color: TextColor?,
            shadow: ShadowColor?,
        ) {
            val nodeColor = node.color() ?: color
            val nodeShadow = node.shadowColor() ?: shadow
            val content = (node as? TextComponent)?.content()?.filter { it.isLetterOrDigit() }.orEmpty()
            if (content.isNotEmpty()) put(content, Run(nodeColor, nodeShadow))
            node.children().forEach { visit(it, nodeColor, nodeShadow) }
        }
        visit(component, null, null)
    }
