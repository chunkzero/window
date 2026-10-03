# Window example

A complete walk through the toolchain: author a UI in TypeScript, build it into a resource pack with the Window rpp
plugin, emit typed Kotlin pack classes, and render it on a Minestom server.

```
example/
├── pack/                       # rpp resource-pack project (built by `rpp build`)
│   ├── rpp.json                # references the Window plugin at ../../plugin
│   ├── rpp.config.ts           # pack, build, and Window plugin configuration
│   ├── src/
│   │   ├── pack.mcmeta
│   │   └── window/             # UI sources — stripped from the built pack
│   │       ├── theme.ts        # generated industrial theme preset
│   │       ├── shop.ts         # paged catalog, toggles, choices, and buttons
│   │       ├── search.ts       # linked native-anvil catalog search
│   │       ├── status_hud.ts   # the shader-relocated actionbar HUD
│   └── dist/                   # build output: fonts, glyph atlas, shaders, zip
└── (server lives in ../jvm/example)
```

The Minestom server is the Gradle module `jvm/example`: `Main.kt` boots the server, `Market.kt` is a throwaway domain
model, and `MyShop.kt` implements the **generated** `ShopView`. `MyStatusHud.kt` implements the **generated** HUD views,
and `Main.kt` loads the generated `WindowPack` object (under `generated/`, produced by the rpp plugin — committed, do
not hand-edit).

## Run it

From the repo root, with `rpp` on your PATH (or pass `rpp=/path/to/rpp`):

```bash
just example-pack      # rpp codegen + build -> dist/ and generated Kotlin views
just example-run       # boot the Minestom server on :25565
```

Then connect a 26.2 client. The catalog opens on spawn with 30 real item entries, category and sort choices,
favorites/affordability toggles, paging, reactive selection and wallet text, plus buy/search/exit actions. Search opens
a static anvil screen with the native rename field and back and search buttons; searching returns to the catalog
filtered by the query, which the catalog's clear button resets and reopening search restores.

The actionbar-backed HUD demo is disabled by default so the inventory examples stay visually focused. Enable it with
`-Dwindow.hud.enabled=true`; use `-Dwindow.hud.spriteDebug=true` or `-Dwindow.hud.flowDebug=true` for the chat
diagnostics.

Override the port with `-Dwindow.port=…`. The server also serves `dist/window-example.zip` as a required resource pack
at `http://127.0.0.1:25567/pack.zip`; override with `-Dwindow.pack=…`, `-Dwindow.pack.port=…`, or `-Dwindow.pack.url=…`
when testing from another machine.

## The typed contract

The rpp plugin emits `WindowPack`, `WindowFonts`, `WindowSpacers`, `WindowDefinitions`, `WindowHudDefinitions`, and one
abstract class for each window/HUD. `MyShop` must implement `balance()`, `status()`, and `onBuy(...)`; `MyStatusHud`
must implement each authored HUD slot. Omit one, rebuild, and the breakage surfaces at compile time.
