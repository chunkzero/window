package dev.oglass.window.codegen

import io.kotest.core.spec.style.StringSpec
import io.kotest.engine.spec.tempdir
import io.kotest.matchers.shouldBe
import io.kotest.matchers.string.shouldContain
import java.io.ByteArrayOutputStream
import java.io.PrintStream
import java.nio.file.Files
import java.nio.file.Path
import kotlin.io.path.createDirectories
import kotlin.io.path.exists
import kotlin.io.path.readText
import kotlin.io.path.writeText

class CliTest :
    StringSpec({
        val sampleManifest =
            """
            {
              "version": 5,
              "windows": {
                "shop": {
                  "slots": { "title": { "x": 8, "y": 6 } },
                  "buttons": { "exit": { "default": "close" } }
                }
              }
            }
            """.trimIndent()

        fun writeManifest(
            dir: Path,
            text: String = sampleManifest,
        ): Path {
            val p = dir.resolve("window-definition.json")
            p.writeText(text)
            return p
        }

        data class RunResult(
            val code: Int,
            val out: String,
            val err: String,
        )

        fun invoke(vararg args: String): RunResult {
            val o = ByteArrayOutputStream()
            val e = ByteArrayOutputStream()
            val code = run(arrayOf(*args), PrintStream(o), PrintStream(e))
            return RunResult(code, o.toString(), e.toString())
        }

        "generates files and reports summary" {
            val dir = tempdir().toPath()
            val manifest = writeManifest(dir)
            val out = dir.resolve("gen")

            val r =
                invoke(
                    "--manifest",
                    manifest.toString(),
                    "--package",
                    "com.example.ui",
                    "--out",
                    out.toString(),
                )

            r.code shouldBe 0
            r.out shouldContain "1 generated, 0 deleted"
            out.resolve("com/example/ui/ShopView.kt").exists() shouldBe true
        }

        "check passes on freshly generated output" {
            val dir = tempdir().toPath()
            val manifest = writeManifest(dir)
            val out = dir.resolve("gen")

            invoke(
                "--manifest",
                manifest.toString(),
                "--package",
                "com.example.ui",
                "--out",
                out.toString(),
            ).code shouldBe 0

            val r =
                invoke(
                    "--manifest",
                    manifest.toString(),
                    "--package",
                    "com.example.ui",
                    "--out",
                    out.toString(),
                    "--check",
                )
            r.code shouldBe 0
        }

        "check flags a tampered file" {
            val dir = tempdir().toPath()
            val manifest = writeManifest(dir)
            val out = dir.resolve("gen")
            invoke(
                "--manifest",
                manifest.toString(),
                "--package",
                "com.example.ui",
                "--out",
                out.toString(),
            )

            val file = out.resolve("com/example/ui/ShopView.kt")
            file.writeText(file.readText() + "// tamper\n")

            val r =
                invoke(
                    "--manifest",
                    manifest.toString(),
                    "--package",
                    "com.example.ui",
                    "--out",
                    out.toString(),
                    "--check",
                )
            r.code shouldBe 1
            r.err shouldContain "out of date"
        }

        "check flags a missing file" {
            val dir = tempdir().toPath()
            val manifest = writeManifest(dir)
            val out = dir.resolve("gen")
            // never generate
            val r =
                invoke(
                    "--manifest",
                    manifest.toString(),
                    "--package",
                    "com.example.ui",
                    "--out",
                    out.toString(),
                    "--check",
                )
            r.code shouldBe 1
            r.err shouldContain "missing"
        }

        "check flags a stale generated file; normal run deletes it" {
            val dir = tempdir().toPath()
            val manifest = writeManifest(dir)
            val out = dir.resolve("gen")
            invoke(
                "--manifest",
                manifest.toString(),
                "--package",
                "com.example.ui",
                "--out",
                out.toString(),
            )

            // Plant a stale generated file (carries the header) for a window no longer present.
            val pkgDir = out.resolve("com/example/ui")
            val stale = pkgDir.resolve("OldView.kt")
            stale.writeText(Generator.HEADER + "\npackage com.example.ui\n")

            val checkResult =
                invoke(
                    "--manifest",
                    manifest.toString(),
                    "--package",
                    "com.example.ui",
                    "--out",
                    out.toString(),
                    "--check",
                )
            checkResult.code shouldBe 1
            checkResult.err shouldContain "stale"

            val r =
                invoke(
                    "--manifest",
                    manifest.toString(),
                    "--package",
                    "com.example.ui",
                    "--out",
                    out.toString(),
                )
            r.code shouldBe 0
            r.out shouldContain "1 deleted"
            stale.exists() shouldBe false
        }

        "non-generated files in out dir are left alone" {
            val dir = tempdir().toPath()
            val manifest = writeManifest(dir)
            val out = dir.resolve("gen")
            val pkgDir = out.resolve("com/example/ui")
            pkgDir.createDirectories()
            val handwritten = pkgDir.resolve("MyShop.kt")
            handwritten.writeText("package com.example.ui\n\nclass MyShop\n")

            invoke(
                "--manifest",
                manifest.toString(),
                "--package",
                "com.example.ui",
                "--out",
                out.toString(),
            ).code shouldBe 0
            handwritten.exists() shouldBe true
        }

        "missing required flag yields usage and exit 2" {
            val r = invoke("--package", "x", "--out", "y")
            r.code shouldBe 2
            r.err shouldContain "usage:"
            r.err shouldContain "--manifest"
        }

        "unknown flag yields exit 2" {
            val r = invoke("--manifest", "m", "--package", "p", "--out", "o", "--bogus")
            r.code shouldBe 2
            r.err shouldContain "unknown"
        }

        "duplicate flag yields exit 2" {
            val r = invoke("--manifest", "m", "--manifest", "n", "--package", "p", "--out", "o")
            r.code shouldBe 2
            r.err shouldContain "duplicate"
        }

        "flag without value yields exit 2" {
            val r = invoke("--manifest")
            r.code shouldBe 2
            r.err shouldContain "requires a value"
        }

        "unreadable manifest yields exit 1" {
            val dir = tempdir().toPath()
            val missing = dir.resolve("nope.json")
            val r =
                invoke(
                    "--manifest",
                    missing.toString(),
                    "--package",
                    "p",
                    "--out",
                    dir.resolve("out").toString(),
                )
            r.code shouldBe 1
            r.err shouldContain "cannot read manifest"
        }

        "generation error (reserved member) yields exit 1" {
            val dir = tempdir().toPath()
            val manifest =
                writeManifest(
                    dir,
                    """
                    { "version": 5, "windows": { "shop": { "slots": { "player": {} }, "buttons": {} } } }
                    """.trimIndent(),
                )
            val r =
                invoke(
                    "--manifest",
                    manifest.toString(),
                    "--package",
                    "p",
                    "--out",
                    dir.resolve("out").toString(),
                )
            r.code shouldBe 1
            r.err shouldContain "reserved"
        }

        "ignores unknown manifest fields" {
            val dir = tempdir().toPath()
            val manifest =
                writeManifest(
                    dir,
                    """
                    {
                      "version": 5,
                      "namespace": "window",
                      "font": "window:ui",
                      "spacers": { "983040": -1024 },
                      "text_advances": { " ": 4 },
                      "windows": {
                        "shop": {
                          "surface": { "kind": "container", "container": "generic_9x6" },
                          "static": "x",
                          "slots": { "title": { "x": 8, "y": 6, "align": "center" } },
                          "buttons": {
                            "buy": {
                              "x": 1,
                              "slots": [{ "area": "container", "index": 46 }],
                              "default": null
                            }
                          }
                        }
                      }
                    }
                    """.trimIndent(),
                )
            val out = dir.resolve("gen")
            val r =
                invoke(
                    "--manifest",
                    manifest.toString(),
                    "--package",
                    "com.example.ui",
                    "--out",
                    out.toString(),
                )
            r.code shouldBe 0
            val content = Files.readString(out.resolve("com/example/ui/ShopView.kt"))
            content shouldContain "fun title(): Component"
            content shouldContain "protected abstract fun onBuy(click: Click)"
        }
    })
