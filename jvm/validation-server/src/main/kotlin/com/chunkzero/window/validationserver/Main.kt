package com.chunkzero.window.validationserver

import com.chunkzero.window.Windows
import com.chunkzero.window.diagnostics.PackFingerprint
import com.chunkzero.window.diagnostics.minestom.MinestomDiagnostics
import com.chunkzero.window.manifest.WindowManifest
import dev.rpp.mcvalidation.minestom.MinestomValidation
import dev.rpp.mcvalidation.minestom.ValidationRoutes
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import net.kyori.adventure.text.Component
import net.minestom.server.MinecraftServer
import net.minestom.server.coordinate.Pos
import net.minestom.server.entity.Player
import net.minestom.server.event.player.AsyncPlayerConfigurationEvent
import net.minestom.server.event.player.PlayerDisconnectEvent
import net.minestom.server.event.player.PlayerLoadedEvent
import net.minestom.server.instance.Instance
import net.minestom.server.instance.block.Block
import java.nio.file.Files
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

/** Disposable Minestom process used by Window's real-client validation harness. */
fun main(args: Array<String>) {
    val config = Config.parse(args)
    val report = RuntimeReport(config.report)
    val server = MinecraftServer.init()
    val instance = createInstance()
    val diagnostics = installDiagnostics(config)
    val windows = Windows.load(WindowManifest.parse(Files.readString(config.manifest)), diagnostics)
    installValidationRoutes(report)
    installListeners(instance, windows, report)
    installShutdownHook(diagnostics, report)
    server.start("127.0.0.1", config.port)
    report.record("server.ready", mapOf("port" to config.port.toString()))
    println("WINDOW_VALIDATION_SERVER_READY ${config.port}")
}

private fun createInstance(): Instance =
    MinecraftServer.getInstanceManager().createInstanceContainer().apply {
        setGenerator { unit -> unit.modifier().fillHeight(0, 1, Block.STONE) }
    }

private fun installDiagnostics(config: Config): MinestomDiagnostics {
    val descriptor = Json.parseToJsonElement(Files.readString(config.descriptor)).jsonObject
    val fingerprint = descriptor.getValue("pack_fingerprint").jsonObject
    return MinestomDiagnostics.install(
        PackFingerprint(
            fingerprint.getValue("algorithm").jsonPrimitive.content,
            fingerprint.getValue("value").jsonPrimitive.content,
        ),
    )
}

private fun installValidationRoutes(report: RuntimeReport) {
    val validation =
        MinestomValidation.installWithEvents(
            MinecraftServer.process(),
            ValidationRoutes
                .builder()
                .route("ping") { invocation ->
                    invocation.player.sendMessage(Component.text("Window validation ready"))
                    invocation.events.emit("window.ping", mapOf("result" to "ready"))
                }.build(),
        )
    report.attachValidationEvents(validation.events())
}

private fun installListeners(
    instance: Instance,
    windows: Windows,
    report: RuntimeReport,
) {
    val openedPlayers = ConcurrentHashMap.newKeySet<UUID>()
    MinecraftServer.getGlobalEventHandler().apply {
        addListener(AsyncPlayerConfigurationEvent::class.java) { event ->
            event.spawningInstance = instance
            event.player.respawnPoint = Pos(0.0, 2.0, 0.0)
        }
        addListener(PlayerLoadedEvent::class.java) { event ->
            if (!openedPlayers.add(event.player.uuid)) return@addListener
            report.record("player.loaded", mapOf("player" to event.player.username))
            openProbeFlow(windows, event.player, report)
        }
        addListener(PlayerDisconnectEvent::class.java) { event ->
            openedPlayers.remove(event.player.uuid)
            report.record("player.disconnected", mapOf("player" to event.player.username))
        }
    }
}

private fun openProbeFlow(
    windows: Windows,
    player: Player,
    report: RuntimeReport,
) {
    windows.open(
        player,
        ProbeView(report) { probePlayer ->
            MinecraftServer.getSchedulerManager().scheduleNextTick {
                if (probePlayer.isOnline) {
                    windows.open(
                        probePlayer,
                        SearchProbeView(report) { searchPlayer ->
                            MinecraftServer.getSchedulerManager().scheduleNextTick {
                                if (searchPlayer.isOnline) {
                                    windows.show(searchPlayer, ProbeHudView(report))
                                }
                            }
                        },
                    )
                }
            }
        },
    )
}

private fun installShutdownHook(
    diagnostics: MinestomDiagnostics,
    report: RuntimeReport,
) {
    Runtime
        .getRuntime()
        .addShutdownHook(
            Thread {
                diagnostics.close()
                report.record("server.stopped", emptyMap())
            },
        )
}
