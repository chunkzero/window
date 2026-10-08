package com.chunkzero.window.validationserver

import com.chunkzero.window.Click
import com.chunkzero.window.HudDefinition
import com.chunkzero.window.HudScope
import com.chunkzero.window.HudView
import com.chunkzero.window.WindowDefinition
import com.chunkzero.window.WindowScope
import com.chunkzero.window.WindowView
import com.chunkzero.window.manifest.WindowManifest
import com.chunkzero.window.minestom.MinestomHost
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material

internal class ProbeView(
    manifest: WindowManifest,
    private val player: Player,
    private val report: RuntimeReport,
    private val openSearch: (Player) -> Unit,
) : WindowView<ItemStack>(WindowDefinition(manifest, "probe"), MinestomHost(player)) {
    private var text by state("III×")
    private var searchOpened = false

    override fun WindowScope<ItemStack>.bind() {
        slot("probe_text") { Component.text(text) }
        sprite("probe_sprite") { "probe_marker" }
        collectionItem("probe_collection") { null }
        // Each card shows a real ItemStack in one of its slots; the card's region still routes every click in
        // the card, including the item's slot.
        val icons = listOf(Material.DIAMOND, Material.EMERALD)
        for (cell in icons.indices) {
            item("probe_card_icon[$cell]") { ItemStack.of(icons[cell]) }
            button("probe_card[$cell]") { click -> receiveCard(cell, click) }
        }
        button("probe_container_button") { click -> receive("container", click) }
        button("probe_hotbar_button") { click -> receive("hotbar", click) }
    }

    override fun onOpen() {
        report.record("window.opened", mapOf("window" to "probe", "text" to text))
    }

    override fun onClose() {
        report.record("window.closed", mapOf("window" to "probe"))
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
            openSearch(player)
        }
    }
}

internal class SearchProbeView(
    manifest: WindowManifest,
    private val player: Player,
    private val report: RuntimeReport,
    private val showHud: (Player) -> Unit,
) : WindowView<ItemStack>(WindowDefinition(manifest, "search_probe"), MinestomHost(player)) {
    private var hudRequested = false

    override fun WindowScope<ItemStack>.bind() {
        anvilInput("query") { value ->
            report.record("input.changed", mapOf("input" to "query", "value" to value))
        }
    }

    override fun onOpen() {
        report.record("search.opened", mapOf("window" to "search_probe"))
    }

    override fun onClose() {
        if (hudRequested) return
        hudRequested = true
        report.record("hud.requested", emptyMap())
        showHud(player)
    }
}

/** The probe HUD shown to [player]; its diagnostics frames are routed to that player. */
internal class ProbeHudView(
    manifest: WindowManifest,
    val player: Player,
) : HudView(HudDefinition(manifest, "probe_hud")) {
    override fun HudScope.bind() {}
}
