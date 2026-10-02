package dev.oglass.window.validationserver

import java.nio.file.Path

internal data class Config(
    val port: Int,
    val manifest: Path,
    val descriptor: Path,
    val report: Path,
) {
    companion object {
        fun parse(args: Array<String>): Config {
            val values =
                args.asList().chunked(2).associate { pair ->
                    require(pair.size == 2 && pair[0].startsWith("--")) {
                        "expected --port, --manifest, and --report arguments"
                    }
                    pair[0].removePrefix("--") to pair[1]
                }
            return Config(
                values.getValue("port").toInt(),
                Path.of(values.getValue("manifest")),
                Path.of(values.getValue("descriptor")),
                Path.of(values.getValue("report")),
            )
        }
    }
}
