package dev.oglass.window

import dev.oglass.window.manifest.Align
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

            val view =
                object : WindowView("w") {
                    var countA by state(0)
                    var countB by state(0)

                    override fun WindowScope.bind() {
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

            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            val session = testSession(twoSlotManifest(), "w", view, scheduler, handle)
            session.open()

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
            val view =
                object : WindowView("w") {
                    var c by state(0)

                    override fun WindowScope.bind() {
                        slot("a") { Component.text("a$c") }
                        slot("b") { Component.text("b") }
                    }
                }
            val scheduler = ManualScheduler()
            val session =
                testSession(twoSlotManifest(), "w", view, scheduler, FakeInventoryHandle())
            session.open()

            view.c = 1
            view.c = 2
            view.c = 3
            scheduler.scheduleCount shouldBe 1
            scheduler.runAll()
        }

        "a state never read by any slot triggers no re-render" {
            val view =
                object : WindowView("w") {
                    var unused by state(0)

                    override fun WindowScope.bind() {
                        slot("a") { Component.text("a") }
                        slot("b") { Component.text("b") }
                    }
                }
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(twoSlotManifest(), "w", view, scheduler, handle).open()

            view.unused = 99
            scheduler.scheduleCount shouldBe 0
            handle.titles.size shouldBe 1 // only the open
        }

        "refresh marks all slots dirty with a single schedule" {
            var aRenders = 0
            var bRenders = 0
            val view =
                object : WindowView("w") {
                    fun forceRefresh() = refresh()

                    override fun WindowScope.bind() {
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
            val scheduler = ManualScheduler()
            testSession(twoSlotManifest(), "w", view, scheduler, FakeInventoryHandle()).open()
            aRenders shouldBe 1
            bRenders shouldBe 1

            view.forceRefresh()
            scheduler.scheduleCount shouldBe 1
            scheduler.runAll()
            aRenders shouldBe 2
            bRenders shouldBe 2
        }
    })
