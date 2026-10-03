package dev.oglass.window.example

import dev.oglass.window.HudView
import dev.oglass.window.Windows
import dev.oglass.window.example.generated.WindowPack
import net.kyori.adventure.resource.ResourcePackCallback
import net.kyori.adventure.resource.ResourcePackRequest
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.TextDecoration
import net.minestom.server.MinecraftServer
import net.minestom.server.command.builder.Command
import net.minestom.server.coordinate.Pos
import net.minestom.server.entity.Player
import net.minestom.server.event.player.AsyncPlayerConfigurationEvent
import net.minestom.server.event.player.PlayerSpawnEvent
import net.minestom.server.event.player.PlayerUseItemEvent
import net.minestom.server.instance.InstanceContainer
import net.minestom.server.instance.LightingChunk
import net.minestom.server.instance.block.Block
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material
import net.minestom.server.tag.Tag
import net.minestom.server.timer.TaskSchedule
import java.nio.file.Path

/**
 * A runnable Minestom server that renders the example shop UI from generated Window pack code.
 *
 * Run after building the pack (`just plugin && cd example/pack && rpp build`):
 * ```
 * jvm/gradlew -p jvm :example:run
 * ```
 *
 * Then connect a 1.21.x client and the shop opens on spawn. Reopen it with `/shop` or the emerald in the hotbar.
 */
fun main() {
    val server = MinecraftServer.init()
    val instance = createInstance()
    val windows = WindowPack.windows()
    val market = Market()
    val serverPort = System.getProperty("window.port")?.toInt() ?: 25565
    val packPort = System.getProperty("window.pack.port")?.toInt() ?: 25567
    val packUrl = System.getProperty("window.pack.url") ?: "http://127.0.0.1:$packPort/pack.zip"
    val packServer = PackServer(resolvePackZip(), packPort, packUrl).also(PackServer::start)
    Runtime.getRuntime().addShutdownHook(Thread { packServer.stop() })

    registerPlayerEvents(instance, createPackRequest(packServer, windows, market))
    registerShopAccess(windows, market)

    server.start("0.0.0.0", serverPort)
    println("Window example server listening on $serverPort — pack served at $packUrl.")
}

private fun createInstance(): InstanceContainer =
    MinecraftServer.getInstanceManager().createInstanceContainer().apply {
        setChunkSupplier(::LightingChunk)
        setGenerator { unit -> unit.modifier().fillHeight(0, 1, Block.GRASS_BLOCK) }
    }

private fun createPackRequest(
    packServer: PackServer,
    windows: Windows,
    market: Market,
): ResourcePackRequest =
    packServer.request(
        Component.text("Window example requires its custom UI pack."),
        ResourcePackCallback.onTerminal(
            { _, audience ->
                val player = audience as? Player
                if (player != null) {
                    MinecraftServer.getSchedulerManager().scheduleNextTick {
                        if (player.isOnline) {
                            openExampleUi(player, windows, market)
                        }
                    }
                }
            },
            { _, audience ->
                (audience as? Player)?.sendMessage(
                    Component.text("The Window example resource pack did not load."),
                )
            },
        ),
    )

private fun registerPlayerEvents(
    instance: InstanceContainer,
    packRequest: ResourcePackRequest,
) {
    val events = MinecraftServer.getGlobalEventHandler()
    events.addListener(AsyncPlayerConfigurationEvent::class.java) { event ->
        event.spawningInstance = instance
        event.player.respawnPoint = Pos(0.0, 1.0, 0.0)
    }
    events.addListener(PlayerSpawnEvent::class.java) { event ->
        if (!event.isFirstSpawn) return@addListener
        event.player.inventory.setItemStack(SHOP_ITEM_SLOT, SHOP_ITEM)
        packRequest.let(event.player::sendResourcePacks)
    }
}

private val SHOP_ITEM_TAG = Tag.Boolean("window_example_shop")
private const val SHOP_ITEM_SLOT = 4

// Lazy: item stacks need the registries that MinecraftServer.init() loads.
private val SHOP_ITEM by lazy {
    ItemStack
        .builder(Material.EMERALD)
        .customName(Component.text("Foundry Exchange", NamedTextColor.GOLD).decoration(TextDecoration.ITALIC, false))
        .lore(Component.text("Use to open the market", NamedTextColor.GRAY).decoration(TextDecoration.ITALIC, false))
        .set(SHOP_ITEM_TAG, true)
        .build()
}

/** Reopens the shop from `/shop` or by using the hotbar emerald handed out on first spawn. */
private fun registerShopAccess(
    windows: Windows,
    market: Market,
) {
    val command = Command("shop")
    command.setDefaultExecutor { sender, _ -> (sender as? Player)?.let { openShop(it, windows, market) } }
    MinecraftServer.getCommandManager().register(command)
    MinecraftServer.getGlobalEventHandler().addListener(PlayerUseItemEvent::class.java) { event ->
        if (event.itemStack.getTag(SHOP_ITEM_TAG) == true) openShop(event.player, windows, market)
    }
}

// A fresh view per open: each carries its own reactive state.
private fun openShop(
    player: Player,
    windows: Windows,
    market: Market,
) = windows.open(player, MyShop(windows, market))

private fun openExampleUi(
    player: Player,
    windows: Windows,
    market: Market,
) {
    if (flag("window.hud.spriteDebug")) {
        sendHudSpriteDebug(player, windows)
    }

    openShop(player, windows, market)
    val startedAt = System.currentTimeMillis()
    val hudEnabled = flag("window.hud.enabled", default = false)
    val flowDebug = flag("window.hud.flowDebug")
    if (!hudEnabled && !flowDebug) {
        return
    }

    val huds = createStatusHuds(market, startedAt)
    if (flowDebug) {
        sendHudFlowDebug(player, windows, huds)
    }
    if (hudEnabled) {
        showHuds(player, windows, huds)
    }
}

private fun showHuds(
    player: Player,
    windows: Windows,
    huds: List<HudView>,
) {
    val sessions = huds.map { hud -> windows.show(player, hud) }
    MinecraftServer.getSchedulerManager().submitTask {
        if (!player.isOnline) {
            sessions.forEach { session -> session.hide() }
            return@submitTask TaskSchedule.stop()
        }
        sessions.forEach { session -> session.refresh() }
        TaskSchedule.seconds(1)
    }
}

private fun flag(
    name: String,
    default: Boolean = false,
): Boolean = System.getProperty(name)?.toBooleanStrictOrNull() ?: default

private fun createStatusHuds(
    market: Market,
    startedAt: Long,
): List<HudView> =
    listOf(
        MyStatusTopLeftHud(market),
        MyStatusTopCenterHud(startedAt),
        MyStatusTopRightHud(startedAt),
        MyStatusLeftSideHud(),
        MyStatusRightSideHud(market),
        MyStatusBottomCenterHud(startedAt),
    )

/** Locate the built resource-pack zip, or use `-Dwindow.pack=/path/to/window-example.zip`. */
private fun resolvePackZip(): Path {
    System.getProperty("window.pack")?.let {
        return Path.of(it)
    }
    val rel = "example/pack/dist/window-example.zip"
    var dir: Path? = Path.of("").toAbsolutePath()
    while (dir != null) {
        val candidate = dir.resolve(rel)
        if (candidate.toFile().isFile) return candidate
        dir = dir.parent
    }
    error(
        "could not find $rel relative to ${Path.of("").toAbsolutePath()} or any ancestor; " +
            "build the pack (just plugin && cd example/pack && rpp build) or pass -Dwindow.pack=…",
    )
}
