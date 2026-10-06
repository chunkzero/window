package com.chunkzero.window

import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.internal.AnvilReopenGate
import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.AnvilInputEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import net.kyori.adventure.text.Component

class AnvilInputTest :
    StringSpec({
        val manifest =
            TestManifests.manifest(
                container = "anvil",
                titleOrigin = listOf(60, 6),
                slots = mapOf("query_text" to TestManifests.slot(x = 60, width = 100, align = Align.LEFT)),
                inputs = mapOf("query" to AnvilInputEntry(slot = TestManifests.containerSlot(0))),
            )

        val static =
            TestManifests.manifest(
                container = "anvil",
                inputs = mapOf("query" to AnvilInputEntry(slot = TestManifests.containerSlot(0))),
            )

        class SearchView(
            host: FakeHost,
        ) : TestView(manifest, host) {
            val edits = mutableListOf<String>()
            var query by state("")

            override fun WindowScope<Any>.bind() {
                slot("query_text") { Component.text(query) }
                anvilInput("query") {
                    edits += it
                    query = it
                }
            }

            fun clear() = input("query", "")
        }

        fun FakeContainer.seed() = items.getValue(SlotRef(SlotArea.CONTAINER, 0)) as WindowItem.AnvilSeed

        fun FakeContainer.seedName() = seed().text

        "static anvils deliver edits without reopening and set the edit box in place" {
            val edits = mutableListOf<String>()
            val host = FakeHost()
            val view =
                object : TestView(static, host) {
                    override fun WindowScope<Any>.bind() {
                        anvilInput("query") { edits += it }
                    }

                    fun clear() = input("query", "")
                }
            val handle = host.container
            view.open()
            handle.input("")
            handle.input("a")
            handle.input("as")
            handle.seedName() shouldBe "as"
            view.clear()
            handle.input("")

            edits shouldBe listOf("a", "as", "")
            handle.seedName() shouldBe ""
            handle.titles.size shouldBe 1
        }

        "setting a static input resends a changed seed, capped at vanilla's name length" {
            val edits = mutableListOf<String>()
            val host = FakeHost()
            val view =
                object : TestView(static, host) {
                    override fun WindowScope<Any>.bind() {
                        anvilInput("query") {
                            edits += it
                            input("query", it.trim())
                        }
                    }

                    fun set(value: String) = input("query", value)
                }
            val handle = host.container
            view.open()
            val opened = handle.items.getValue(SlotRef(SlotArea.CONTAINER, 0))
            handle.input(" a ")
            view.set("")
            handle.items.getValue(SlotRef(SlotArea.CONTAINER, 0)) shouldNotBe opened
            handle.seedName() shouldBe ""

            view.set("x".repeat(60))
            edits shouldBe listOf(" a ", "a", "", "x".repeat(50))
        }

        "setting the input of a reopening anvil delivers it and reopens with it" {
            val host = FakeHost()
            val view = SearchView(host)
            val scheduler = host.scheduler
            val handle = host.container
            view.open()
            handle.input("")
            handle.pong()

            handle.input("ab")
            scheduler.runAll()
            handle.pong()
            view.clear()
            scheduler.runAll()

            view.edits shouldBe listOf("ab", "")
            handle.titles.size shouldBe 3
            handle.seedName() shouldBe ""
        }

        "a binding that normalizes a reopening anvil's input settles" {
            val edits = mutableListOf<String>()
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    override fun WindowScope<Any>.bind() {
                        slot("query_text") { Component.empty() }
                        anvilInput("query") {
                            edits += it
                            input("query", it.trim())
                        }
                    }
                }
            val handle = host.container
            view.open()
            handle.input("")
            handle.pong()
            handle.input(" a ")

            edits shouldBe listOf(" a ", "a")
            handle.seedName() shouldBe "a"
        }

        "title changes wait until the player pauses typing" {
            val host = FakeHost()
            val view = SearchView(host)
            val scheduler = host.scheduler
            val handle = host.container
            view.open()
            handle.input("")
            handle.pong()

            handle.input("d")
            scheduler.tick()
            handle.input("di")
            scheduler.tick()
            handle.input("din")
            scheduler.tick()
            handle.titles.size shouldBe 1

            scheduler.runAll()
            handle.titles.size shouldBe 2
            view.edits shouldBe listOf("d", "di", "din")
            handle.seedName() shouldBe "din"
        }

        "reopen echoes are not edits, and edits typed on a rewound edit box are rebased" {
            val host = FakeHost()
            val view = SearchView(host)
            val scheduler = host.scheduler
            val handle = host.container
            view.open()
            handle.input("")
            handle.pong()

            handle.input("a")
            scheduler.runAll()
            handle.titles.size shouldBe 2

            // Typed on the old screen before the "a" reopen lands, then the reopen's echo.
            handle.input("as")
            handle.input("a")
            handle.pong()
            // Typed on the box the reopen rewound to "a".
            handle.input("ad")
            scheduler.runAll()
            handle.titles.size shouldBe 3

            handle.input("asd")
            handle.pong()
            scheduler.runAll()
            handle.titles.size shouldBe 3
            view.edits shouldBe listOf("a", "as", "asd")
            handle.seedName() shouldBe "asd"
        }

        "an edit equal to the in-flight seed is delivered, and the trailing echo is dropped" {
            val host = FakeHost()
            val view = SearchView(host)
            val scheduler = host.scheduler
            val handle = host.container
            view.open()
            handle.input("")
            handle.pong()

            handle.input("d")
            scheduler.runAll()
            handle.input("di")
            handle.input("d")
            handle.input("d")
            handle.pong()
            scheduler.runAll()

            view.edits shouldBe listOf("d", "di", "d")
            handle.seedName() shouldBe "d"
        }

        "stale edits rebase onto the latest input around the text the edit box lost" {
            AnvilReopenGate.rebase("a", "ad", "as") shouldBe "asd"
            AnvilReopenGate.rebase("a", "", "as") shouldBe "s"
            AnvilReopenGate.rebase("a", "Xa", "as") shouldBe "Xas"
            AnvilReopenGate.rebase("ab", "a", "axb") shouldBe "ax"
            AnvilReopenGate.rebase("ab", "aXb", "ayb") shouldBe "ayXb"
            AnvilReopenGate.rebase("ab", "aZb", "aXbY") shouldBe "aXZbY"
        }

        "edits held for a pending reopen reach the view before a close" {
            val host = FakeHost()
            val view = SearchView(host)
            val scheduler = host.scheduler
            val handle = host.container
            view.open()
            handle.input("")
            handle.pong()

            handle.input("d")
            scheduler.runAll()
            handle.input("di")
            handle.clientClose()

            view.edits shouldBe listOf("d", "di")
        }

        "a held edit that closes the window ends the client close" {
            var closes = 0
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    override fun WindowScope<Any>.bind() {
                        slot("query_text") { Component.empty() }
                        anvilInput("query") { if (it == "x") close() }
                    }

                    override fun onClose() {
                        closes++
                    }
                }
            val scheduler = host.scheduler
            val handle = host.container
            view.open()
            handle.input("x")
            handle.clientClose()

            closes shouldBe 1
        }
    })
