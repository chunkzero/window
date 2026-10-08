# Window

Custom Minecraft UIs for [Minestom](https://minestom.net)-based servers, with no client mods.

You describe a UI in TypeScript. Window compiles it into resource-pack fonts and shaders that vanilla clients draw
through the inventory title and HUD, and generates a typed Kotlin view for each window. You implement the view on the
server. If you rename a button in the UI, the server code stops compiling instead of breaking at runtime.

Inventory UIs support real items, scrolling collections, paging, toggles, choices, tooltips, and native anvil text
input. HUDs are positioned with core shaders and rendered by the server whenever it wants to update them.

> [!WARNING]
>
> Window is early alpha software (`0.1.0-alpha.0`). There are known bugs, and the TypeScript API, generated Kotlin, and
> runtime will change in breaking ways between releases. Pin a version and expect to update your UIs when you upgrade.

## How it works

Write a UI in your [rpp](https://github.com/chunkzero/rpp) resource-pack project, as JSX laid out automatically with
flexbox and slot grids:

```tsx
// src/window/confirm.tsx
import { Section, Text, Window, action, industrial, text } from "#plugins/window";

export default (
  <Window name="confirm" container="generic_9x3">
    <Section of="container">
      <Text bind={text("question")} span={9} />
      <industrial.Button onClick={action("accept")} at={[3, 1]} tooltip="Confirm">
        <Text bold>YES</Text>
      </industrial.Button>
    </Section>
  </Window>
);
```

Or with the function-style API under `raw`, which builds the same primitives and calls components as functions:

```ts
// src/window/confirm.ts
import { action, industrial, raw, text } from "#plugins/window";

export default raw.ui({
  name: "confirm",
  container: "generic_9x3",
  children: [
    raw.text(text("question"), { x: 8, y: 6, width: 160, align: "center" }),
    raw.section("container", {
      children: [
        industrial.Button({
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
import { defineConfig } from "#rpp/config";
import window from "#plugins/window";

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

## Supported versions

Window targets Minecraft 26.2 (resource pack format 88). The hosts and example server use Minestom builds for 26.2, and
Minestom serves a single protocol version, so they accept only 26.2 clients. The [inspector](docs/INSPECTOR.md) also
targets 26.2.

The compiler still emits packs for 26.1.x (pack format 84): its glyph metrics are shared with 26.2, and `hudShaders`
selects the core text shaders for the configured `pack.format`. Serving 26.1.x clients requires a server that speaks
their protocol; the Minestom and Multistom hosts do not.

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
