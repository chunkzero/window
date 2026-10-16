# Window

Custom Minecraft UIs for [Minestom](https://minestom.net)-based servers, with no client mods.

You describe a UI in TypeScript. Window compiles it into resource-pack fonts and shaders that vanilla clients draw
through the inventory title and HUD, and generates a typed Kotlin view for each window. You implement the view on the
server. If you rename a button in the UI, the server code stops compiling instead of breaking at runtime.

Inventory UIs support real items, scrolling collections, paging, toggles, choices, tooltips, and native anvil text
input. HUDs are positioned with core shaders and rendered by the server whenever it wants to update them. Full-screen
windows are drawn over the whole screen with no screen open, and the player aims a cursor by turning.

> [!WARNING]
>
> Window is early alpha software (`0.1.0-alpha.0`). There are known bugs, and the TypeScript API, generated Kotlin, and
> runtime will change in breaking ways between releases. Pin a version and expect to update your UIs when you upgrade.

## How it works

Write a UI in your [rpp](https://github.com/chunkzero/rpp) resource-pack project, as JSX laid out automatically with
flexbox and slot grids:

```tsx
// src/window/confirm.tsx
import { Section, Text, Window } from "plugin:window/ui";
import { action, text } from "plugin:window/bind";
import { Button } from "plugin:window/theme/industrial";

export default (
  <Window name="confirm" container="generic_9x3">
    <Section of="container">
      <Text bind={text("question")} span={9} />
      <Button onClick={action("accept")} at={[3, 1]} tooltip="Confirm">
        <Text bold>YES</Text>
      </Button>
    </Section>
  </Window>
);
```

Or with the function-style API under `raw`, which builds the same primitives and calls components as functions:

```ts
// src/window/confirm.ts
import { action, text } from "plugin:window/bind";
import * as raw from "plugin:window/raw";
import { Button } from "plugin:window/theme/industrial";

export default raw.ui({
  name: "confirm",
  container: "generic_9x3",
  children: [
    raw.text(text("question"), { x: 8, y: 6, width: 160, align: "center" }),
    raw.section("container", {
      children: [
        Button({
          onClick: action("accept"),
          at: [3, 1],
          tooltip: "Confirm",
          children: raw.text("YES", { bold: true }),
        }),
      ],
    }),
  ],
});
```

`rpp build` produces the resource pack and a Kotlin base class, which you extend on the server:

```kotlin
class Confirm(player: Player, private val question: String, private val onYes: () -> Unit) : ConfirmView(player) {
    override fun question(): Component = Component.text(question)

    override fun onAccept(click: Click) = onYes()
}

Confirm(player, "Buy this sword?") { buySword(player) }.open()
```

Window views are single-use: create a new one each time you `open()` a window. HUDs work differently; see [HUDs](#huds).

## Getting started

Add the plugin to an rpp project and configure where the generated Kotlin goes:

```bash
rpp add window
```

```ts
// rpp.config.ts
import { defineConfig } from "rpp:config";
import window from "plugin:window";

export default defineConfig({
  pack: { name: "my-pack", description: "My server pack", format: 88 },
  plugins: [
    window({
      namespace: "window",
      kotlin: { package: "com.example.ui", output: "../server/src/main/kotlin/com/example/ui", target: "minestom" },
    }),
  ],
});
```

The server depends on the host for its server (see [Servers](#servers)), which brings the runtime. Pin the same version
as the plugin, and keep your own Minestom dependency; hosts compile against it but do not bring it:

```kotlin
repositories {
    maven("https://maven.chunkzero.com")
    mavenCentral()
}
dependencies { implementation("com.chunkzero.window:window-minestom:0.1.0-alpha.0") }
```

Nightly versions come from `https://maven.chunkzero.com/nightlies`; see [Releasing](docs/RELEASING.md).

## Servers

The Kotlin runtime (`window-runtime`) does not depend on any server. A host connects windows to one: it opens
containers, builds items, and schedules work for one player. Pick the host for your server and set the plugin's
`kotlin.target` to match:

| Server                           | Artifact                                | Host                    | `target`      |
| -------------------------------- | --------------------------------------- | ----------------------- | ------------- |
| [Minestom](https://minestom.net) | `com.chunkzero.window:window-minestom`  | `MinestomHost(player)`  | `"minestom"`  |
| Multistom                        | `com.chunkzero.window:window-multistom` | `MultistomHost(player)` | `"multistom"` |
| Anything else                    | `com.chunkzero.window:window-runtime`   | your own `WindowHost`   | `"agnostic"`  |

With `minestom` or `multistom`, generated window views take a `Player`, use that player's host, and expose the player to
subclasses as `player`. With `agnostic`, window views take a `WindowHost<I>` and are generic over the server's item
type:

```kotlin
class Confirm<I : Any>(host: WindowHost<I>, private val onYes: () -> Unit) : ConfirmView<I>(host) {
    override fun question(): Component = Component.text("Buy this sword?")

    override fun onAccept(click: Click) = onYes()
}

Confirm(MyHost(player)) { buySword(player) }.open()
```

To write a host, see [Hosts](docs/ARCHITECTURE.md#hosts).

Window is not thread-safe. Open window views and write their `state` only on the thread that serves the player, such as
Minestom's event handlers and `player.scheduler()`; Window schedules window re-renders on that thread too.

The [example](example/README.md) is a complete project: a shop with a paged catalog, filters and anvil search, and a set
of shader HUDs, running on a Minestom server. To try it:

```bash
just example-pack    # build the resource pack and Kotlin bindings
just example-run     # start the server on :25565
```

Then connect with a Minecraft 26.2 client.

## HUDs

A HUD only produces a `Component`; Window does not deliver, schedule, or track it. Generated HUD classes are identical
for every `target` and take no player or host, and the generated `WindowDefinitions` and `WindowHudDefinitions` objects
are public, so views written in other modules can pass their definitions to `WindowView` or `HudView` directly.
Implement the slot members, then call `render()` whenever you want the current content:

```kotlin
class Status(private val player: Player) : StatusHud() {
    override fun coins(): Component = Component.text(balanceOf(player))
}

val status = Status(player)
player.scheduler().submitTask {
    if (!player.isOnline) return@submitTask TaskSchedule.stop()
    player.sendHud(status)
    TaskSchedule.seconds(1)
}
```

`render()` runs `bind()` once, then evaluates every slot and switch on each call, reusing the layout of slots whose
component did not change. `HudStack(channel)` joins several HUDs of one channel into one component, in the order they
were added. The runtime sends with Adventure: `Audience.sendHud(hud)` for action-bar HUDs and `BossBar.showHud(hud)` for
boss-bar HUDs, each with a `HudStack` overload. The Minestom and Multistom hosts add `Sidebar.showHud(hud)`.

You own the rest:

- When to render and how often to resend. Vanilla hides an action bar about 60 ticks after it was sent, so resend it
  sooner to keep it visible.
- Sending each HUD on its definition's `channel`. Shader layouts are composed per channel, so the helpers reject a HUD
  composed for another one.
- Creating the boss bar or sidebar, showing it to players, and hiding it again.
- Per-player state and cleanup, such as stopping the resend task when the player leaves.
- Threading: a `HudView` or `HudStack` is not thread-safe, so render it from one thread at a time.

## Full-screen windows

A full-screen window is authored with [`<Fullscreen>`](docs/AUTHORING.md#full-screen-windows) and generates a typed
view, such as `LobbyFullscreen`, that extends `FullscreenView`. With the `minestom` target it takes a `Player`; with
other targets it takes a `FullscreenHost`, which only `window-minestom` implements (`MinestomFullscreenHost`). The
player aims by turning, clicks with the attack key, and leaves by sneaking. An excerpt of the example's
[`MyLobby`](jvm/example/src/main/kotlin/com/chunkzero/window/example/MyLobby.kt):

```kotlin
class MyLobby(player: Player) : LobbyFullscreen(player) {
    private var name by state("${player.username}'s Foundry")
    private var draft: String? by state(null)
    private val worlds = list(cells = 3, key = { it: String -> it }, step = 1) { WORLDS }

    override fun foundryName(): Component = Component.text(draft?.let { "${it}_" } ?: name)

    override fun worldName(index: Int): Component = Component.text(worlds.at(index).orEmpty())

    // A click lets the player type the name in place, until they press Escape.
    @OptIn(ExperimentalFullscreenApi::class)
    override fun onRename(click: FullscreenClick) {
        draft = name
        promptText(name, onEdit = { draft = it }) { text ->
            draft = null
            text?.let { name = it }
        }
    }

    override fun onScroll(steps: Int) = repeat(abs(steps)) { if (steps > 0) worlds.next() else worlds.previous() }

    // The other generated bindings and handlers are omitted.
}

MyLobby(player).open()
```

- Handlers receive a `FullscreenClick` with the cursor's canvas position. `onScroll(steps)` gets the scroll wheel as
  signed steps, and `onRightClick(click)` gets right clicks anywhere on the canvas.
- `list(...)` pages a list through a `<Repeater fixed>`'s cells, like `WindowView.list(...)`.
- Two text prompts share one slot: opening either replaces an open prompt, whose callback gets `null`, as it does when a
  server container opens over the prompt or the window closes. While one is open, canvas input is ignored and the
  hovered control's tooltip is hidden.
  - `promptText(initial, onEdit) { text -> }`, experimental (`@OptIn(ExperimentalFullscreenApi::class)`), lets the
    player type while the canvas stays in view. Nothing of it is drawn: draw the text from `onEdit`, which re-renders at
    once. Escape finishes with the text, and a click on a slot or outside the anvil may finish it too; there is no
    cancel. The text holds at most 50 characters, and `initial` is cut to fit.
  - `promptTextDialog(title, initial) { text -> }` shows a native dialog over the canvas with a text field and Confirm
    and Cancel buttons, and gets `null` on Cancel. It holds up to 256 characters, and Escape does not dismiss it. Use it
    when the player must be able to back out, or to move the caret.
- Views are single-use, as with windows, and a player has at most one full-screen window open.

While the window is open, the Minestom host:

- keeps the player in spectator mode on the server and tells only their client it is in survival, so clicks, the scroll
  wheel, and number keys reach the server. Game mode, ability, hotbar, time, and max health updates the server sends
  meanwhile still reach the client, followed by packets that hide them again. Closing restores the player's game mode
  (or the one the server set meanwhile), abilities, position, and held slot.
- hides the HUD by holding the client's world age negative, which the pack's shaders read. The hidden strip at the
  bottom of the screen can clip the player's own inventory (E) at small GUI heights, since the server is not told it
  opened. While the keyboard prompt is open, the age sits in a second negative band in which the shaders also hide the
  untitled anvil that captures the typing.
- closes the window when the player dies or spawns into another instance. Opening one while the player is dead or
  respawning throws, so a respawn listener should open it on the next tick.

Limitations:

- First person only: third person (F5) mirrors the camera, so the drawn cursor and the server's hit-test disagree.
- Shader packs that replace world rendering, such as Iris, are not supported.
- The client sends at most one held-slot change per tick, so `onScroll` gets one update per tick of at most 4 steps
  either way, more than 4 notches in one tick read as a scroll the other way, and number keys look like scrolling.
- Chat and other GUI overlays draw over the canvas, and there are no real items.
- The keyboard prompt is experimental. The server never sees the caret, so draw it at the end of the text. The anvil
  holds 50 characters, so longer text continues in a fresh field: text in earlier fields can no longer be edited, and
  keys typed just as the fresh field opens can be lost. While it is open, GUI text and fills in the anvil's rename field
  and label bands, such as chat lines crossing them, are hidden, and the system mouse pointer shows. Each edit takes a
  round trip to reach `onEdit`. Without the pack's shaders, the player sees a plain vanilla anvil.

## Supported versions

Window targets Minecraft 26.2 (resource pack format 88). The hosts and example server use Minestom builds for 26.2, and
Minestom serves a single protocol version, so they accept only 26.2 clients. The [inspector](docs/INSPECTOR.md) also
targets 26.2.

The compiler still emits packs for 26.1.x (pack format 84): its glyph metrics are shared with 26.2, and
`coreShaders.hud` selects the core text shaders for the configured `pack.format`; `coreShaders.hover` needs pack formats
85 through 88. Serving 26.1.x clients requires a server that speaks their protocol; the Minestom and Multistom hosts do
not.

## Documentation

- [Authoring](docs/AUTHORING.md): the TypeScript API and layout rules
- [Architecture](docs/ARCHITECTURE.md): how the compiler, plugin, and runtime fit together
- [Compiled definition](docs/MANIFEST.md): the contract between the compiler and the runtime
- [Validation](docs/VALIDATION.md): automated checks and real-client scenarios
- [Inspector](docs/INSPECTOR.md): a Fabric mod for debugging rendered UIs
- [Releasing](docs/RELEASING.md): publishing the plugin to the rpp registry

## Contributing

Tools are pinned with [mise](https://mise.jdx.dev):

```bash
mise trust && mise install
pnpm install --frozen-lockfile
just doctor          # check your setup
just                 # list tasks
just ready           # everything CI runs
```

Mise 2026.10.0 or newer is required; `mise install` also provides the pinned [rpp](https://github.com/chunkzero/rpp)
nightly used by the example pack, golden cases, and TypeScript checks. Commits and pull request titles follow
[Conventional Commits](https://www.conventionalcommits.org).

`just bench` times `rpp build` for the same UI written with JSX and with the function-style `raw` API
(`bench/authoring/`), after checking that both produce identical output. It accepts `--runs`, `--scale`,
`--scenarios cold,noop,edit`, and `--cold-wasm`, always uses its own `RPP_CACHE_DIR` under `build/bench/`, and honors
`RPP` to choose the rpp binary.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
