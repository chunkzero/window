package com.chunkzero.window

import com.chunkzero.window.internal.CanvasPoint
import com.chunkzero.window.internal.FullscreenMapping
import com.chunkzero.window.internal.Turn
import com.chunkzero.window.manifest.ControlEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe

class FullscreenMappingTest :
    StringSpec({
        val canvas = FullscreenManifests.canvas
        val mapping = FullscreenMapping(canvas)

        "the cursor starts at the canvas centre and reaches the edges at the edge angles" {
            mapping.cursor(0f, 0f) shouldBe CanvasPoint(240, 135)
            mapping.cursor(45f, 25.3125f) shouldBe CanvasPoint(479, 269)
            mapping.cursor(-45f, -25.3125f) shouldBe CanvasPoint(0, 0)
            mapping.cursor(60f, -40f) shouldBe CanvasPoint(479, 0)
        }

        "a rotation just short of a pixel boundary snaps onto it, and yaw wraps" {
            mapping.cursor(0.1875f - 0.001f, 0f) shouldBe CanvasPoint(241, 135)
            mapping.cursor(360.1875f, 0f) shouldBe CanvasPoint(241, 135)
            mapping.cursor(179f, 0f) shouldBe CanvasPoint(479, 135)
            mapping.cursor(-179f, 0f) shouldBe CanvasPoint(0, 135)
        }

        "turning more than a degree past an edge pulls back to it, past the seam to the nearer edge" {
            mapping.pullBack(45.9f, 26.3f) shouldBe null
            mapping.pullBack(47f, 0f) shouldBe Turn(-2f, 0f)
            mapping.pullBack(0f, -27.3125f) shouldBe Turn(0f, 2f)
            mapping.pullBack(181f, 0f) shouldBe Turn(134f, 0f)
        }

        "a hit grows the control by the margin" {
            val control = ControlEntry(index = 0, x = 10, y = 20, width = 4, height = 2, hover = true)
            mapping.hits(control, CanvasPoint(13, 21)) shouldBe true
            mapping.hits(control, CanvasPoint(14, 21)) shouldBe false
            mapping.hits(control, CanvasPoint(14, 22), canvas.hitMargin) shouldBe true
            mapping.hits(control, CanvasPoint(8, 20), canvas.hitMargin) shouldBe false
        }
    })
