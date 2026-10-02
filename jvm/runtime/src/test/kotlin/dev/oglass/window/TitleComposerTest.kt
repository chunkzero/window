package dev.oglass.window

import dev.oglass.window.internal.FontRegistry
import dev.oglass.window.internal.Spacers
import dev.oglass.window.internal.TitleComposer
import dev.oglass.window.manifest.Align
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.nulls.shouldNotBeNull
import io.kotest.matchers.shouldBe
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.TextComponent
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.ShadowColor
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.format.TextDecoration

class TitleComposerTest :
    StringSpec({
        val table = TestManifests.spacerTable()
        val spacers = Spacers(table)
        val fonts = FontRegistry.fromManifest(TestManifests.manifest())
        val hiAdvance = fonts.measure(Component.text("Hi"), "minecraft:default").advance

        fun decode(s: String): Int {
            var total = 0
            var i = 0
            while (i < s.length) {
                val cp = s.codePointAt(i)
                total += table.getValue(cp)
                i += Character.charCount(cp)
            }
            return total
        }

        "static component carries the main font and verbatim string" {
            val manifest =
                TestManifests.manifest(
                    static = "RAW_CHROME",
                    slots = mapOf("title" to TestManifests.slot(8, 160, Align.CENTER)),
                )
            val window = manifest.windows.getValue("w")
            val composer = TitleComposer(manifest, window)
            val sc = composer.staticComponent as TextComponent
            sc.content() shouldBe "RAW_CHROME"
            sc.style().font() shouldBe Key.key("window:ui")
            sc.style().color() shouldBe NamedTextColor.WHITE
            sc.style().shadowColor() shouldBe ShadowColor.none()
            sc.style().decoration(TextDecoration.ITALIC) shouldBe TextDecoration.State.FALSE
            sc.style().decoration(TextDecoration.BOLD) shouldBe TextDecoration.State.FALSE
        }

        "left-aligned slot segment is net-zero with correct font, color, and spacers" {
            // title origin x = 8. Slot at x=8, left aligned. Text "Hi" measures 8 (H=6,i=2).
            val manifest =
                TestManifests.manifest(
                    titleOrigin = listOf(8, 6),
                    slots =
                        mapOf(
                            "title" to
                                TestManifests.slot(
                                    x = 8,
                                    width = 160,
                                    align = Align.LEFT,
                                    font = "window:y0",
                                    color = "#ff0000",
                                ),
                        ),
                )
            val window = manifest.windows.getValue("w")
            val composer = TitleComposer(manifest, window)

            val segment = composer.renderSlot(window.slots.getValue("title"), Component.text("Hi"))
            segment.shouldNotBeNull()

            // Expected: dx = xStart(8) - origin(8) = 0 -> no lead spacer.
            // trail = -(0 + 8) = -8.
            val children = segment.children()
            // First child = styled text "Hi"; net-zero requires a trailing spacer.
            val styled = children[0] as TextComponent
            styled.content() shouldBe "Hi"
            styled.style().font() shouldBe Key.key("window:y0")
            (styled.color() as TextColor) shouldBe TextColor.fromHexString("#ff0000")
            styled.style().shadowColor() shouldBe ShadowColor.none()
            styled.style().decoration(TextDecoration.ITALIC) shouldBe TextDecoration.State.FALSE
            styled.style().decoration(TextDecoration.BOLD) shouldBe TextDecoration.State.FALSE

            val trail = children[1] as TextComponent
            trail.style().font() shouldBe Key.key("window:ui")
            trail.style().color() shouldBe NamedTextColor.WHITE
            decode(trail.content()) shouldBe -8

            // Net advance of the whole segment = lead(0) + measured(8) + trail(-8) = 0.
            (0 + hiAdvance + decode(trail.content())) shouldBe 0
        }

        "shadowed slot uses an explicit text shadow color" {
            val manifest =
                TestManifests.manifest(
                    slots =
                        mapOf(
                            "title" to
                                TestManifests.slot(
                                    x = 8,
                                    width = 160,
                                    align = Align.LEFT,
                                    shadow = true,
                                ),
                        ),
                )
            val window = manifest.windows.getValue("w")
            val composer = TitleComposer(manifest, window)

            val segment =
                composer.renderSlot(window.slots.getValue("title"), Component.text("Hi"))!!
            val styled = segment.children()[0] as TextComponent

            styled.style().shadowColor() shouldBe ShadowColor.shadowColor(0, 0, 0, 180)
            val trail = segment.children()[1] as TextComponent
            trail.style().shadowColor() shouldBe ShadowColor.none()
        }

        "center-aligned slot emits a leading and trailing spacer summing net-zero" {
            // origin x = 8. Slot x=8 width=20, center. "Hi" has visible width 7 and
            // cursor advance 8. xStart = 8 + (20 - 7)/2 = 14. dx = 6.
            // trail = -(6 + 8) = -14.
            val manifest =
                TestManifests.manifest(
                    titleOrigin = listOf(8, 6),
                    slots =
                        mapOf(
                            "title" to TestManifests.slot(x = 8, width = 20, align = Align.CENTER),
                        ),
                )
            val window = manifest.windows.getValue("w")
            val composer = TitleComposer(manifest, window)
            val segment =
                composer.renderSlot(window.slots.getValue("title"), Component.text("Hi"))!!
            val children = segment.children()

            val lead = children[0] as TextComponent
            decode(lead.content()) shouldBe 6
            val styled = children[1] as TextComponent
            styled.content() shouldBe "Hi"
            val trail = children[2] as TextComponent
            decode(trail.content()) shouldBe -14

            (decode(lead.content()) + hiAdvance + decode(trail.content())) shouldBe 0
        }

        "right-aligned slot aligns by visible width but resets by advance width" {
            val manifest =
                TestManifests.manifest(
                    titleOrigin = listOf(8, 6),
                    slots =
                        mapOf("title" to TestManifests.slot(x = 8, width = 20, align = Align.RIGHT)),
                )
            val window = manifest.windows.getValue("w")
            val composer = TitleComposer(manifest, window)
            val segment =
                composer.renderSlot(window.slots.getValue("title"), Component.text("Hi"))!!
            val children = segment.children()

            val lead = children[0] as TextComponent
            decode(lead.content()) shouldBe 13
            val styled = children[1] as TextComponent
            styled.content() shouldBe "Hi"
            val trail = children[2] as TextComponent
            decode(trail.content()) shouldBe -21

            (decode(lead.content()) + hiAdvance + decode(trail.content())) shouldBe 0
        }

        "empty rendered content yields no segment" {
            val manifest =
                TestManifests.manifest(
                    slots = mapOf("title" to TestManifests.slot(8, 160, Align.LEFT)),
                )
            val window = manifest.windows.getValue("w")
            val composer = TitleComposer(manifest, window)
            composer.renderSlot(window.slots.getValue("title"), Component.empty()) shouldBe null
        }

        "runtime sprite segment uses the sprite font and resets by glyph advance" {
            val manifest =
                TestManifests.manifest(
                    titleOrigin = listOf(8, 6),
                    sprites =
                        mapOf(
                            "pickaxe" to
                                TestManifests.sprite(width = 16, advance = 14, glyph = "\uE123"),
                        ),
                    spriteSlots =
                        mapOf(
                            "icon" to
                                TestManifests.spriteSlot(
                                    x = 8,
                                    y = 24,
                                    width = 20,
                                    align = Align.CENTER,
                                    font = "window:sprite_y18",
                                ),
                        ),
                )
            val window = manifest.windows.getValue("w")
            val composer = TitleComposer(manifest, window)
            val segment = composer.renderSprite(window.spriteSlots.getValue("icon"), "pickaxe")!!
            val children = segment.children()

            val lead = children[0] as TextComponent
            decode(lead.content()) shouldBe 2
            val glyph = children[1] as TextComponent
            glyph.content() shouldBe "\uE123"
            glyph.style().font() shouldBe Key.key("window:sprite_y18")
            val trail = children[2] as TextComponent
            decode(trail.content()) shouldBe -16

            (decode(lead.content()) + 14 + decode(trail.content())) shouldBe 0
        }

        "runtime sprite segment aligns by visible ink bounds" {
            val manifest =
                TestManifests.manifest(
                    titleOrigin = listOf(8, 6),
                    sprites =
                        mapOf(
                            "token" to
                                TestManifests.sprite(
                                    width = 12,
                                    xOffset = 1,
                                    glyphWidth = 11,
                                    advance = 13,
                                    glyph = "\uE124",
                                ),
                        ),
                    spriteSlots =
                        mapOf(
                            "currency" to
                                TestManifests.spriteSlot(
                                    x = 8,
                                    y = 24,
                                    width = 12,
                                    height = 12,
                                    align = Align.CENTER,
                                    font = "window:sprite_y18",
                                ),
                        ),
                )
            val window = manifest.windows.getValue("w")
            val composer = TitleComposer(manifest, window)
            val segment = composer.renderSprite(window.spriteSlots.getValue("currency"), "token")!!
            val children = segment.children()

            val lead = children[0] as TextComponent
            decode(lead.content()) shouldBe -1
            val trail = children[2] as TextComponent
            decode(trail.content()) shouldBe -12

            (decode(lead.content()) + 13 + decode(trail.content())) shouldBe 0
        }

        "width is measured from effective component styling" {
            // A colored component keeps its color while slot font fallback still measures "Hi" as
            // 8.
            val manifest =
                TestManifests.manifest(
                    slots = mapOf("title" to TestManifests.slot(8, 160, Align.LEFT)),
                )
            val window = manifest.windows.getValue("w")
            val composer = TitleComposer(manifest, window)
            val content = Component.text("Hi").color(NamedTextColor.GREEN)
            val segment = composer.renderSlot(window.slots.getValue("title"), content)!!
            val styled = segment.children()[0] as TextComponent
            styled.style().color() shouldBe NamedTextColor.GREEN
            val trail = segment.children().last() as TextComponent
            decode(trail.content()) shouldBe -8
        }

        "bold text uses bold metrics for net-zero spacer math" {
            val manifest =
                TestManifests.manifest(
                    slots = mapOf("title" to TestManifests.slot(8, 160, Align.LEFT)),
                )
            val window = manifest.windows.getValue("w")
            val composer = TitleComposer(manifest, window)
            val content = Component.text("Hi").decorate(TextDecoration.BOLD)
            val segment = composer.renderSlot(window.slots.getValue("title"), content)!!
            val trail = segment.children().last() as TextComponent
            decode(trail.content()) shouldBe -10
        }

        "child font metrics influence slot alignment" {
            val manifest =
                TestManifests.manifest(
                    slots = mapOf("title" to TestManifests.slot(8, 160, Align.LEFT)),
                    fontMetrics =
                        TestManifests.fontMetricEntries(
                            mapOf("window:wide" to TestManifests.wideMetrics()),
                        ),
                )
            val window = manifest.windows.getValue("w")
            val composer = TitleComposer(manifest, window)
            val content =
                Component.empty().append(Component.text("Hi").font(Key.key("window:wide")))
            val segment = composer.renderSlot(window.slots.getValue("title"), content)!!
            val trail = segment.children().last() as TextComponent
            decode(trail.content()) shouldBe -15
        }

        "compose appends already-rendered segments without wrapping them again" {
            val manifest =
                TestManifests.manifest(
                    slots = mapOf("title" to TestManifests.slot(8, 160, Align.LEFT)),
                )
            val window = manifest.windows.getValue("w")
            val composer = TitleComposer(manifest, window)
            val segment =
                composer.renderSlot(window.slots.getValue("title"), Component.text("Hi"))!!

            val title = composer.compose(linkedMapOf("title" to segment))
            title.children().size shouldBe 1
            title.children()[0] shouldBe segment
        }
    })
