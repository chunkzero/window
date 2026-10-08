package com.chunkzero.window

import com.chunkzero.window.internal.Reactivity
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe

class ReactivityMemoTest :
    StringSpec({
        "a memo stops depending on states its latest computation did not read" {
            val reactivity = Reactivity(ManualScheduler()) {}
            var coins by reactivity.state(0)
            var filtered by reactivity.state(true)
            var computations = 0
            val memo =
                reactivity.memo {
                    computations++
                    if (filtered) coins else -1
                }

            memo.get() shouldBe 0
            filtered = false
            memo.get() shouldBe -1
            val before = computations

            coins = 5
            memo.get() shouldBe -1
            computations shouldBe before
        }
    })
