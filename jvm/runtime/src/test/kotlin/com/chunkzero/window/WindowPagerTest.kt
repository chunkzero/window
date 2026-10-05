package com.chunkzero.window

import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe

class WindowPagerTest :
    StringSpec({
        "page mode keeps the final partial page as an absolute page" {
            val pager = WindowPager(cellCount = 6)

            pager.pageCount(total = 13) shouldBe 3
            pager.page(offset = 0, total = 13) shouldBe 1
            pager.page(offset = 6, total = 13) shouldBe 2
            pager.page(offset = 12, total = 13) shouldBe 3

            pager.next(offset = 0, total = 13) shouldBe 6
            pager.next(offset = 6, total = 13) shouldBe 12
            pager.previous(offset = 12, total = 13) shouldBe 6
            pager.canNext(offset = 12, total = 13) shouldBe false

            pager.itemIndex(offset = 12, cell = 0, total = 13) shouldBe 12
            pager.itemIndex(offset = 12, cell = 1, total = 13) shouldBe null
        }

        "scroll mode still clamps to a full viewport when possible" {
            val pager = WindowPager(cellCount = 15, step = 3)

            pager.next(offset = 0, total = 16) shouldBe 1
            pager.previous(offset = 1, total = 16) shouldBe 0
            pager.canNext(offset = 1, total = 16) shouldBe false
        }
    })
