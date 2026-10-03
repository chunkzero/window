# Compiled Window Pack Definition — schema v5

The compiled definition is the typed contract between the rpp plugin and the server runtime/codegen. The rpp plugin now
emits Kotlin sources that instantiate this schema directly (`WindowPack`, `WindowFonts`, `WindowDefinitions`, and
friends); it does not write a JSON manifest into the pack output.

The JSON below is the schema's compatibility representation, used by legacy tools and parser tests. Schema changes bump
`version` and must update `window-core/src/manifest.rs`, the runtime DTOs, and codegen together.

Resource packs do contain `assets/<namespace>/window/debug.json`, but that schema-v1 document is a client diagnostics
descriptor, not this runtime manifest. It intentionally contains semantic expectations, pack/resource fingerprints, and
isolation masks while omitting server bindings and control routing. See `docs/INSPECTOR.md`.

All coordinates are GUI-space pixels (origin = container/HUD top-left). All codepoints are serialized as the actual
characters inside JSON strings, except in `spacers` where they are integers for readability.

```jsonc
{
  "version": 5,
  "namespace": "window",
  "font": "window:ui", // font id of the main (static + spacer) font

  // Spacer table: codepoint (integer) → horizontal advance in pixels.
  // Values are ±1, ±2, ±4 … ±1024. Compose arbitrary offsets from these.
  "spacers": {
    "983040": -1024,
    "983041": -512,
    /* … */ "983061": 1024,
  },

  // Vanilla default-font advance widths (pixels per character: cursor movement
  // after rendering). Applies to every shifted label font too. Consumers use
  // this for net-zero spacer math instead of bundling their own table.
  "text_advances": { " ": 4, "!": 2, /* … */ "~": 7 },

  // Vanilla default-font visible glyph widths (pixels per character, excluding
  // the inter-glyph gap). Consumers use this for visual alignment.
  "text_glyph_widths": { " ": 0, "!": 1, /* … */ "~": 6 },

  // Per-font metrics keyed by font id. Generated definitions include
  // "minecraft:default", every generated shifted text font, and every
  // text-font offset ("window:small_caps/y0"), whose sheet glyphs replace
  // the vanilla entries.
  "font_metrics": {
    "window:y0": {
      "advances": { " ": 4, "!": 2, /* … */ "~": 7 },
      "glyph_widths": { " ": 0, "!": 1, /* … */ "~": 6 },
      "bold_advance": 1,
    },
  },

  // Runtime sprite catalog. The same glyph is repeated into every generated
  // sprite_y... font needed by authored sprite slots; those font providers
  // reference the same texture file, so PNGs are not duplicated by height.
  // width/height are the authored visual canvas. x_offset/glyph_width are the
  // measured visible ink bounds used for alignment. advance is the Minecraft
  // cursor movement from the rightmost opaque pixel after bitmap scaling, plus
  // the one-pixel bitmap gap.
  "sprites": {
    "wooden_pickaxe": {
      "width": 32,
      "height": 32,
      "x_offset": 0,
      "glyph_width": 32,
      "advance": 33,
      "glyph": "",
    },
  },

  "windows": {
    "shop": {
      "surface": {
        "kind": "container",
        "container": "generic_9x6", // window-core surface kind id
        "size": [176, 222], // GUI texture size
        "title_origin": [8, 6], // where the title cursor starts
      },

      // Net-zero baked chrome. Send as-is in font `font` before slot segments.
      "static": "󰀀…",

      // Dynamic text slots, including static labels (which carry "text").
      "slots": {
        "title": {
          "x": 8,
          "y": 6, // top-left of the text line (8px tall)
          "width": 160, // reserved width for alignment/clipping
          "align": "center", // "left" | "center" | "right"
          "font": "window:y0", // shifted font for this y offset; "window:<font>/y0" for text fonts
          "color": "#404040", // default text color (hex, lowercase)
          "shadow": false,
          "bold": false, // optional, omitted when false
          "italic": false,
          "underlined": false,
          "strikethrough": false,
          "obfuscated": false,
        },
        "buy_label": {
          "x": 58,
          "y": 196,
          "width": 60,
          "align": "center",
          "font": "window:y190",
          "color": "#ffffff",
          "shadow": false,
          "text": "Buy", // present ⇒ static label, auto-rendered
        },
      },

      // Runtime sprite regions. Runtime code binds a sprite catalog key; the
      // slot's font carries the correct ascent for this y offset.
      "sprite_slots": {
        "selected_tool_icon": {
          "x": 112,
          "y": 45,
          "width": 32,
          "height": 32,
          "align": "center",
          "font": "window:sprite_y39",
          "sprite": "icon_tool", // optional fixed sprite; absent ⇒ runtime binding
        },
      },

      // Clickable regions mapped to typed backing inventory slots.
      "buttons": {
        "buy": {
          "x": 52,
          "y": 190,
          "width": 72,
          "height": 20,
          // Slots whose clicks route to this button.
          "slots": [
            { "area": "container", "index": 46 },
            { "area": "container", "index": 47 },
            { "area": "player", "index": 0 },
          ],
          // Optional strict subset of "slots" that this button fills with its
          // own hitbox/state item. Absent means "every slot in `slots`". A
          // repeater cell emits this when it yields a slot to an item control.
          "fill_slots": [
            { "area": "container", "index": 47 },
            { "area": "player", "index": 0 },
          ],
          "default": null, // or "close"
          "action": true, // false for hover-only hotspots; omitted when true
          "tooltip": {
            "title": "Buy",
            "lines": ["Spend coins"],
          },
          "sprite_font": "window:sprite_y184",
          "states": {
            "disabled": {
              "item_model": "example:gui/buy_disabled",
              "sprite": "wide_button_disabled",
              "tooltip": { "title": "Need more coins" },
            },
          },
        },
      },

      // Dynamic inventory item regions. Runtime code binds `item("featured")`.
      "items": {
        "featured": {
          "slots": [{ "area": "container", "index": 13 }],
        },
      },

      // Repeated dynamic item regions. Slots are ordered; collection click handlers
      // receive the clicked cell index. Set `"action": false` for display-only grids.
      // `selection` holds one sprite slot per cell, in cell order, drawn over the cell's
      // 18x18 slot box; it is omitted when the collection has no `selected_sprite`.
      "collections": {
        "entries": {
          "slots": [
            { "area": "container", "index": 19 },
            { "area": "container", "index": 20 },
            { "area": "player", "index": 9 },
          ],
          "action": true,
          "selection": [
            {
              "x": 7,
              "y": 17,
              "width": 18,
              "height": 18,
              "align": "left",
              "font": "window:sprite_y11",
              "sprite": "cell_selected",
            },
            // …one entry per cell
          ],
        },
      },

      // Native text inputs. Anvil surfaces support one input, backed by slot 0.
      // Generated views expose an `onQueryChanged(value: String)` member.
      "inputs": {
        "query": {
          "slot": { "area": "container", "index": 0 },
          "initial": "",
          "item_model": "example:gui/search_input",
        },
      },

      // Non-binding slot claims/fills. These are produced by `slot_rects`
      // authoring primitives and do not generate Kotlin abstract members.
      "slot_rects": {
        "inventory_fill": {
          "slots": [
            { "area": "player", "index": 9 },
            { "area": "player", "index": 10 },
          ],
        },
      },

      // Group metadata for controls flattened from repeaters. Codegen uses
      // this to expose indexed methods while the runtime still binds the
      // flattened controls by source name.
      "groups": {
        "entry": {
          "count": 2,
          "slots": {
            "price": ["entry_price_0", "entry_price_1"],
          },
          "sprite_slots": {
            "icon": ["entry_icon_0", "entry_icon_1"],
          },
          "items": {
            "stack": ["entry_stack_0", "entry_stack_1"],
          },
          "buttons": ["entry_0", "entry_1"],
        },
      },
    },
  },

  "huds": {
    "status": {
      "surface": {
        "kind": "hud",
        "channel": "actionbar", // "actionbar" | "bossbar" | "sidebar"
        "width": 160, // fixed line width in GUI pixels
        "height": 16,
      },

      // Fixed-width baked chrome. Send as-is in font `font` before slot
      // overlays; it intentionally claims exactly `surface.width` pixels.
      "static": "󰀀…",

      // Same slot shape as windows. Static labels carry "text"; dynamic slots
      // are bound by generated HudView subclasses.
      "slots": {
        "coins": {
          "x": 54,
          "y": 4,
          "width": 98,
          "align": "right",
          "font": "window:y4",
          "color": "#ffffff",
          "shader_marker": "#120034",
          "shadow": false,
        },
      },

      // Present only when the authored HUD requested core-shader relocation.
      "shader": {
        "static_marker": "#010002",
        "source_bottom": 59,
        "origin_x": 0.5,
        "origin_y": 0.08,
        "anchor_x": 0.5,
        "anchor_y": 0.0,
        "offset_x": 0,
        "offset_y": 0,
      },
    },
  },
}
```

Notes:

- `windows.*.static` already starts and ends at `title_origin.x` (net-zero) — consumers concatenate segments without
  bookkeeping.
- `font_metrics` is the preferred measurement table for styled runtime text. `text_advances` and `text_glyph_widths`
  remain for compatibility and are the same vanilla default-font metrics used by generated shifted fonts.
- Shader HUDs render from the actionbar center: `huds.*.static` and every slot segment are net-zero, and the generated
  shader moves the local HUD coordinate system into place. Non-shader HUD fallbacks keep a fixed-width static segment so
  vanilla actionbar/bossbar/sidebar centering stays stable.
- A slot's vertical placement is fully encoded in its `font`; `y` is informational (and used by codegen/tooling).
- Runtime sprite slot vertical placement is likewise encoded in `sprite_slots.*.font`. The sprite glyph comes from
  `sprites.<id>.glyph`, and the generated `sprite_y...` font repeats that glyph at the slot's vertical ascent.
- `bold` changes text advance by `bold_advance` per rendered character. Italic, underline, strikethrough, and obfuscated
  are style defaults but do not change cursor measurement.
- `buttons.*.slots`, `items.*.slots`, `collections.*.slots`, `inputs.*.slot`, and `slot_rects.*.slots` are typed slot
  references. `area = "container"` addresses the opened inventory. `area = "player"` addresses the viewing player's
  inventory, with Minestom's slot indices (`0..8` hotbar, `9..35` main inventory). Runtime player slots are snapshotted
  before Window writes them and restored when the menu closes.
- Buttons/hotspots may omit `slots`; Window infers every container/player slot overlapped by their rectangle. If no slot
  overlaps, the build fails. Items and collections require slots resolved from authoring `slots`, `pattern`,
  `transform`, or — inside a repeater — `cell_slot`.
- Slot _ownership_ and slot _routing_ are separate contracts. Exactly one control fills any given slot:
  `buttons.*.fill_slots` (defaulting to `buttons.*.slots`), `items.*.slots`, `collections.*.slots`, `inputs.*.slot`, and
  `slot_rects.*.slots` must be disjoint. Independently, `buttons.*.slots` and the slots of action collections must be
  disjoint from each other, because a slot can only route its clicks to one control. A repeater cell relies on exactly
  this split: it routes all of its slots while an item control fills one of them. A button over an anvil input's slot
  uses it too, and may leave `fill_slots` empty. The runtime writes `fill_slots` and routes `slots`.
- `inputs` describes native inventory input metadata. It is currently valid only for `surface.container = "anvil"`;
  exactly one input owns container slot `0`, whose item name drives the vanilla edit field.
- `slot_rects` entries are runtime fills only. The visual frames are already in the baked static layer.
- `groups` is optional metadata for codegen. Runtime routing does not depend on it.
- `tooltip` may be absent, or an object with `title` and optional `lines`. Authoring accepts a string shorthand, but the
  compiled definition always uses the object form.
- `states` maps author-chosen names to runtime-selectable inventory item states. `item_model` is optional; when omitted,
  the runtime uses the emitted invisible `{namespace}:gui/hitbox` model. `tooltip` overrides the button's default
  tooltip for that state.
- `action: false` denotes a hover-only hotspot. Codegen skips it and the runtime never requires a click handler.
- Slot, button, item, and collection names match `^[a-z][a-z0-9_]*$` and are unique per window across all maps (codegen
  turns them into members of one class).
- HUD slot names follow the same pattern and are unique per HUD.
- `shader` is metadata for generated core-shader packs. `origin_*` is a normalized GUI target point, `anchor_*` is the
  normalized point inside the HUD surface placed on that target, and `offset_*` is a GUI-pixel nudge. `source_bottom` is
  the nominal bottom of the source text surface measured up from the GUI bottom; generated shaders apply Minecraft's
  fixed actionbar source inset, independent of the authored HUD height. Runtime display still uses the HUD's fallback
  `channel`.
- Shader HUDs include generated marker colors in the compiled definition: `shader.static_marker` for baked chrome and
  `slots.*.shader_marker` for slot text. These marker colors reserve green channel `00`; red+blue encode a generated
  16-bit segment id. Generated shaders only move exact ids in their table, then restore the authored display color, so
  unrelated green-zero text does not relocate.
- Generated Kotlin is deterministic: files use stable iteration order, fixed headers, and `\n` endings. Legacy JSON
  serializers should keep sorted keys so identical inputs stay byte-identical.
