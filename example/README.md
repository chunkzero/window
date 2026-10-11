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
│   │       ├── index.ts        # defineWindows entry: the sprite catalog, windows, HUDs, and previews
│   │       ├── handles.ts      # typed bindings shared by the windows
│   │       ├── art.ts          # inline art: icons, coin, vents, bars, and the rank sprites
│   │       ├── shop.tsx        # paged catalog, selections, toggles, and buttons
│   │       ├── search.tsx      # linked native-anvil catalog search
│   │       ├── upgrades.tsx    # themed workshop: styles, vars, a repeater of cards, choices, and a hotspot
│   │       ├── status_hud.tsx  # the shader-relocated actionbar HUDs
│   │       └── clock_hud.ts    # the clock HUD, written with the function-style raw API
│   ├── build/previews/         # preview PNGs of the windows and HUDs, rewritten by every build
│   └── dist/                   # build output: fonts, glyph atlas, shaders, zip
└── (server lives in ../jvm/example)
```

The Minestom server is the Gradle module `jvm/example`, which depends on `window-minestom`. `Main.kt` boots the server,
`Market.kt` and `Workshop.kt` are a throwaway domain model, and `MyShop.kt` and `MyUpgrades.kt` implement the
**generated** `ShopView` and `UpgradesView`. `MyStatusHud.kt` implements the **generated** HUD views. The pack uses
`target: "minestom"`, so a window view takes the `Player` it is shown to: `Main.kt` opens the shop with
`MyShop(player, market).open()`. Generated HUD views take no player; the handwritten ones that need it take it
themselves. `Main.kt` joins the status HUDs into one action-bar `HudStack` and resends it with `player.sendHud(stack)`
every second, which re-renders their clocks and stays ahead of the action bar fading out. The bindings live under
`generated/`, produced by the rpp plugin — committed, do not hand-edit.

## Run it

From the repo root, with `rpp` on your PATH (or pass `rpp=/path/to/rpp`):

```bash
just example-pack      # rpp codegen + build -> dist/ and generated Kotlin views
just example-run       # boot the Minestom server on :25565
```

Then connect a 26.2 client. The catalog opens on spawn with 30 real item entries, category and sort choices,
favorites/affordability toggles, paging, reactive selection and wallet text, plus buy/search/workshop/exit actions.
Search opens a static anvil screen with the native rename field and back and search buttons; searching returns to the
catalog filtered by the query, which the catalog's clear button resets and reopening search restores.

The workshop, under its own theme, sells three upgrades for the same coins: a click on a card buys the chosen amount of
levels, each card shows its level as text and is marked new, shows a bar, or is marked maxed, and the rank badge takes
the shared tone. Its levels last until the server stops.

The actionbar-backed HUD demo is disabled by default so the inventory examples stay visually focused. Enable it with
`-Dwindow.hud.enabled=true`; use `-Dwindow.hud.spriteDebug=true` to print each HUD's static art to chat. The pack blanks
vanilla's experience bar, and one of the HUDs draws its wave bar there.

| command                          | does                                                                   |
| -------------------------------- | ---------------------------------------------------------------------- |
| `/shop`                          | opens the catalog, like the hotbar emerald                             |
| `/upgrades`                      | opens the workshop, like the catalog's Workshop button                 |
| `/tone <ok\|warn\|danger\|auto>` | pins the tone the HUDs and newly opened windows draw in; `auto` unpins |
| `/bar <fraction\|auto>`          | pins how full the wave bar and a newly opened shop's bar are           |

`rpp build` prints where the compiler's output bytes go and fails above the `size.budget` in `rpp.config.ts`.

Override the port with `-Dwindow.port=…`. The server also serves `dist/window-example.zip` as a required resource pack
at `http://127.0.0.1:25567/pack.zip`; override with `-Dwindow.pack=…`, `-Dwindow.pack.port=…`, or `-Dwindow.pack.url=…`
when testing from another machine.

## The typed contract

The rpp plugin emits `internal` pack data (`WindowPackData`, `WindowFonts`, `WindowSpacers`, and friends), the public
`WindowDefinitions` and `WindowHudDefinitions`, and one public abstract class for each window/HUD. Each handle in
`handles.ts` becomes a member of the views that use it: `text` handles such as `balance` and `status` are functions
returning a `Component`, `flag` handles such as `canBuy` are Boolean functions, `action` handles such as `buy` are
`onBuy(click)` handlers, `products` is a `WindowCollection` that `MyShop` backs with a `WindowList`, and the
`category`/`sort` selections and `favorites`/`affordable` toggles are UI-owned `var`s with `on…Changed` hooks. In
`UpgradesView` the indexed handles take the card's index (`upgradeLevel(index)`, `onUpgrade(click)` with `click.index`),
the `upgradeStage` value and the `rank` sprite return generated enums, and `upgradeProgress` and `tone` return a bar's
fraction and a tint's palette entry. `MyStatusHud` implements each HUD's handles. Omit one, rebuild, and the breakage
surfaces at compile time.
