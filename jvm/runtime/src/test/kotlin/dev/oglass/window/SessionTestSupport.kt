package dev.oglass.window

import dev.oglass.window.internal.ClickInfo
import dev.oglass.window.internal.InventoryHandle
import dev.oglass.window.internal.RenderScheduler
import dev.oglass.window.manifest.WindowManifest
import net.kyori.adventure.text.Component
import net.minestom.server.item.ItemStack

/** A scheduler that records tasks and runs them on demand, for deterministic flush testing. */
internal class ManualScheduler : RenderScheduler {
    private val tasks = ArrayDeque<Runnable>()
    var scheduleCount = 0
        private set

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

    val pending: Int
        get() = tasks.size
}

/** A fake handle capturing title sends and exposing the click/close callbacks for driving. */
internal class FakeInventoryHandle : InventoryHandle {
    override val containerId: Int = 7
    val titles = mutableListOf<Component>()
    val items = mutableMapOf<SlotRef, ItemStack>()
    var opened = false
        private set

    var closed = false
        private set

    var listenersTorndown = false
        private set

    private var onClick: ((ClickInfo) -> Unit)? = null
    private var onClose: (() -> Unit)? = null
    private var onInput: ((String) -> Unit)? = null
    private var onPong: ((Int) -> Unit)? = null
    val pings = mutableListOf<Int>()

    override fun open(title: Component) {
        opened = true
        titles += title
    }

    override fun setTitle(title: Component) {
        titles += title
    }

    override fun setItem(
        slot: SlotRef,
        item: ItemStack,
    ) {
        items[slot] = item
    }

    override fun stageItem(
        slot: SlotRef,
        item: ItemStack,
    ) {
        items[slot] = item
    }

    override fun registerListeners(
        onClick: (ClickInfo) -> Unit,
        onClose: () -> Unit,
        onInput: (String) -> Unit,
        onPong: (Int) -> Unit,
    ) {
        this.onClick = onClick
        this.onClose = onClose
        this.onInput = onInput
        this.onPong = onPong
    }

    override fun ping(id: Int) {
        pings += id
    }

    override fun bundle(action: () -> Unit) {
        action()
    }

    override fun close() {
        closed = true
        listenersTorndown = true
    }

    override fun teardownListeners() {
        listenersTorndown = true
    }

    /** Simulates a client click landing on [slot]. */
    fun click(
        slot: SlotRef,
        shift: Boolean = false,
        right: Boolean = false,
    ) {
        onClick?.invoke(ClickInfo(slot, shift, right))
    }

    fun clickContainer(
        slot: Int,
        shift: Boolean = false,
        right: Boolean = false,
    ) {
        click(SlotRef(SlotArea.CONTAINER, slot), shift, right)
    }

    /** Simulates a client-initiated close. */
    fun clientClose() {
        onClose?.invoke()
    }

    /** Simulates a native inventory text-input update. */
    fun input(value: String) {
        onInput?.invoke(value)
    }

    /** Simulates the client answering the latest ping. */
    fun pong() {
        onPong?.invoke(pings.last())
    }
}

/** Builds a [WindowDefinition] from a test manifest for the given window name. */
internal fun definitionOf(
    manifest: WindowManifest,
    name: String,
): WindowDefinition = WindowDefinition(name, manifest, manifest.windows.getValue(name))

/**
 * An uninitialised [net.minestom.server.entity.Player] for headless tests.
 *
 * The session never dereferences the player — it only stores it and passes it into [Click]. We
 * allocate an instance without running its constructor; loading the `Player` class needs Minestom's
 * in-memory registries, so [net.minestom.server.MinecraftServer.init] is invoked once. `init()`
 * builds registries/managers only — it does NOT bind a socket or start the server tick loop (that
 * is `start(...)`), so tests stay fully headless.
 */
internal val stubPlayer: net.minestom.server.entity.Player by lazy {
    net.minestom.server.MinecraftServer
        .init()
    val unsafeField = sun.misc.Unsafe::class.java.getDeclaredField("theUnsafe")
    unsafeField.isAccessible = true
    val unsafe = unsafeField.get(null) as sun.misc.Unsafe
    unsafe.allocateInstance(net.minestom.server.entity.Player::class.java)
        as net.minestom.server.entity.Player
}

/** Builds a [WindowSession] over a fake handle and manual scheduler, without a live server. */
internal fun testSession(
    manifest: WindowManifest,
    name: String,
    view: WindowView,
    scheduler: ManualScheduler,
    handle: FakeInventoryHandle,
    player: net.minestom.server.entity.Player = stubPlayer,
    diagnosticsObserver: RenderDiagnosticsObserver = RenderDiagnosticsObserver.NONE,
): WindowSession =
    windowSession(
        definitionOf(manifest, name),
        view,
        player,
        scheduler,
        handle,
        diagnosticsObserver,
    )
