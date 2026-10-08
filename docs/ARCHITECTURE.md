# Architecture

Window's compiler produces the resource pack and typed server bindings from the same compiled definition. Changes to
that contract must update its Rust model, Kotlin consumers, generators, and [schema reference](MANIFEST.md) together.

## Component boundaries

```text
TypeScript UI sources + art assets
    → rpp TypeScript package (plugin/src/plugin.ts)
    → WASIp2 component (crates/rpp-plugin)
    → pure Rust compiler (crates/window-core)
        → resource-pack fonts, textures, optional HUD shaders
        → deterministic debug descriptor
        → generated Kotlin pack classes and typed views
    → server-agnostic runtime (jvm/runtime)
    → server host (jvm/minestom, jvm/multistom, or your own)
    → vanilla Minecraft client
```

`window-core` accepts bytes and authored models and returns asset bytes, diagnostics, and an in-memory compiled
definition. It performs parsing, layout, rasterization, composition, codepoint allocation, and Kotlin generation without
host I/O. `compile_project_json` is the JSON entry point used by the component.

The TypeScript package discovers UI sources, collects asset bytes, removes authoring-only files from the pack output,
and invokes the component. Pack assets use rpp's pack output; Kotlin sources use a declared external output. The plugin
cannot write arbitrary server project files.

The JVM runtime consumes generated definitions, composes Adventure components, runs window sessions, and renders HUDs.
It has no server dependency: every window operation goes through a [host](#hosts), and HUDs only return components (see
[HUDs](#huds)). Generated pack data (`WindowPackData` and its entry, font, sprite, and spacer tables) is `internal` to
the module it is generated into; the view and HUD classes and the `WindowDefinitions` and `WindowHudDefinitions` objects
are public. HUD classes are identical for every Kotlin target. A JSON definition can still be loaded at runtime with
`WindowManifest.parse(json)` and `WindowDefinition(manifest, name)` or `HudDefinition(manifest, name)`, passed to a
handwritten `WindowView` or `HudView`. The example pack and server are described in
[example/README.md](../example/README.md).

## Hosts

`com.chunkzero.window.host.WindowHost<I>` is the protocol-level contract between the runtime and one player on one
server, where `I` is the server's item type. A host:

- opens a `ContainerKind` with a title and returns an `OpenContainer<I>`, reporting clicks (normalised to `SlotRef`),
  closes, anvil text, and pongs to a `ContainerListener`;
- builds the `WindowItem`s Window renders itself (invisible hitboxes and anvil seeds) as native items; and
- runs a task on the next tick that serves its player.

The KDoc on these types is the contract, including the anvil guarantees: `setTitle` on an anvil reopens the screen on
the same container without reporting `onClose`, and delivers items staged with `stageItem`. A server-initiated `close`
never reports `onClose`; a client close or a replacing inventory reports it once, synchronously, after listening has
stopped and the player's slots are restored (before the server returns a cursor item to the inventory). `onClose` runs
while the server is still closing or replacing the inventory, including on disconnect, so opening another container from
it must be scheduled, e.g. on the next tick. Clicks addressed to the player's own inventory window are dropped while a
container is open. A replacement that a server listener cancels reports nothing and leaves the current container open.
Opening a container the server refuses to show (cancelled or redirected) throws `IllegalStateException` from
`WindowHost.open` with nothing registered (a redirect has already replaced the current container, which reported
`onClose`), and the failed `WindowView.open()` still consumes the view.

Hosts hold no runtime state. `MinestomHost(player)` and `MultistomHost(player)` are cheap wrappers that compare equal
for the same player. Hosts duplicate their code rather than share it; each compiles against exactly one server.

`window-host-testkit` holds the shared behaviour tests. Implement `HostFixture<I>` for a fresh player on a headless
server and a simulated client, then extend `HostConformance`:

```kotlin
class MyHostConformanceTest : HostConformance<MyItem>(::MyFixture)
```

The Minestom and Multistom hosts both run the full kit.

## HUDs

HUDs take no input, so they are not part of the host contract. A `HudView` is a pull-based renderer over a
`HudDefinition`: `render()` runs `bind()` once to register slot and switch providers through `HudScope`, then evaluates
every provider on each call. It caches each slot's composed segment and lays a slot out again only when its provider
returns a different component, or a switch selects a different case; when nothing changed it returns the previous
component. HUDs have no reactive state and no scheduling. `HudStack(channel)` concatenates the renders of several HUDs
of one channel in insertion order.

Delivery belongs to the caller. The runtime provides Adventure helpers, `Audience.sendHud` for the action bar and
`BossBar.showHud` for a boss bar's name, and the server hosts add `Sidebar.showHud` for a sidebar title. Each rejects a
HUD composed for another channel, because shader layouts are composed per channel. The caller decides when to render and
resend (vanilla hides an action bar about 60 ticks after it arrives), creates and shows boss bars and sidebars, and
keeps any per-player state and cleanup. Neither `HudView` nor `HudStack` is thread-safe.

## Rendering coordinates

All authored and compiled coordinates are GUI pixels. Container titles start at a surface-specific `title_origin`,
`(8, 6)` for generic 9-column containers.

For a bitmap provider with ascent `A`, the top of a glyph rendered on title line `ty` is `ty + 7 - A`. A full-container
overlay beginning at y=0 therefore uses ascent 13 on generic containers. Providers must satisfy `ascent <= height` and
`height <= 512`; negative ascent places glyphs below the normal text line.

Bitmap advance follows the rightmost opaque source pixel after scaling, plus Minecraft's one-pixel gap. Transparent
padding affects visible bounds and cursor advance differently. Generated metrics and runtime measurement must agree with
that behavior.

Shifted fonts reuse Minecraft's built-in bitmap sheets at the required ascent; Window does not ship vanilla artwork.
Supported text coverage and authoring constraints are documented in [AUTHORING.md](AUTHORING.md).

## Cursor and paint contracts

Every container-title segment is net-zero: it returns the cursor to `title_origin.x`. Static chrome carries its own
spacers. Dynamic segments move to their x position, render, then compensate for both the offset and the measured
advance. Alignment uses visible bounds; compensation uses advance width.

Window paints the static chrome first, then runtime layers (text, sprite slots, switch cases, and collection selections)
in the authored tree order recorded in each definition's `layers`. Vanilla inventory items render above the title
artwork.

HUD components claim exactly their authored width so vanilla centering remains stable. Shader-relocated overlays are
individually net-zero relative to that fixed-width component. HUD origin, anchor, offsets, and source-position semantics
live in the compiled definition. Actionbar source calibration is independent of HUD height; the version-specific
constant is documented beside its implementation in `crates/window-core/src/hud/renderer/text.rs`.

## Fonts and allocation

The main font contains static bitmap providers and the spacing provider. Shifted text and sprite fonts encode distinct
vertical positions while reusing textures.

Spacers encode positive and negative powers of two through 1024 at fixed supplementary private-use codepoints. Arbitrary
movement is composed from those entries. The runtime uses the generated spacer table.

Bitmap glyph allocation hashes stable element keys into the BMP private-use area and resolves collisions
deterministically. Keep keys stable and retain allocation regression coverage when changing the emitted asset set.

## Inventory and session contracts

Inventory slots have separate ownership and click-routing assignments. Each slot has at most one item-writing owner and
one click target. A region over a slot an item, collection, or the anvil input fills routes its clicks while that
control keeps the slot's stack. `regions.*.slots` records routing for regions with an action; `fill_slots` records the
region's item ownership. Regions in mutually exclusive switch cases may claim the same slots.

Generated views expose typed bindings. The runtime validates handwritten bindings when a session opens and routes clicks
to them. The host cancels inventory interactions (its containers also refuse native click handling, so later listeners
cannot re-enable item movement), normalises their slots, and leases the player-inventory slots temporarily claimed by
the UI. Closing restores those slots and releases the session's listeners.

A window session owns the host's container: it turns container input into view calls and delivers what its renderer
produces. The renderer evaluates the view's bindings into the title and slot items without touching the host.

Reactive window state records which bindings read it. Writes mark those bindings dirty and schedule a batched update on
the host's next tick. Sessions are not thread-safe: state writes must happen on the thread that serves the host's
player. Text, sprites, and item state updates share the current compiled definition.

## Diagnostics and validation

The compiler emits `assets/<namespace>/window/debug.json`, a separate schema-v1 diagnostics descriptor. Its pack
fingerprint covers canonical compiled-definition bytes and emitted assets, excluding the descriptor itself.

`jvm/diagnostics-protocol` owns versioned render-frame data. `jvm/minestom-diagnostics` provides an optional
capability-negotiated transport. The observer registered as `WindowDiagnostics.observer` receives traces from the same
composition pass that produces the rendered component; vanilla clients receive no diagnostic payloads. Window frames
arrive through `observe(host, frame)`. A HUD frame arrives through `observeHud(hud, frame)` when `render()` composes new
content; HUDs have no player, so an observer that routes frames to players attributes them itself.

The Fabric inspector reads active resources and observes Minecraft's actual glyph pipeline. Its MC Validation extension
is compiled only when that external checkout is configured. Both builds use the same inspector implementation.

Keep deterministic compiler and runtime tests as the normal development checks. Use the optional real-client scenario
for resource activation, input, render geometry, and shader behavior. See [VALIDATION.md](VALIDATION.md) and
[INSPECTOR.md](INSPECTOR.md) for commands and fixture guidance.
