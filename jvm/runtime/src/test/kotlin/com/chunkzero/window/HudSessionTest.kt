package com.chunkzero.window

import com.chunkzero.window.host.HudChannel
import com.chunkzero.window.manifest.Align
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.collections.shouldContainExactly
import io.kotest.matchers.shouldBe
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer
import java.lang.ref.WeakReference

/** Runs the garbage collector until [until] holds, or a few rounds passed. */
private fun collectGarbage(until: () -> Boolean = { false }) {
    val marker = WeakReference(Any())
    repeat(20) {
        System.gc()
        if (marker.get() == null && until()) return
        Thread.sleep(10)
    }
}

class HudSessionTest :
    StringSpec({
        /** The HUD's text without the fixture's static art and spacers. */
        fun text(component: Component): String =
            PlainTextComponentSerializer
                .plainText()
                .serialize(component)
                .replace("STATIC", "")
                .filter(Char::isLetter)

        fun manifest(channel: String) =
            TestManifests.hudManifest(
                channel = channel,
                slots = mapOf("text" to TestManifests.slot(0, 40, Align.LEFT)),
            )

        class Label(
            manifest: com.chunkzero.window.manifest.WindowManifest,
            host: FakeHost,
            private val text: String,
        ) : TestHud(manifest, host) {
            override fun HudScope.bind() {
                slot("text") { Component.text(text) }
            }

            fun close() = hide()
        }

        "action-bar HUDs of one host share one output until the last one hides" {
            val host = FakeHost()
            val first = Label(manifest("actionbar"), host, "A")
            val second = Label(manifest("actionbar"), host, "B")
            first.show()
            second.show()

            val output = host.huds.single()
            output.descriptor.channel shouldBe HudChannel.ACTION_BAR
            text(output.contents.last()) shouldBe "AB"

            first.close()
            text(output.contents.last()) shouldBe "B"
            output.hidden shouldBe false
            second.close()
            output.hidden shouldBe true
        }

        "a shown action-bar HUD stays merged after its view and session are collected" {
            val host = FakeHost()
            Label(manifest("actionbar"), host, "A").show()
            collectGarbage()
            Label(manifest("actionbar"), host, "B").show()

            text(
                host.huds
                    .single()
                    .contents
                    .last(),
            ) shouldBe "AB"
        }

        "a shown action-bar HUD does not keep its host reachable" {
            var host: FakeHost? = FakeHost()
            val reference = WeakReference(host)
            Label(manifest("actionbar"), host!!, "A").show()
            host = null

            collectGarbage(until = { reference.get() == null })

            reference.get() shouldBe null
        }

        "other channels get their own output, updated in place and hidden once" {
            val host = FakeHost()
            val hud =
                object : TestHud(manifest("bossbar"), host) {
                    var text by state("A")

                    override fun HudScope.bind() {
                        slot("text") { Component.text(text) }
                    }

                    fun close() = hide()
                }
            val session = hud.show()
            hud.text = "B"
            host.scheduler.runAll()
            hud.close()
            session.hide()

            val output = host.huds.single()
            output.descriptor.channel shouldBe HudChannel.BOSS_BAR
            output.contents.map(::text) shouldContainExactly listOf("A", "B")
            output.hidden shouldBe true
        }
    })
