# Compiled Window Pack Definition — schema v9

The compiled definition is the typed contract between the rpp plugin and the server runtime/codegen. The rpp plugin now
emits Kotlin sources that instantiate this schema directly (`WindowPackData`, `WindowFonts`, and friends are `internal`;
`WindowDefinitions`, `WindowHudDefinitions`, and `WindowSprite` are public); it does not write a JSON manifest into the
pack output.

The JSON below is the schema's compatibility representation, used by legacy tools and parser tests. Schema changes bump
`version` and must update `window-core/src/manifest.rs`, the runtime DTOs, and codegen together.

Resource packs do contain `assets/<namespace>/window/debug.json`, but that schema-v1 document is a client diagnostics
descriptor, not this runtime manifest. It intentionally contains semantic expectations, pack/resource fingerprints, and
isolation masks while omitting server bindings and control routing. See `docs/INSPECTOR.md`.

All coordinates are GUI-space pixels (origin = container/HUD top-left). All codepoints are serialized as the actual
characters inside JSON strings, except in `spacers` where they are integers for readability.

The definition holds only primitives: baked art (`static`), text (`slots`), images (`sprite_slots`), inventory
`regions`, `items`, `collections`, `inputs`, and `switches` whose cases hold any of these, including other switches.
Authoring components compile into these: a control with states, such as an `industrial.Button` with `enabled`, is a
switch on its handle with one region per state.

```jsonc
{
  "version": 9,
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
          "overflow": "ellipsis", // optional: shorten content wider than `width`
        },
        "holder": {
          "x": 8,
          "y": 20, // top of the slot's box, (count - 1) * line_height + 8 px tall
          "width": 46,
          "align": "center",
          "font": "window:y14",
          "color": "#ffffff",
          "shadow": false,
          "overflow": "ellipsis",
          // Optional: wrap onto at most `count` lines. fonts[s] draws a line at y + floor(s * line_height / 2).
          "lines": { "count": 2, "line_height": 7, "fonts": ["window:y14", "window:y17", "window:y21"] },
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
          "source": "tool-icon", // optional: the nearest authored `debugName`, for diagnostics
        },
      },

      // Inventory regions: the slots each claims and fills with its hitbox item, and the action its clicks name.
      // Regions directly in a switch case are claimed only while that case is active.
      "regions": {
        "buy": {
          "x": 52,
          "y": 190,
          "width": 72,
          "height": 20,
          // Slots whose clicks route to this region.
          "slots": [
            { "area": "container", "index": 46 },
            { "area": "container", "index": 47 },
            { "area": "player", "index": 0 },
          ],
          "action": "buy", // the handler key clicks name; absent ⇒ hover- or claim-only
          // Optional item filling the slots; absent leaves them empty.
          "hitbox": {
            "item_model": "example:gui/buy", // absent ⇒ the pack's invisible hitbox model
            "tooltip": { "title": "Buy", "lines": ["Spend coins"] },
          },
          "source": "region `buy`", // the authored element, for diagnostics
        },
        // The disabled state of the same button: hover-only, so it has no action.
        "region~0": {
          "x": 52,
          "y": 190,
          "width": 72,
          "height": 20,
          "slots": [/* same as buy */],
          "hitbox": { "tooltip": { "title": "Need more coins" } },
          "source": "region `region~0`",
        },
        "window:close": {
          "x": 8,
          "y": 190,
          "width": 40,
          "height": 20,
          "slots": [{ "area": "container", "index": 45 }],
          "action": "window:close",
          "default_action": "window:close", // a runtime action: works unbound
          "source": "region `window:close`",
        },
        "clear": {
          // A region over the anvil input's slot routes its clicks while the input keeps its seed item.
          "x": 27,
          "y": 47,
          "width": 16,
          "height": 16,
          "slots": [{ "area": "container", "index": 0 }],
          "fill_slots": [], // optional strict subset of "slots" filled with the hitbox; absent ⇒ every slot
          "action": "clear",
          "source": "region `clear`",
        },
        "player_section": {
          // A claim-only region from a section's unowned slots: no action, no hitbox.
          "x": 8,
          "y": 140,
          "width": 160,
          "height": 16,
          "slots": [
            { "area": "player", "index": 9 },
            { "area": "player", "index": 10 },
          ],
          "source": "player section",
        },
      },

      // Dynamic inventory item regions. Runtime code binds `item("featured")`. Items directly in a switch case
      // are filled only while that case is active.
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
              "sprite": "art/industrial-slot-selected-2423c7",
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

      // Runtime-selected cases by key. Only the active case's net-zero `static`
      // art, slots, sprite slots, regions, items, collections, and switches are drawn or claimed.
      "switches": {
        "mode": {
          "cases": [
            { "value": "buy", "static": "󰀀…", "slots": ["price"], "switches": ["stock"] },
            { "value": "sell", "static": "󰀀…", "slots": ["label_3", "payout"], "items": ["featured"] },
          ],
        },
        // Nested in case `buy` of `mode`: active only while `mode` is `buy`.
        "stock": {
          "cases": [
            { "value": "low", "slots": ["count"] },
            { "value": "high", "static": "󰀀…", "slots": ["count~2"] },
          ],
        },
        // The state switch of a button with `enabled={canBuy}`, keyed by the flag's entry.
        "can_buy": {
          "source": "buy-button", // optional: the switch's `debugName`, for diagnostics
          "cases": [
            { "value": "true", "static": "󰀀…", "slots": ["buy_label"], "regions": ["buy"] },
            { "value": "false", "static": "󰀀…", "slots": ["buy_label~2"], "regions": ["region~0"] },
          ],
        },
      },

      // Every slot, sprite slot, switch, and selectable collection in authored
      // tree order: the order runtime layers compose above `static`.
      "layers": [
        { "kind": "slot", "name": "title" },
        { "kind": "switch", "name": "mode" },
        { "kind": "slot", "name": "price" },
        { "kind": "switch", "name": "stock" },
        { "kind": "slot", "name": "count" },
        { "kind": "slot", "name": "count~2" },
        { "kind": "slot", "name": "label_3" },
        { "kind": "slot", "name": "payout" },
        { "kind": "collection", "name": "entries" },
        { "kind": "switch", "name": "can_buy" },
        { "kind": "slot", "name": "buy_label" },
        { "kind": "slot", "name": "buy_label~2" },
      ],

      // Typed handles by id, with the entries that use them.
      "handles": {
        "category": {
          "kind": "selection",
          "values": ["all", "gear"],
          "initial": "gear",
          "uses": [
            { "role": "click", "entry": "category=all", "value": "all" },
            { "role": "switch", "entry": "category?gear", "value": "gear" },
          ],
        },
        "can_buy": { "kind": "flag", "uses": [{ "role": "switch", "entry": "can_buy" }] },
        "buy_label": {
          "kind": "text",
          "uses": [
            { "role": "slot", "entry": "buy_label" },
            { "role": "slot", "entry": "buy_label~2" },
          ],
        },
        "names": { "kind": "text", "shape": [2], "uses": [{ "role": "slot", "entry": "names[0]", "at": [0] }] },
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

      // HUDs carry `switches` and `layers` like windows; HUD layers hold slots and switches.

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
- `sprites` holds the `defineWindows` sprite catalog and inline art that the runtime draws (a collection's selected
  sprite). Inline art is keyed `art/{name}-{hash}`, or `art/{hash}` without a name, where `hash` is a hex prefix of the
  art's content hash; codegen gives these no `WindowSprite` constant.
- `bold` changes text advance by `bold_advance` per rendered character. Italic, underline, strikethrough, and obfuscated
  are style defaults but do not change cursor measurement.
- `regions.*.slots`, `items.*.slots`, `collections.*.slots`, and `inputs.*.slot` are typed slot references.
  `area = "container"` addresses the opened inventory. `area = "player"` addresses the viewing player's inventory, with
  Minestom's slot indices (`0..8` hotbar, `9..35` main inventory). Runtime player slots are snapshotted before Window
  writes them and restored when the menu closes.
- Regions take the slots their laid-out rectangle overlaps: their parent box's, or their section area's. If no slot
  overlaps, the build fails. Items and collections take slots resolved from authoring `slots`, `pattern`, `transform`,
  their section area, or the slots their parent box covers.
- Slot _ownership_ and slot _routing_ are separate contracts. Exactly one active control fills any given slot:
  `regions.*.fill_slots` (defaulting to `regions.*.slots`), `items.*.slots`, `collections.*.slots`, and `inputs.*.slot`
  must be disjoint. Independently, the `slots` of regions with an `action` must be disjoint, because a slot can only
  route its clicks to one region. Regions, items, and collections in mutually exclusive switch cases are exempt from
  both: two entries are exclusive when their case paths (the cases enclosing them, outermost first) pick different cases
  of the same switch. A region over a slot an item, collection, or anvil input fills relies on the ownership/routing
  split: it routes the slot while the other control fills it, and `fill_slots` leaves the slot out (it may be empty).
  The runtime writes the hitbox into `fill_slots` of every active region and routes `slots`; a click on an action
  collection's cell goes to the collection even when a region also routes that slot.
- `regions.*.action` names the handler a click runs. The runtime runs the handler bound to that id, else
  `default_action`; a region without `action` ignores clicks. Ids in the `window:` namespace (`window:close`) are
  runtime actions: codegen declares no member for them, and they always carry the same `default_action`, so they work
  unbound. A region's `action` is its entry name, so regions of the same action handle have one action each (`buy`,
  `buy~2`), all bound to the handle's member.
- `regions.*.hitbox` is the item the runtime writes into the region's fill slots: `item_model` (default: the emitted
  invisible `{namespace}:gui/hitbox` model) and an optional `tooltip`. Without `hitbox` the slots stay empty.
- `inputs` describes native inventory input metadata. It is currently valid only for `surface.container = "anvil"`;
  exactly one input owns container slot `0`, whose item name drives the vanilla edit field.
- `handles` is optional metadata for codegen on windows and HUDs; runtimes ignore it. `kind` is `flag`, `toggle`,
  `value`, `selection`, `text`, `sprite`, `items`, `collection`, `action`, `input`, or `builtin`. `values` lists a value
  or selection's values, `shape` an indexed handle's extents, `initial` a toggle's (`"true"`/`"false"`) or selection's
  initial value, `selectable` whether a collection marks a selected cell, and `only` the sprites a sprite handle
  returns, which codegen generates as an enum of those `WindowSprite` constants. Each use names the entry it binds: a
  `slot`, `sprite_slot`, `item`, `collection`, `input`, `switch`, or a region's `click`. `at` is the index read from an
  indexed handle, and `value` the value a condition compares with or a click sets. A use is keyed `{id}`, then `[{i}]`
  per index, `={value}` for a click that sets a value, or `?{value}` for a condition, with `~2`, `~3`, … appended to
  later uses of the same key. Codegen generates members only from handles and binds every use to its handle's member; an
  entry no handle uses gets no member.
- `switches` is optional on windows and HUDs. At most one case of a switch is active: the case whose `value` the
  switch's binding returns, or none until it is bound. A switch named in a case's `switches` is nested: it and
  everything in its cases are active only while that case is. A case lists only the entries directly inside it. The
  runtime draws the active cases' `static` at the switch's position in `layers`, leaves out the slots, sprite slots, and
  collection selections of inactive cases, and claims only the regions, `items`, and `collections` of active cases, so
  changing a case swaps the items and click routes of its slots. A click routes by the cases selected when it arrives:
  the runtime reads each enclosing switch's binding at click time rather than waiting for the next render. Each case's
  `static` is net-zero: window cases start and end at `title_origin.x`, HUD cases at the HUD's left edge. Codegen binds
  a `Boolean` when the case values are exactly `true` and `false`, otherwise an enum of the values.
- `layers` lists every slot, sprite slot, switch, and collection with a selected sprite exactly once, in authored tree
  order. The runtime composes `static` first, then these layers in order, skipping entries in inactive cases. A switch
  layer draws its active case's `static`; a collection layer draws its selected cell's `selection` sprite.
- `slots.*.overflow` and `slots.*.lines` appear only on dynamic slots. With `overflow: "ellipsis"`, runtimes truncate
  rendered content whose visible width exceeds `width` and append "…", or "..." when the slot's font metrics have no
  advance for "…", keeping the styling of the kept text. With `lines`, runtimes wrap content at spaces onto at most
  `count` lines, ellipsizing a word wider than `width` and the last line when text is left over (`overflow` is then
  always set). When `k` lines are used, line `j` (0-based) draws in `fonts[count - k + 2j]`, whose top is
  `y + floor((count - k + 2j) * line_height / 2)`, which centers the used lines in the box. `fonts` has `2 * count - 1`
  entries and `fonts[0]` equals `font`.
- `tooltip` may be absent, or an object with `title` and optional `lines`. Authoring accepts a string shorthand, but the
  compiled definition always uses the object form.
- `source` on regions names the authored element they come from, such as ``region `category=all` `` or `player section`,
  for diagnostics. On slots, sprite slots, and switches it is the `debugName` of the nearest authored element that sets
  one.
- Entry keys are unique per window or HUD across all maps. Handle ids match `^[a-z][a-z0-9_]*$`; entries derived from
  them add `[`, `]`, `?`, `=`, or `~`, and runtime actions and section claims use `:` and `_section`.
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
