package com.chunkzero.window

import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.SwitchCaseEntry
import com.chunkzero.window.manifest.SwitchEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.string.shouldContain
import io.kotest.matchers.string.shouldNotContain
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer

class SwitchBindingTest :
    StringSpec({
        fun plain(component: Component): String = PlainTextComponentSerializer.plainText().serialize(component)

        val modeSwitch =
            SwitchEntry(
                listOf(
                    SwitchCaseEntry("buy", static = "BUYART", slots = listOf("price")),
                    SwitchCaseEntry("sell", static = "SELLART", slots = listOf("sell_label")),
                ),
            )

        "window title draws only the active case's art and slots" {
            val manifest =
                TestManifests.manifest(
                    slots =
                        mapOf(
                            "price" to TestManifests.slot(8, 40, Align.LEFT),
                            "sell_label" to TestManifests.slot(8, 40, Align.LEFT, text = "Sell"),
                        ),
                    switches = mapOf("mode" to modeSwitch),
                )
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    var mode by state("buy")

                    override fun WindowScope<Any>.bind() {
                        slot("price") { Component.text("Ten") }
                        switch("mode") { mode }
                    }
                }
            val scheduler = host.scheduler
            val handle = host.container
            view.open()

            val opened = plain(handle.titles.last())
            opened shouldContain "BUYART"
            opened shouldContain "Ten"
            opened shouldNotContain "SELLART"
            opened shouldNotContain "Sell"

            view.mode = "sell"
            scheduler.runAll()
            val swapped = plain(handle.titles.last())
            swapped shouldContain "SELLART"
            swapped shouldContain "Sell"
            swapped shouldNotContain "BUYART"
            swapped shouldNotContain "Ten"
        }

        "hud redraws the active case on update" {
            val show =
                SwitchEntry(
                    listOf(
                        SwitchCaseEntry("true", static = "BADGE", slots = listOf("sale")),
                        SwitchCaseEntry("false"),
                    ),
                )
            val manifest =
                TestManifests.hudManifest(
                    slots = mapOf("sale" to TestManifests.slot(10, 40, Align.LEFT, text = "Sale")),
                    switches = mapOf("on_sale" to show),
                )
            val view =
                object : TestHud(manifest) {
                    var onSale = false

                    override fun HudScope.bind() {
                        switch("on_sale") { onSale.toString() }
                    }
                }

            plain(view.render()) shouldNotContain "BADGE"
            plain(view.render()) shouldNotContain "Sale"
            view.onSale = true
            plain(view.render()) shouldContain "BADGE"
            plain(view.render()) shouldContain "Sale"
        }
    })
