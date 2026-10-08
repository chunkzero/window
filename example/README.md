# Window example

A complete walk through the toolchain: author a UI in TypeScript, build it into a resource pack with the Window rpp
plugin, emit typed Kotlin pack classes, and render it on a Minestom server.

```
example/
├── pack/                       # rpp resource-pack project (built by `rpp build`)
│   ├── rpp.json                # references the Window plugin at ../../plugin
│   ├── rpp.config.ts           # pack metadata, build, and Window plugin configuration
│   ├── src/
│   │   └── window/             # UI sources — stripped from the built pack
│   │       ├── index.ts        # defineWindows entry listing the sprite catalog, windows, and HUDs
│   │       ├── handles.ts      # typed bindings shared by the shop and search windows
│   │       ├── art.ts          # inline art: icons, coin, vents, and the search field sprite
│   │       ├── shop.tsx        # paged catalog, selections, toggles, and buttons
│   │       ├── search.tsx      # linked native-anvil catalog search
│   │       ├── status_hud.tsx  # the shader-relocated actionbar HUDs
│   └── dist/                   # build output: fonts, glyph atlas, shaders, zip
└── (server lives in ../jvm/example)
```

The Minestom server is the Gradle module `jvm/example`, which depends on `window-minestom`. `Main.kt` boots the server,
`Market.kt` is a throwaway domain model, and `MyShop.kt` implements the **generated** `ShopView`. `MyStatusHud.kt`
implements the **generated** HUD views. The pack uses `target: "minestom"`, so the shop view takes the `Player` it is
shown to: `Main.kt` opens it with `MyShop(player, market).open()`. Generated HUD views take no player; the handwritten
ones that need it take it themselves. `Main.kt` joins the status HUDs into one action-bar `HudStack` and resends it with
`player.sendHud(stack)` every second, which re-renders their clocks and stays ahead of the action bar fading out. The
bindings live under `generated/`, produced by the rpp plugin — committed, do not hand-edit.

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
`-Dwindow.hud.enabled=true`; use `-Dwindow.hud.spriteDebug=true` to print each HUD's static art to chat.

Override the port with `-Dwindow.port=…`. The server also serves `dist/window-example.zip` as a required resource pack
at `http://127.0.0.1:25567/pack.zip`; override with `-Dwindow.pack=…`, `-Dwindow.pack.port=…`, or `-Dwindow.pack.url=…`
when testing from another machine.

## The typed contract

The rpp plugin emits `internal` pack data (`WindowPackData`, `WindowFonts`, `WindowSpacers`, and friends), the public
`WindowDefinitions` and `WindowHudDefinitions`, and one public abstract class for each window/HUD. Each handle in
`handles.ts` becomes a member of `ShopView`: `text` handles such as `balance` and `status` are functions returning a
`Component`, `flag` handles such as `canBuy` are Boolean functions, `action` handles such as `buy` are `onBuy(click)`
handlers, `products` is a `WindowCollection` that `MyShop` backs with a `WindowList`, and the `category`/`sort`
selections and `favorites`/`affordable` toggles are UI-owned `var`s with `on…Changed` hooks. `MyStatusHud` implements
each HUD's `text` handles. Omit one, rebuild, and the breakage surfaces at compile time.
