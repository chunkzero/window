package com.chunkzero.window

import com.chunkzero.window.manifest.Align
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import net.kyori.adventure.text.Component

class StateTrackingTest :
    StringSpec({
        fun twoSlotManifest() =
            TestManifests.manifest(
                container = "generic_9x3",
                slots =
                    mapOf(
                        "a" to TestManifests.slot(x = 8, width = 50, align = Align.LEFT),
                        "b" to TestManifests.slot(x = 8, width = 50, align = Align.LEFT),
                    ),
            )

        "mutating a state re-evaluates only the dependent slot, one schedule per burst" {
            var aRenders = 0
            var bRenders = 0

            val host = FakeHost()
            val view =
                object : TestView(twoSlotManifest(), host) {
                    var countA by state(0)
                    var countB by state(0)

                    override fun WindowScope<Any>.bind() {
                        slot("a") {
                            aRenders++
                            Component.text("a$countA")
                        }
                        slot("b") {
                            bRenders++
                            Component.text("b$countB")
                        }
                    }
                }

            val scheduler = host.scheduler
            val handle = host.container
            view.open()

            // Open seeds both slots once.
            aRenders shouldBe 1
            bRenders shouldBe 1
            scheduler.scheduleCount shouldBe 0

            // Mutate state read by slot "a" only.
            view.countA = 1
            // A single flush is scheduled.
            scheduler.scheduleCount shouldBe 1
            scheduler.pending shouldBe 1

            scheduler.runAll()
            // Only slot "a" re-rendered.
            aRenders shouldBe 2
            bRenders shouldBe 1
            // A title update was sent.
            handle.titles.size shouldBe 2 // open + one re-render
        }

        "multiple writes in one burst coalesce into a single schedule" {
            val host = FakeHost()
            val view =
                object : TestView(twoSlotManifest(), host) {
                    var c by state(0)

                    override fun WindowScope<Any>.bind() {
                        slot("a") { Component.text("a$c") }
                        slot("b") { Component.text("b") }
                    }
                }
            val scheduler = host.scheduler
            view.open()

            view.c = 1
            view.c = 2
            view.c = 3
            scheduler.scheduleCount shouldBe 1
            scheduler.runAll()
        }

        "a state never read by any slot triggers no re-render" {
            val host = FakeHost()
            val view =
                object : TestView(twoSlotManifest(), host) {
                    var unused by state(0)

                    override fun WindowScope<Any>.bind() {
                        slot("a") { Component.text("a") }
                        slot("b") { Component.text("b") }
                    }
                }
            val scheduler = host.scheduler
            val handle = host.container
            view.open()

            view.unused = 99
            scheduler.scheduleCount shouldBe 0
            handle.titles.size shouldBe 1 // only the open
        }

        "refresh marks all slots dirty with a single schedule" {
            var aRenders = 0
            var bRenders = 0
            val host = FakeHost()
            val view =
                object : TestView(twoSlotManifest(), host) {
                    fun forceRefresh() = refresh()

                    override fun WindowScope<Any>.bind() {
                        slot("a") {
                            aRenders++
                            Component.text("a")
                        }
                        slot("b") {
                            bRenders++
                            Component.text("b")
                        }
                    }
                }
            val scheduler = host.scheduler
            view.open()
            aRenders shouldBe 1
            bRenders shouldBe 1

            view.forceRefresh()
            scheduler.scheduleCount shouldBe 1
            scheduler.runAll()
            aRenders shouldBe 2
            bRenders shouldBe 2
        }
    })
