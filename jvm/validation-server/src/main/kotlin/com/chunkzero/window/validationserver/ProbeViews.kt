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
) : WindowView<ItemStack>(WindowDefinition(manifest, "probe"), MinestomHost.of(player)) {
    private var text by state("III×")
    private var searchOpened = false

    override fun WindowScope<ItemStack>.bind() {
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
) : WindowView<ItemStack>(WindowDefinition(manifest, "search_probe"), MinestomHost.of(player)) {
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

internal class ProbeHudView(
    manifest: WindowManifest,
    player: Player,
    private val report: RuntimeReport,
) : HudView(HudDefinition(manifest, "probe_hud"), MinestomHost.of(player)) {
    override fun HudScope.bind() {}

    override fun onShow() {
        report.record("hud.shown", mapOf("hud" to "probe_hud"))
    }
}
