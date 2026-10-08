package com.chunkzero.window

import com.chunkzero.window.manifest.CollectionEntry
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer

class WindowListTest :
    StringSpec({
        class Lists : TestView(TestManifests.manifest(), FakeHost()) {
            var data: List<Int> = (0 until 13).toList()

            fun of(
                cells: Int,
                step: Int = cells,
                select: WindowList.Select = WindowList.Select.FIRST,
            ) = list(cells, { it: Int -> it }, step, select) { data }

            override fun WindowScope<Any>.bind() {}
        }

        "page mode moves by whole pages and keeps the final partial page" {
            val list = Lists().of(cells = 6)

            list.pageCount shouldBe 3
            list.canPrevious() shouldBe false
            list.next()
            list.next()
            list.page shouldBe 3
            list.visible shouldBe 12..12
            list.at(0) shouldBe 12
            list.at(1) shouldBe null
            list.canNext() shouldBe false
            list.previous()
            list.visible shouldBe 6..11
            list.last()
            list.page shouldBe 3
            list.first()
            list.page shouldBe 1
            shouldThrow<IllegalArgumentException> { list.at(6) }
            shouldThrow<IllegalArgumentException> { list.isSelected(-1) }
        }

        "row scrolling clamps to a full viewport, and the offset clamps when the source shrinks" {
            val lists = Lists()
            val list = lists.of(cells = 6, step = 3)

            list.next()
            list.at(0) shouldBe 3
            list.next()
            list.next()
            list.visible shouldBe 7..12
            list.canNext() shouldBe false

            lists.data = (0 until 8).toList()
            list.visible shouldBe 2..7
            lists.data = (0 until 4).toList()
            list.visible shouldBe 0..3
            list.canPrevious() shouldBe false
        }

        "a partial final step is the last page, and pages increase with every step" {
            val list = Lists().of(cells = 6, step = 3)
            val pages = mutableListOf<Int>()
            while (true) {
                pages += list.page
                if (!list.canNext()) break
                list.next()
            }

            pages shouldBe listOf(1, 2, 3, 4)
            list.pageCount shouldBe 4
        }

        "a step larger than the cells is rejected" {
            shouldThrow<IllegalArgumentException> { Lists().of(cells = 3, step = 4) }
        }

        "selection follows its key across paging and re-sorting, and select(item) pages to it" {
            val lists = Lists()
            val list = lists.of(cells = 6)

            list.selected shouldBe 0
            list.next()
            list.selectAt(2) shouldBe 8
            list.selectedCell shouldBe 2
            list.previous()
            list.selectedCell shouldBe null
            list.selected shouldBe 8

            lists.data = lists.data.reversed()
            list.selected shouldBe 8
            list.select(8)
            list.visible shouldBe 0..5
            list.isSelected(4) shouldBe true

            list.select(0)
            list.page shouldBe 3
            list.selectedCell shouldBe 0

            list.reset()
            list.page shouldBe 1
            list.selected shouldBe 12
        }

        "a missing selection falls back to the first item under FIRST and to nothing under NONE" {
            val lists = Lists()
            val first = lists.of(cells = 6)
            val none = lists.of(cells = 6, select = WindowList.Select.NONE)

            none.selected shouldBe null
            first.selectAt(3)
            none.selectAt(3)
            none.selected shouldBe 3

            lists.data = lists.data - 3
            first.selected shouldBe 0
            none.selected shouldBe null
            none.selectedCell shouldBe null
        }

        "a bound list computes its source once per change and routes clicks to selection" {
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    collections =
                        mapOf(
                            "pets" to
                                CollectionEntry(
                                    slots = (0 until 3).map(TestManifests::containerSlot),
                                    action = true,
                                    selection =
                                        (0 until 3).map {
                                            TestManifests.spriteSlot(
                                                x = 7 + it * 18,
                                                y = 17,
                                                width = 18,
                                                height = 18,
                                                sprite = "box",
                                            )
                                        },
                                ),
                        ),
                    sprites = mapOf("box" to TestManifests.sprite(width = 18, height = 18, glyph = "")),
                )
            var computed = 0
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    var count by state(5)
                    val pets =
                        list(cells = 3, key = { it: String -> it }, select = WindowList.Select.NONE) {
                            computed++
                            (0 until count).map { "pet$it" }
                        }

                    override fun WindowScope<Any>.bind() {
                        collection("pets", pets.items({ it }))
                    }
                }
            val handle = host.container
            view.open()
            val boxes = { PlainTextComponentSerializer.plainText().serialize(handle.titles.last()).count { it == '' } }

            computed shouldBe 1
            handle.items.getValue(SlotRef(SlotArea.CONTAINER, 2)) shouldBe "pet2"
            boxes() shouldBe 0

            handle.clickContainer(1)
            view.pets.next()
            host.scheduler.runAll()
            computed shouldBe 1
            handle.items.getValue(SlotRef(SlotArea.CONTAINER, 0)) shouldBe "pet3"
            handle.items[SlotRef(SlotArea.CONTAINER, 2)] shouldBe null
            boxes() shouldBe 0

            view.count = 2
            host.scheduler.runAll()
            computed shouldBe 2
            handle.items.getValue(SlotRef(SlotArea.CONTAINER, 1)) shouldBe "pet1"
            boxes() shouldBe 1
            view.pets.selected shouldBe "pet1"
        }
    })
