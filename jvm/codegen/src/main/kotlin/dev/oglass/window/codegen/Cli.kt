package dev.oglass.window.codegen

import dev.oglass.window.codegen.manifest.parseManifest
import java.io.PrintStream
import java.nio.file.Files
import java.nio.file.Path
import kotlin.io.path.createDirectories
import kotlin.io.path.deleteExisting
import kotlin.io.path.exists
import kotlin.io.path.isRegularFile
import kotlin.io.path.listDirectoryEntries
import kotlin.io.path.name
import kotlin.io.path.readText
import kotlin.io.path.writeText

/** Usage string shown on argument errors and as documentation. */
internal const val USAGE: String =
    "usage: window-codegen --manifest <file> --package <pkg> --out <dir> [--check]"

private const val EXIT_OK = 0
private const val EXIT_DRIFT = 1
private const val EXIT_GEN_ERROR = 1
private const val EXIT_USAGE = 2

/**
 * Testable CLI entry point. Returns the process exit code; never calls `exit`. [out] and [err] are
 * injectable so tests can capture output.
 */
internal fun run(
    args: Array<String>,
    out: PrintStream = System.out,
    err: PrintStream = System.err,
): Int {
    val parsed =
        when (val result = parseArgs(args)) {
            is ArgParse.Error -> {
                err.println(result.message)
                err.println(USAGE)
                return EXIT_USAGE
            }

            is ArgParse.Ok -> {
                result.args
            }
        }

    val manifestText =
        try {
            Path.of(parsed.manifest).readText()
        } catch (e: Exception) {
            err.println("error: cannot read manifest `${parsed.manifest}`: ${e.message}")
            return EXIT_GEN_ERROR
        }

    val files =
        try {
            val manifest = parseManifest(manifestText)
            Generator.generate(manifest, parsed.packageName)
        } catch (e: GenerationException) {
            err.println("error: ${e.message}")
            return EXIT_GEN_ERROR
        } catch (e: Exception) {
            err.println("error: invalid manifest: ${e.message}")
            return EXIT_GEN_ERROR
        }

    val outDir = Path.of(parsed.out).resolve(parsed.packageName.replace('.', '/'))

    return if (parsed.check) {
        check(files, outDir, err)
    } else {
        write(files, outDir, out)
    }
}

/** Generate files to disk, deleting stale previously-generated files. */
private fun write(
    files: List<GeneratedFile>,
    outDir: Path,
    out: PrintStream,
): Int {
    outDir.createDirectories()

    for (file in files) {
        outDir.resolve(file.fileName).writeText(file.content)
    }

    val expected = files.map { it.fileName }.toSet()
    var deleted = 0
    for (stale in staleGeneratedFiles(outDir, expected)) {
        stale.deleteExisting()
        deleted++
    }

    out.println("window-codegen: ${files.size} generated, $deleted deleted")
    return EXIT_OK
}

/** Compare in-memory generation against disk, byte-exact. */
private fun check(
    files: List<GeneratedFile>,
    outDir: Path,
    err: PrintStream,
): Int {
    val offenders = mutableListOf<String>()
    val expected = files.map { it.fileName }.toSet()

    for (file in files) {
        val path = outDir.resolve(file.fileName)
        when {
            !path.exists() -> offenders += "missing: $path"
            path.readText() != file.content -> offenders += "out of date: $path"
        }
    }

    for (stale in staleGeneratedFiles(outDir, expected)) {
        offenders += "stale: $stale"
    }

    return if (offenders.isEmpty()) {
        EXIT_OK
    } else {
        err.println("window-codegen --check found drift:")
        offenders.sorted().forEach { err.println("  $it") }
        EXIT_DRIFT
    }
}

/**
 * Previously-generated files in [dir] (identified by the header line) whose window no longer
 * exists. Returns an empty list if the directory does not exist.
 */
private fun staleGeneratedFiles(
    dir: Path,
    expected: Set<String>,
): List<Path> {
    if (!dir.exists()) return emptyList()
    return dir
        .listDirectoryEntries("*.kt")
        .filter { it.isRegularFile() && it.name !in expected }
        .filter { isGeneratedFile(it) }
        .sorted()
}

/** True if [path] begins with the generated-file header. */
private fun isGeneratedFile(path: Path): Boolean =
    try {
        Files.newBufferedReader(path).use { it.readLine() == Generator.HEADER }
    } catch (_: Exception) {
        false
    }

private data class Args(
    val manifest: String,
    val packageName: String,
    val out: String,
    val check: Boolean,
)

private sealed interface ArgParse {
    data class Ok(
        val args: Args,
    ) : ArgParse

    data class Error(
        val message: String,
    ) : ArgParse
}

/** Hand-rolled flag parsing: no dependencies, strict about missing/duplicate/unknown flags. */
private fun parseArgs(args: Array<String>): ArgParse {
    var manifest: String? = null
    var packageName: String? = null
    var out: String? = null
    var check = false

    var i = 0
    while (i < args.size) {
        val flag = args[i]
        when (flag) {
            "--check" -> {
                if (check) return ArgParse.Error("error: duplicate flag --check")
                check = true
                i += 1
            }

            "--manifest",
            "--package",
            "--out",
            -> {
                if (i + 1 >= args.size) return ArgParse.Error("error: $flag requires a value")
                val value = args[i + 1]
                when (flag) {
                    "--manifest" -> {
                        if (manifest != null) {
                            return ArgParse.Error("error: duplicate flag --manifest")
                        }
                        manifest = value
                    }

                    "--package" -> {
                        if (packageName != null) {
                            return ArgParse.Error("error: duplicate flag --package")
                        }
                        packageName = value
                    }

                    "--out" -> {
                        if (out != null) return ArgParse.Error("error: duplicate flag --out")
                        out = value
                    }
                }
                i += 2
            }

            else -> {
                return ArgParse.Error("error: unknown argument `$flag`")
            }
        }
    }

    if (manifest == null) return ArgParse.Error("error: missing required flag --manifest")
    if (packageName == null) return ArgParse.Error("error: missing required flag --package")
    if (out == null) return ArgParse.Error("error: missing required flag --out")

    return ArgParse.Ok(Args(manifest, packageName, out, check))
}
