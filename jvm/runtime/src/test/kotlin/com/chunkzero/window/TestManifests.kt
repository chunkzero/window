package com.chunkzero.window

import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.AnvilInputEntry
import com.chunkzero.window.manifest.CollectionEntry
import com.chunkzero.window.manifest.FontMetricsEntry
import com.chunkzero.window.manifest.HitboxEntry
import com.chunkzero.window.manifest.HudEntry
import com.chunkzero.window.manifest.HudShaderEntry
import com.chunkzero.window.manifest.HudSurfaceEntry
import com.chunkzero.window.manifest.ItemEntry
import com.chunkzero.window.manifest.LayerEntry
import com.chunkzero.window.manifest.LayerKind
import com.chunkzero.window.manifest.RegionEntry
import com.chunkzero.window.manifest.RepeatGroupEntry
import com.chunkzero.window.manifest.SlotAreaEntry
import com.chunkzero.window.manifest.SlotEntry
import com.chunkzero.window.manifest.SlotRefEntry
import com.chunkzero.window.manifest.SpriteEntry
import com.chunkzero.window.manifest.SpriteSlotEntry
import com.chunkzero.window.manifest.SurfaceEntry
import com.chunkzero.window.manifest.SwitchCaseEntry
import com.chunkzero.window.manifest.SwitchEntry
import com.chunkzero.window.manifest.TooltipEntry
import com.chunkzero.window.manifest.WindowEntry
import com.chunkzero.window.manifest.WindowManifest

/** Hand-written sample manifests and builders shared across tests. */
object TestManifests {
    /**
     * The canonical spacer table: codepoints starting at U+F0000 for ±1, ±2, ±4 … ±1024. The exact
     * codepoint assignment mirrors `docs/ARCHITECTURE.md` (negative powers ascending, then
     * positive). Magnitudes: 1,2,4,8,16,32,64,128,256,512,1024.
     */
    val spacerMagnitudes = listOf(1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024)

    /** Builds the spacer table as the manifest serialises it: codepoint (Int) -> advance. */
    fun spacerTable(): Map<Int, Int> {
        val map = LinkedHashMap<Int, Int>()
        var cp = 0xF0000
        // negative powers ascending (most negative first): -1024 .. -1
        for (m in spacerMagnitudes.reversed()) {
            map[cp++] = -m
        }
        // positive powers ascending: 1 .. 1024
        for (m in spacerMagnitudes) {
            map[cp++] = m
        }
        return map
    }

    /** Codepoint for a given signed advance in [spacerTable]. */
    fun spacerCodepoint(advance: Int): Int = spacerTable().entries.first { it.value == advance }.key

    /** A minimal advance table covering ASCII letters used in tests. */
    fun advances(): Map<String, Int> =
        buildMap {
            put(" ", 4)
            put("!", 2)
            put("i", 2)
            put("l", 3)
            for (c in 'a'..'z') if (c !in setOf('i', 'l')) put(c.toString(), 6)
            for (c in 'A'..'Z') put(c.toString(), 6)
            for (c in '0'..'9') put(c.toString(), 6)
        }

    /** A minimal visible glyph-width table covering ASCII letters used in tests. */
    fun glyphWidths(): Map<String, Int> =
        advances().mapValues { (char, advance) -> if (char == " ") 0 else advance - 1 }

    /** Canonical generated font metrics for tests. */
    fun fontMetricEntries(extra: Map<String, FontMetricsEntry> = emptyMap()): Map<String, FontMetricsEntry> {
        val entry =
            FontMetricsEntry(advances = advances(), glyphWidths = glyphWidths(), boldAdvance = 1)
        return buildMap {
            put("minecraft:default", entry)
            put("window:y0", entry)
            put("window:y2", entry)
            put("window:y6", entry)
            put("window:y190", entry)
            putAll(extra)
        }
    }

    fun wideMetrics(): FontMetricsEntry =
        FontMetricsEntry(
            advances = advances() + mapOf("H" to 10, "i" to 5),
            glyphWidths = glyphWidths() + mapOf("H" to 9, "i" to 4),
            boldAdvance = 1,
        )

    /** A full sample legacy JSON definition, written by hand per docs/MANIFEST.md. */
    val sampleJson: String =
        """
        {
          "version": 8,
          "namespace": "window",
          "font": "window:ui",
          "spacers": { "983040": -1024, "983061": 1024, "983050": -1, "983051": 1 },
          "text_advances": { " ": 4, "!": 2, "~": 7, "A": 6 },
          "text_glyph_widths": { " ": 0, "!": 1, "~": 6, "A": 5 },
          "font_metrics": {
            "minecraft:default": {
              "advances": { " ": 4, "!": 2, "~": 7, "A": 6 },
              "glyph_widths": { " ": 0, "!": 1, "~": 6, "A": 5 },
              "bold_advance": 1
            },
            "window:y0": {
              "advances": { " ": 4, "!": 2, "~": 7, "A": 6 },
              "glyph_widths": { " ": 0, "!": 1, "~": 6, "A": 5 },
              "bold_advance": 1
            }
          },
          "windows": {
            "shop": {
              "surface": {
                "kind": "container",
                "container": "generic_9x6",
                "size": [176, 222],
                "title_origin": [8, 6]
              },
              "static": "STATIC_CHROME",
              "slots": {
                "title": {
                  "x": 8, "y": 6, "width": 160, "align": "center",
                  "font": "window:y0", "color": "#404040", "shadow": false
                },
                "buy_label": {
                  "x": 58, "y": 196, "width": 60, "align": "center",
                  "font": "window:y190", "color": "#ffffff", "shadow": false,
                  "text": "Buy"
                }
              },
              "sprite_slots": {
                "icon": { "x": 8, "y": 20, "width": 16, "height": 16, "align": "left", "font": "window:sprite_y14" }
              },
              "regions": {
                "buy.enabled": {
                  "x": 52, "y": 190, "width": 72, "height": 20,
                  "slots": [
                    { "area": "container", "index": 46 },
                    { "area": "container", "index": 47 },
                    { "area": "player", "index": 0 }
                  ],
                  "fill_slots": [{ "area": "container", "index": 47 }],
                  "action": "buy",
                  "hitbox": { "item_model": "example:gui/buy", "tooltip": { "title": "Buy", "lines": ["Spend coins"] } },
                  "source": "button `buy`"
                },
                "exit": {
                  "x": 0, "y": 0, "width": 18, "height": 18,
                  "slots": [{ "area": "container", "index": 0 }],
                  "action": "exit",
                  "default_action": "window:close"
                }
              },
              "switches": {
                "buy": {
                  "states": true,
                  "initial": "enabled",
                  "source": "button `buy`",
                  "cases": [{ "value": "enabled", "static": "ART", "regions": ["buy.enabled"], "switches": ["inner"] }]
                },
                "inner": { "binding": "mode", "cases": [{ "value": "a", "sprite_slots": ["icon"] }] }
              },
              "layers": [
                { "kind": "slot", "name": "title" },
                { "kind": "switch", "name": "buy" },
                { "kind": "switch", "name": "inner" },
                { "kind": "sprite_slot", "name": "icon" }
              ],
              "groups": { "cell": { "count": 1, "actions": ["cell_0"] } }
            }
          }
        }
        """.trimIndent()

    /**
     * Builds a tiny in-memory manifest with a single window and the given entries, using the
     * canonical spacer/advance tables. Title origin defaults to (8, 6). [layers] defaults to
     * switches, selectable collections, sprite slots, then slots, each in map order.
     */
    fun manifest(
        windowName: String = "w",
        container: String = "generic_9x3",
        titleOrigin: List<Int> = listOf(8, 6),
        static: String = "STATIC",
        slots: Map<String, SlotEntry> = emptyMap(),
        spriteSlots: Map<String, SpriteSlotEntry> = emptyMap(),
        regions: Map<String, RegionEntry> = emptyMap(),
        items: Map<String, ItemEntry> = emptyMap(),
        collections: Map<String, CollectionEntry> = emptyMap(),
        inputs: Map<String, AnvilInputEntry> = emptyMap(),
        groups: Map<String, RepeatGroupEntry> = emptyMap(),
        huds: Map<String, HudEntry> = emptyMap(),
        sprites: Map<String, SpriteEntry> = emptyMap(),
        fontMetrics: Map<String, FontMetricsEntry> = fontMetricEntries(),
        switches: Map<String, SwitchEntry> = emptyMap(),
        layers: List<LayerEntry> =
            switches.keys.map { LayerEntry(LayerKind.SWITCH, it) } +
                collections.filterValues { it.selection.isNotEmpty() }.keys.map {
                    LayerEntry(LayerKind.COLLECTION, it)
                } +
                spriteSlots.keys.map { LayerEntry(LayerKind.SPRITE_SLOT, it) } +
                slots.keys.map { LayerEntry(LayerKind.SLOT, it) },
    ): WindowManifest =
        WindowManifest(
            version = 8,
            namespace = "window",
            font = "window:ui",
            spacers = spacerTable(),
            textAdvances = advances(),
            textGlyphWidths = glyphWidths(),
            fontMetrics = fontMetrics,
            sprites = sprites,
            windows =
                mapOf(
                    windowName to
                        WindowEntry(
                            surface =
                                SurfaceEntry(
                                    kind = "container",
                                    container = container,
                                    size = listOf(176, 144),
                                    titleOrigin = titleOrigin,
                                ),
                            static = static,
                            slots = slots,
                            spriteSlots = spriteSlots,
                            regions = regions,
                            items = items,
                            collections = collections,
                            inputs = inputs,
                            groups = groups,
                            switches = switches,
                            layers = layers,
                        ),
                ),
            huds = huds,
        )

    fun hudManifest(
        hudName: String = "h",
        channel: String = "actionbar",
        width: Int = 100,
        height: Int = 16,
        static: String = "STATIC",
        slots: Map<String, SlotEntry> = emptyMap(),
        shader: HudShaderEntry? = null,
        fontMetrics: Map<String, FontMetricsEntry> = fontMetricEntries(),
        switches: Map<String, SwitchEntry> = emptyMap(),
        layers: List<LayerEntry> =
            switches.keys.map { LayerEntry(LayerKind.SWITCH, it) } + slots.keys.map { LayerEntry(LayerKind.SLOT, it) },
    ): WindowManifest =
        WindowManifest(
            version = 8,
            namespace = "window",
            font = "window:ui",
            spacers = spacerTable(),
            textAdvances = advances(),
            textGlyphWidths = glyphWidths(),
            fontMetrics = fontMetrics,
            windows = emptyMap(),
            huds =
                mapOf(
                    hudName to
                        HudEntry(
                            surface =
                                HudSurfaceEntry(
                                    kind = "hud",
                                    channel = channel,
                                    width = width,
                                    height = height,
                                ),
                            static = static,
                            slots = slots,
                            shader = shader,
                            switches = switches,
                            layers = layers,
                        ),
                ),
        )

    fun slot(
        x: Int,
        width: Int,
        align: Align,
        font: String = "window:y0",
        color: String = "#ffffff",
        shaderMarker: String? = null,
        shaderColor: String? = null,
        text: String? = null,
        y: Int = 6,
        shadow: Boolean = false,
    ): SlotEntry =
        SlotEntry(
            x = x,
            y = y,
            width = width,
            align = align,
            font = font,
            color = color,
            shaderMarker = shaderMarker,
            shaderColor = shaderColor,
            shadow = shadow,
            text = text,
        )

    fun sprite(
        width: Int = 16,
        height: Int = 16,
        xOffset: Int = 0,
        glyphWidth: Int = width,
        advance: Int = width + 1,
        glyph: String = "\uE000",
    ): SpriteEntry =
        SpriteEntry(
            width = width,
            height = height,
            xOffset = xOffset,
            glyphWidth = glyphWidth,
            advance = advance,
            glyph = glyph,
        )

    fun spriteSlot(
        x: Int,
        y: Int,
        width: Int,
        height: Int = 16,
        align: Align = Align.LEFT,
        font: String = "window:sprite_y0",
        sprite: String? = null,
    ): SpriteSlotEntry =
        SpriteSlotEntry(
            x = x,
            y = y,
            width = width,
            height = height,
            align = align,
            font = font,
            sprite = sprite,
        )

    fun region(
        slots: List<Int>,
        action: String? = null,
        defaultAction: String? = null,
        hitbox: HitboxEntry? = null,
        fillSlots: List<Int>? = null,
    ): RegionEntry =
        regionRefs(slots.map(::containerSlot), action, defaultAction, hitbox, fillSlots?.map(::containerSlot))

    fun regionRefs(
        slots: List<SlotRefEntry>,
        action: String? = null,
        defaultAction: String? = null,
        hitbox: HitboxEntry? = null,
        fillSlots: List<SlotRefEntry>? = null,
    ): RegionEntry =
        RegionEntry(
            x = 0,
            y = 0,
            width = 18,
            height = 18,
            slots = slots,
            fillSlots = fillSlots,
            action = action,
            defaultAction = defaultAction,
            hitbox = hitbox,
        )

    /** The compiled form of a button with named states: its state switch and one region per case. */
    class StateButton(
        val switch: SwitchEntry,
        val regions: Map<String, RegionEntry>,
    )

    /**
     * Compiles button [name] over [slots] with [states] keyed by state, plus the synthesized `default`
     * state carrying [tooltip]. [sprites] names a state's sprite slot key by state.
     */
    fun stateButton(
        name: String,
        slots: List<Int>,
        states: Map<String, HitboxEntry>,
        action: String? = name,
        tooltip: TooltipEntry? = null,
        sprites: Map<String, String> = emptyMap(),
    ): StateButton {
        val all = mapOf("default" to HitboxEntry(tooltip = tooltip)) + states
        val regions = all.map { (state, hitbox) -> "$name.$state" to region(slots, action, hitbox = hitbox) }.toMap()
        val cases =
            all.keys.map { state ->
                SwitchCaseEntry(state, regions = listOf("$name.$state"), spriteSlots = listOfNotNull(sprites[state]))
            }
        return StateButton(SwitchEntry(cases, states = true, initial = "default", source = "button `$name`"), regions)
    }

    fun containerSlot(index: Int): SlotRefEntry = SlotRefEntry(SlotAreaEntry.CONTAINER, index)

    fun playerSlot(index: Int): SlotRefEntry = SlotRefEntry(SlotAreaEntry.PLAYER, index)
}
