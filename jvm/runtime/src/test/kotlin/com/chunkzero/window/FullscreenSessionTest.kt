package com.chunkzero.window

import com.chunkzero.window.host.FullscreenHost
import com.chunkzero.window.host.FullscreenListener
import com.chunkzero.window.host.OpenFullscreen
import com.chunkzero.window.internal.Turn
import com.chunkzero.window.manifest.FullscreenMode
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.collections.shouldContainExactly
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

        "right clicks and scroll steps reach their handlers, right clicks at the cursor" {
            val host = FakeFullscreenHost()
            val menu = Menu(host).also { it.open() }

            host.look(200, 200)
            host.listener.onRightClick()
            host.listener.onScroll(1)
            host.listener.onScroll(-3)

            menu.rightClicks shouldContainExactly listOf(FullscreenClick(200, 200))
            menu.scrolls shouldContainExactly listOf(1, -3)
        }

        "a prompt ignores canvas input until it ends, including when a container covers it" {
            val host = FakeFullscreenHost()
            val menu = Menu(host).also { it.open() }

            menu.mode = "on"
            host.look(110, 170)
            menu.prompt("a")
            host.listener.onClick()
            host.listener.onRightClick()
            host.listener.onScroll(1)
            menu.secrets shouldBe 0
            menu.rightClicks shouldBe emptyList()
            menu.scrolls shouldBe emptyList()

            host.answer(null)
            menu.prompts shouldContainExactly listOf(null)
            host.listener.onClick()
            host.listener.onScroll(1)
            menu.secrets shouldBe 1
            menu.scrolls shouldContainExactly listOf(1)
        }

        "a callback that opens a prompt while another replaces it leaves that prompt pending" {
            val host = FakeFullscreenHost()
            val menu = Menu(host).also { it.open() }

            menu.mode = "on"
            host.look(110, 170)
            menu.prompt("a") { menu.prompt("c") }
            menu.prompt("b")
            menu.prompts shouldContainExactly listOf(null, null)
            host.listener.onClick()
            menu.secrets shouldBe 0

            host.answer("text")
            menu.prompts shouldContainExactly listOf(null, null, "text")
            host.listener.onClick()
            menu.secrets shouldBe 1
        }

        "closing with a prompt open reports null and runs onClose even when the callback prompts again or throws" {
            val host = FakeFullscreenHost()
            val menu = Menu(host)
            val session = menu.open()

            menu.prompt("a") {
                menu.prompt("b")
                error("boom")
            }
            shouldThrow<IllegalStateException> { session.close() }

            menu.prompts shouldContainExactly listOf(null, null)
            menu.closes shouldBe 1
            host.closes shouldBe 1
            host.pending shouldBe null
        }

        "horse mode sends no cursor run, since the entity shader draws the cursor, and hit-tests the same rotation" {
            val host = FakeFullscreenHost()
            val menu = Menu(host, FullscreenMode.HORSE).also { it.open() }

            host.look(110, 110)
            host.listener.onClick()

            host.shown.forEach { text(it) shouldNotContain "CURSOR" }
            text(host.shown.last()) shouldContain "OUTLINE"
            menu.plays shouldBe 1
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
    val rightClicks = mutableListOf<FullscreenClick>()
    val scrolls = mutableListOf<Int>()
    val prompts = mutableListOf<String?>()

    override fun FullscreenScope.bind() {
        slot("count") { Component.text("N$count") }
        switch("mode") { mode }
        button("play") {
            plays++
            count++
        }
        button("secret") { secrets++ }
        rightClick { rightClicks += it }
        scroll { scrolls += it }
    }

    fun prompt(
        initial: String,
        then: () -> Unit = {},
    ) = promptText(initial = initial) {
        prompts += it
        then()
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
    var pending: ((String?) -> Unit)? = null
    var closes = 0
    private var open = true
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

    override val promptOpen get() = pending != null

    override fun promptText(
        title: Component,
        initial: String,
        onDone: (String?) -> Unit,
    ) {
        if (!open) return onDone(null)
        val replaced = pending
        pending = onDone
        replaced?.invoke(null)
    }

    override fun close() {
        closes++
        open = false
        answer(null)
    }

    /** Ends the pending prompt with [text], as a button press or a container takeover does. */
    fun answer(text: String?) {
        val done = pending ?: return
        pending = null
        done(text)
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
