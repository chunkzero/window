package dev.oglass.window.codegen

import kotlin.system.exitProcess

/** Process entry point. All logic lives in [run] so it can be driven from tests. */
fun main(args: Array<String>) {
    exitProcess(run(args))
}
