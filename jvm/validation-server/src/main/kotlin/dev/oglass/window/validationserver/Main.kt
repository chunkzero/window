package dev.oglass.window.validationserver

import dev.oglass.window.Click
import dev.oglass.window.HudScope
import dev.oglass.window.HudView
import dev.oglass.window.WindowScope
import dev.oglass.window.WindowView
import dev.oglass.window.Windows
import dev.oglass.window.diagnostics.PackFingerprint
import dev.oglass.window.diagnostics.minestom.MinestomDiagnostics
import dev.oglass.window.manifest.WindowManifest
import dev.rpp.mcvalidation.minestom.MinestomValidation
import dev.rpp.mcvalidation.minestom.ValidationEventSink
import dev.rpp.mcvalidation.minestom.ValidationRoutes
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import net.kyori.adventure.text.Component
import net.minestom.server.MinecraftServer
import net.minestom.server.coordinate.Pos
import net.minestom.server.event.player.AsyncPlayerConfigurationEvent
import net.minestom.server.event.player.PlayerDisconnectEvent
import net.minestom.server.event.player.PlayerLoadedEvent
import net.minestom.server.instance.block.Block
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import java.util.concurrent.ConcurrentHashMap

/** Disposable Minestom process used by Window's real-client validation harness. */
fun main(args: Array<String>) {
    val config = Config.parse(args)
    val report = RuntimeReport(config.report)
    val server = MinecraftServer.init()
    val instance =
        MinecraftServer.getInstanceManager().createInstanceContainer().apply {
            setGenerator { unit -> unit.modifier().fillHeight(0, 1, Block.STONE) }
        }
    val descriptor = Json.parseToJsonElement(Files.readString(config.descriptor)).jsonObject
    val fingerprint = descriptor.getValue("pack_fingerprint").jsonObject
    val diagnostics =
        MinestomDiagnostics.install(
            PackFingerprint(
                fingerprint.getValue("algorithm").jsonPrimitive.content,
                fingerprint.getValue("value").jsonPrimitive.content,
            ),
        )
    val windows = Windows.load(WindowManifest.parse(Files.readString(config.manifest)), diagnostics)
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
    val openedPlayers = ConcurrentHashMap.newKeySet<java.util.UUID>()

    MinecraftServer.getGlobalEventHandler().apply {
        addListener(AsyncPlayerConfigurationEvent::class.java) { event ->
            event.spawningInstance = instance
            event.player.respawnPoint = Pos(0.0, 2.0, 0.0)
        }
        addListener(PlayerLoadedEvent::class.java) { event ->
            if (!openedPlayers.add(event.player.uuid)) return@addListener
            report.record("player.loaded", mapOf("player" to event.player.username))
            windows.open(
                event.player,
                ProbeView(report) { player ->
                    MinecraftServer.getSchedulerManager().scheduleNextTick {
                        if (player.isOnline) {
                            windows.open(
                                player,
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
        addListener(PlayerDisconnectEvent::class.java) { event ->
            openedPlayers.remove(event.player.uuid)
            report.record("player.disconnected", mapOf("player" to event.player.username))
        }
    }

    Runtime
        .getRuntime()
        .addShutdownHook(
            Thread {
                diagnostics.close()
                report.record("server.stopped", emptyMap())
            },
        )
    server.start("127.0.0.1", config.port)
    report.record("server.ready", mapOf("port" to config.port.toString()))
    println("WINDOW_VALIDATION_SERVER_READY ${config.port}")
}

private class ProbeView(
    private val report: RuntimeReport,
    private val openSearch: (net.minestom.server.entity.Player) -> Unit,
) : WindowView("probe") {
    private var text by state("III×")
    private var searchOpened = false

    override fun WindowScope.bind() {
        slot("probe_text") { Component.text(text) }
        sprite("probe_sprite") { "probe_marker" }
        collectionItem("probe_collection") { null }
        // One repeater cell shows a real ItemStack in its own cell slot; the
        // cell button still owns every click in the cell, including the item's
        // slot.
        item("probe_card_icon_0") { ItemStack.of(Material.DIAMOND) }
        item("probe_card_icon_1") { ItemStack.of(Material.EMERALD) }
        button("probe_card_0") { click -> receiveCard(0, click) }
        button("probe_card_1") { click -> receiveCard(1, click) }
        button("probe_container_button") { click -> receive("container", click) }
        button("probe_hotbar_button") { click -> receive("hotbar", click) }
    }

    override fun onOpen() {
        report.record("window.opened", mapOf("window" to windowName, "text" to text))
    }

    override fun onClose() {
        report.record("window.closed", mapOf("window" to windowName))
    }

    private fun receiveCard(
        cell: Int,
        click: Click,
    ) {
        report.record(
            "card.clicked",
            mapOf(
                "cell" to cell.toString(),
                "area" to click.area.name.lowercase(),
                "slot" to click.inventorySlot.toString(),
            ),
        )
    }

    private fun receive(
        control: String,
        click: Click,
    ) {
        report.record(
            "click.received",
            mapOf(
                "control" to control,
                "area" to click.area.name.lowercase(),
                "slot" to click.inventorySlot.toString(),
                "shift" to click.shift.toString(),
                "right" to click.right.toString(),
            ),
        )
        if (text != "WWW×") {
            text = "WWW×"
            report.record("title.invalidated", mapOf("text" to text))
        }
        if (control == "hotbar" && !searchOpened) {
            searchOpened = true
            report.record("search.requested", emptyMap())
            openSearch(click.player)
        }
    }
}

private class SearchProbeView(
    private val report: RuntimeReport,
    private val showHud: (net.minestom.server.entity.Player) -> Unit,
) : WindowView("search_probe") {
    private var hudRequested = false

    override fun WindowScope.bind() {
        anvilInput("query") { value ->
            report.record("input.changed", mapOf("input" to "query", "value" to value))
        }
    }

    override fun onOpen() {
        report.record("search.opened", mapOf("window" to windowName))
    }

    override fun onClose() {
        if (hudRequested) return
        hudRequested = true
        report.record("hud.requested", emptyMap())
        showHud(player)
    }
}

private class ProbeHudView(
    private val report: RuntimeReport,
) : HudView("probe_hud") {
    override fun HudScope.bind() {}

    override fun onShow() {
        report.record("hud.shown", mapOf("hud" to hudName))
    }
}

private data class Config(
    val port: Int,
    val manifest: Path,
    val descriptor: Path,
    val report: Path,
) {
    companion object {
        fun parse(args: Array<String>): Config {
            val values =
                args.asList().chunked(2).associate { pair ->
                    require(pair.size == 2 && pair[0].startsWith("--")) {
                        "expected --port, --manifest, and --report arguments"
                    }
                    pair[0].removePrefix("--") to pair[1]
                }
            return Config(
                values.getValue("port").toInt(),
                Path.of(values.getValue("manifest")),
                Path.of(values.getValue("descriptor")),
                Path.of(values.getValue("report")),
            )
        }
    }
}

@Serializable
private data class ReportEvent(
    val sequence: Int,
    val name: String,
    val details: Map<String, String>,
)

@Serializable
private data class ReportDocument(
    @SerialName("schema_version") val schemaVersion: Int = 1,
    val target: String = "window-minestom-runtime",
    val events: List<ReportEvent>,
)

private class RuntimeReport(
    private val path: Path,
) {
    private val events = mutableListOf<ReportEvent>()
    private var validationEvents: ValidationEventSink? = null
    private val json =
        Json {
            prettyPrint = true
            encodeDefaults = true
        }

    @Synchronized
    fun record(
        name: String,
        details: Map<String, String>,
    ) {
        events += ReportEvent(events.size + 1, name, details.toSortedMap())
        validationEvents?.emit(name, details)
        Files.createDirectories(path.parent)
        val temporary = path.resolveSibling("${path.fileName}.tmp")
        Files.writeString(temporary, json.encodeToString(ReportDocument(events = events)))
        Files.move(
            temporary,
            path,
            StandardCopyOption.REPLACE_EXISTING,
            StandardCopyOption.ATOMIC_MOVE,
        )
    }

    fun attachValidationEvents(events: ValidationEventSink) {
        validationEvents = events
    }
}
