package com.chunkzero.window

import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.ButtonDefault
import com.chunkzero.window.manifest.SlotAreaEntry
import com.chunkzero.window.manifest.WindowManifest
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.nulls.shouldBeNull
import io.kotest.matchers.nulls.shouldNotBeNull
import io.kotest.matchers.shouldBe

class ManifestParseTest :
    StringSpec({
        "full sample round-trips into DTOs" {
            val m = WindowManifest.parse(TestManifests.sampleJson)
            m.version shouldBe 6
            m.namespace shouldBe "window"
            m.font shouldBe "window:ui"
            m.spacers[983040] shouldBe -1024
            m.spacers[983061] shouldBe 1024
            m.textAdvances[" "] shouldBe 4
            m.textAdvances["~"] shouldBe 7
            m.textGlyphWidths[" "] shouldBe 0
            m.textGlyphWidths["~"] shouldBe 6
            m.fontMetrics.getValue("window:y0").boldAdvance shouldBe 1

            val shop = m.windows.getValue("shop")
            shop.surface.kind shouldBe "container"
            shop.surface.container shouldBe "generic_9x6"
            shop.surface.size shouldBe listOf(176, 222)
            shop.surface.titleOrigin shouldBe listOf(8, 6)
            shop.static shouldBe "STATIC_CHROME"

            val title = shop.slots.getValue("title")
            title.align shouldBe Align.CENTER
            title.font shouldBe "window:y0"
            title.color shouldBe "#404040"
            title.shadow shouldBe false
            title.text.shouldBeNull()

            val label = shop.slots.getValue("buy_label")
            label.text.shouldNotBeNull()
            label.text shouldBe "Buy"

            val buy = shop.buttons.getValue("buy")
            buy.slots.map { it.area } shouldBe
                listOf(SlotAreaEntry.CONTAINER, SlotAreaEntry.CONTAINER, SlotAreaEntry.CONTAINER)
            buy.slots.map { it.index } shouldBe listOf(46, 47, 48)
            buy.default.shouldBeNull()

            shop.buttons.getValue("exit").default shouldBe ButtonDefault.CLOSE
        }

        "version 2 is rejected" {
            val json = TestManifests.sampleJson.replaceFirst("\"version\": 6", "\"version\": 2")
            val ex = shouldThrow<IllegalArgumentException> { WindowManifest.parse(json) }
            ex.message shouldBe
                "Unsupported Window definition version 2; this runtime only supports version 6"
        }

        "unknown keys are ignored" {
            val json =
                TestManifests.sampleJson.replaceFirst(
                    "\"namespace\": \"window\",",
                    "\"namespace\": \"window\", \"future_field\": 42,",
                )
            val m = WindowManifest.parse(json)
            m.namespace shouldBe "window"
        }
    })
