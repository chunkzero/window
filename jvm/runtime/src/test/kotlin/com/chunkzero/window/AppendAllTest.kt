package com.chunkzero.window

import com.chunkzero.window.internal.appendAll
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.types.shouldBeSameInstanceAs
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor

class AppendAllTest :
    StringSpec({
        "appendAll matches repeated append, including skipped empty components" {
            val base = Component.text("base").color(NamedTextColor.RED).append(Component.text("existing"))
            val parts =
                listOf(
                    Component.text("a"),
                    Component.empty(),
                    Component.text("b").color(NamedTextColor.BLUE).append(Component.text("c")),
                )

            base.appendAll(parts) shouldBe parts.fold(base) { joined, part -> joined.append(part) }
        }

        "appendAll without parts returns the component itself" {
            val base = Component.text("base")

            base.appendAll(emptyList()).shouldBeSameInstanceAs(base)
            base.appendAll(listOf(Component.empty())).shouldBeSameInstanceAs(base)
        }
    })
