# Window

Custom Minecraft UIs for [Minestom](https://minestom.net) servers, with no client mods.

You describe a UI in TypeScript. Window compiles it into resource-pack fonts and shaders that vanilla clients draw
through the inventory title and HUD, and generates a typed Kotlin view for each window. You implement the view on the
server. If you rename a button in the UI, the server code stops compiling instead of breaking at runtime.

Inventory UIs support real items, scrolling collections, paging, toggles, choices, tooltips, and native anvil text
input. HUDs are positioned with core shaders and update live from the server.

> [!WARNING]
> Window is early alpha software (`0.1.0-alpha.0`). There are known bugs, and the TypeScript API, generated Kotlin, and
> runtime will change in breaking ways between releases. Pin a version and expect to update your UIs when you upgrade.

## How it works

Write a UI in your [rpp](https://github.com/chunkzero/rpp) resource-pack project:

```ts
// src/window/confirm.ts
import { button, label, slot, ui } from "#plugins/window";

export default ui({
  name: "confirm",
  container: "generic_9x3",
  children: [
    slot("question", { x: 8, y: 6, width: 160, align: "center" }),
    button("accept", {
      transform: { section: "container", x: 3, y: 1, width: 1, height: 1 },
      tooltip: "Confirm",
      children: [label("YES", { bold: true })],
    }),
  ],
});
```

`rpp build` produces the resource pack and a Kotlin base class, which you extend on the server:

```kotlin
class Confirm(private val question: String, private val onYes: () -> Unit) : ConfirmView() {
    override fun question(): Component = Component.text(question)

    override fun onAccept(click: Click) = onYes()
}

val windows = WindowPack.windows()
windows.open(player, Confirm("Buy this sword?") { buySword(player) })
```

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
  pack: { name: "my-pack", description: "My server pack", packFormat: 84 },
  plugins: [
    window({
      namespace: "window",
      kotlin: { package: "com.example.ui", output: "../server/src/main/kotlin/com/example/ui" },
    }),
  ],
});
```

The server uses the Kotlin runtime in [`jvm/runtime`](jvm/runtime) (`dev.oglass.window:runtime`).

The [example](example/README.md) is a complete project: a shop with a paged catalog, filters and anvil search, and a set
of shader HUDs, running on a Minestom server. To try it:

```bash
just example-pack    # build the resource pack and Kotlin bindings
just example-run     # start the server on :25565
```

Then connect with a Minecraft 26.1.2 client.

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

The example pack also needs [rpp](https://github.com/chunkzero/rpp) on your `PATH`. Commits and pull request titles
follow [Conventional Commits](https://www.conventionalcommits.org).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
