package dev.oglass.window.example

import dev.oglass.window.HudView
import dev.oglass.window.Windows
import dev.oglass.window.example.generated.WindowPack
import net.kyori.adventure.resource.ResourcePackCallback
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.minestom.server.MinecraftServer
import net.minestom.server.coordinate.Pos
import net.minestom.server.event.player.AsyncPlayerConfigurationEvent
import net.minestom.server.event.player.PlayerSpawnEvent
import net.minestom.server.instance.block.Block
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
 * Then connect a 1.21.x client and the shop opens on spawn.
 */
fun main() {
    val server = MinecraftServer.init()

    val instance =
        MinecraftServer.getInstanceManager().createInstanceContainer().apply {
            setGenerator { unit -> unit.modifier().fillHeight(0, 1, Block.GRASS_BLOCK) }
        }

    val windows = WindowPack.windows()
    val market = Market()
    val serverPort = System.getProperty("window.port")?.toInt() ?: 25565
    val packPort = System.getProperty("window.pack.port")?.toInt() ?: 25567
    val packUrl = System.getProperty("window.pack.url") ?: "http://127.0.0.1:$packPort/pack.zip"
    val packServer = PackServer(resolvePackZip(), packPort, packUrl).also(PackServer::start)
    val packRequest =
        packServer.request(
            Component.text("Window example requires its custom UI pack."),
            ResourcePackCallback.onTerminal(
                { _, audience ->
                    val player = audience as? net.minestom.server.entity.Player
                    if (player != null) {
                        MinecraftServer.getSchedulerManager().scheduleNextTick {
                            if (player.isOnline) {
                                openExampleUi(player, windows, market)
                            }
                        }
                    }
                },
                { _, audience ->
                    (audience as? net.minestom.server.entity.Player)?.sendMessage(
                        Component.text("The Window example resource pack did not load."),
                    )
                },
            ),
        )
    Runtime.getRuntime().addShutdownHook(Thread { packServer.stop() })

    val events = MinecraftServer.getGlobalEventHandler()
    events.addListener(AsyncPlayerConfigurationEvent::class.java) { event ->
        event.spawningInstance = instance
        event.player.respawnPoint = Pos(0.0, 1.0, 0.0)
    }
    events.addListener(PlayerSpawnEvent::class.java) { event ->
        if (!event.isFirstSpawn) return@addListener
        packRequest.let(event.player::sendResourcePacks)
    }

    server.start("0.0.0.0", serverPort)
    println("Window example server listening on $serverPort — pack served at $packUrl.")
}

private fun openExampleUi(
    player: net.minestom.server.entity.Player,
    windows: Windows,
    market: Market,
) {
    if (flag("window.hud.spriteDebug")) {
        sendHudSpriteDebug(player, windows)
    }

    // A fresh view per open: each carries its own reactive state.
    windows.open(player, MyShop(windows, market))
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
    if (!hudEnabled) {
        return
    }

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
        MyStatusBottomCenterHud(),
    )

private fun sendHudSpriteDebug(
    player: net.minestom.server.entity.Player,
    windows: Windows,
) {
    player.sendMessage(Component.text("HUD sprite debug (chat, window:ui font):"))
    for (hudName in HUD_DEBUG_NAMES) {
        player.sendMessage(Component.text(hudName, NamedTextColor.GRAY))
        player.sendMessage(windows.hud(hudName).staticHud.color(NamedTextColor.WHITE))
        repeat(8) { player.sendMessage(Component.text(" ")) }
    }
}

private fun sendHudFlowDebug(
    player: net.minestom.server.entity.Player,
    windows: Windows,
    huds: List<HudView>,
) {
    player.sendMessage(Component.text("HUD flow debug (chat, composed via Windows.renderHud):"))
    for (hud in huds) {
        player.sendMessage(Component.text(hud.hudName, NamedTextColor.GRAY))
        player.sendMessage(windows.renderHud(player, hud))
        repeat(8) { player.sendMessage(Component.text(" ")) }
    }
}

private val HUD_DEBUG_NAMES =
    listOf(
        "status_top_center",
        "status_top_left",
        "status_top_right",
        "status_left_side",
        "status_right_side",
        "status_bottom_center",
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
