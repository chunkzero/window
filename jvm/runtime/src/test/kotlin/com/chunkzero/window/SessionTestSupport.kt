package com.chunkzero.window

import com.chunkzero.window.host.ContainerKind
import com.chunkzero.window.host.ContainerListener
import com.chunkzero.window.host.HudDescriptor
import com.chunkzero.window.host.HudOutput
import com.chunkzero.window.host.OpenContainer
import com.chunkzero.window.host.WindowHost
import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.internal.RenderScheduler
import com.chunkzero.window.manifest.WindowManifest
import net.kyori.adventure.text.Component

/** A scheduler that records tasks and runs them on demand, for deterministic flush testing. */
internal class ManualScheduler : RenderScheduler {
    private val tasks = ArrayDeque<Runnable>()
    var scheduleCount = 0
        private set

    val pending: Int
        get() = tasks.size

    override fun schedule(task: Runnable) {
        scheduleCount++
        tasks.addLast(task)
    }

    /** Runs only the tasks queued so far, like one server tick. */
    fun tick() {
        repeat(tasks.size) { tasks.removeFirst().run() }
    }

    /** Runs all queued tasks (FIFO), draining the queue. */
    fun runAll() {
        while (tasks.isNotEmpty()) tasks.removeFirst().run()
    }
}

/**
 * An in-memory host whose items are the [WindowItem]s themselves (tests may also pass any other
 * value as an item). It opens [container] once and records shown HUDs.
 */
internal class FakeHost : WindowHost<Any> {
    val scheduler = ManualScheduler()
    val container = FakeContainer()
    val huds = mutableListOf<FakeHudOutput>()

    override fun open(
        kind: ContainerKind,
        title: Component,
        listener: ContainerListener,
    ): OpenContainer<Any> = container.also { it.open(kind, title, listener) }

    override fun showHud(hud: HudDescriptor): HudOutput = FakeHudOutput(hud).also { huds += it }

    override fun item(item: WindowItem): Any = item

    override fun scheduleNextTick(task: Runnable) = scheduler.schedule(task)
}

/** A container capturing what the core sends, with helpers to drive client input. */
internal class FakeContainer : OpenContainer<Any> {
    lateinit var kind: ContainerKind
        private set
    val titles = mutableListOf<Component>()
    val items = mutableMapOf<SlotRef, Any>()
    val pings = mutableListOf<Int>()
    var opened = false
        private set
    var closed = false
        private set
    var playerSlotsRestored = false
        private set
    private var listener: ContainerListener? = null

    override val size: Int
        get() = kind.size

    fun open(
        kind: ContainerKind,
        title: Component,
        listener: ContainerListener,
    ) {
        check(!opened) { "FakeContainer opened twice" }
        opened = true
        this.kind = kind
        this.listener = listener
        titles += title
    }

    override fun setTitle(title: Component) {
        titles += title
    }

    override fun setItem(
        slot: SlotRef,
        item: Any?,
    ) {
        if (item == null) items.remove(slot) else items[slot] = item
    }

    override fun stageItem(
        slot: Int,
        item: Any?,
    ) = setItem(SlotRef(SlotArea.CONTAINER, slot), item)

    override fun restorePlayerSlots() {
        playerSlotsRestored = true
    }

    override fun batch(action: () -> Unit) = action()

    override fun ping(id: Int) {
        pings += id
    }

    override fun close() {
        closed = true
        listener = null
    }

    /** Simulates a client click landing on [slot]. */
    fun click(
        slot: SlotRef,
        shift: Boolean = false,
        right: Boolean = false,
    ) {
        listener?.onClick(slot, shift, right)
    }

    fun clickContainer(
        slot: Int,
        shift: Boolean = false,
        right: Boolean = false,
    ) = click(SlotRef(SlotArea.CONTAINER, slot), shift, right)

    /** Simulates a client-initiated close; the container stops listening first. */
    fun clientClose() {
        val listener = listener ?: return
        this.listener = null
        listener.onClose()
    }

    /** Simulates an anvil text-box update. */
    fun input(value: String) {
        listener?.onAnvilInput(value)
    }

    /** Simulates the client answering the latest ping. */
    fun pong() {
        listener?.onPong(pings.last())
    }
}

/** A HUD output recording every component it shows. */
internal class FakeHudOutput(
    val descriptor: HudDescriptor,
) : HudOutput {
    val contents = mutableListOf(descriptor.content)
    var hidden = false
        private set

    override fun update(content: Component) {
        contents += content
    }

    override fun hide() {
        hidden = true
    }
}

/** A window view over [manifest]'s window [name], shown through [host]. */
internal abstract class TestView(
    manifest: WindowManifest,
    host: FakeHost,
    name: String = "w",
) : WindowView<Any>(WindowDefinition(manifest, name), host)

/** A HUD view over [manifest]'s HUD [name], shown through [host]. */
internal abstract class TestHud(
    manifest: WindowManifest,
    host: FakeHost,
    name: String = "h",
) : HudView(HudDefinition(manifest, name), host)
