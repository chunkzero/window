use std::collections::BTreeMap;

use crate::ir::{Align, CLOSE_ACTION, Handle, HandleKind, HandleRole, HandleUse};
use crate::manifest::{
    AnvilInputEntry, CollectionEntry, FontMetricsEntry, HudEntry, HudSurfaceEntry, ItemEntry, Manifest, RegionEntry,
    SlotAreaEntry, SlotEntry, SlotRefEntry, SpriteEntry, SpriteSlotEntry, SurfaceEntry, SwitchCaseEntry, SwitchEntry,
    VERSION, WindowEntry,
};
use crate::pipeline::OutputFile;

use super::{KotlinTarget, generate_kotlin};

fn manifest(windows: BTreeMap<String, WindowEntry>, huds: BTreeMap<String, HudEntry>) -> Manifest {
    Manifest {
        version: VERSION,
        namespace: "window".into(),
        font: "window:ui".into(),
        spacers: BTreeMap::new(),
        text_advances: BTreeMap::new(),
        text_glyph_widths: BTreeMap::new(),
        font_metrics: BTreeMap::new(),
        sprites: BTreeMap::new(),
        windows,
        huds,
    }
}

fn window(container: &str, size: [u32; 2], title_origin: [i32; 2]) -> WindowEntry {
    WindowEntry {
        surface: SurfaceEntry { kind: "container".into(), container: container.into(), size, title_origin },
        static_text: String::new(),
        slots: BTreeMap::new(),
        sprite_slots: BTreeMap::new(),
        regions: BTreeMap::new(),
        items: BTreeMap::new(),
        collections: BTreeMap::new(),
        inputs: BTreeMap::new(),
        switches: BTreeMap::new(),
        layers: Vec::new(),
        handles: BTreeMap::new(),
    }
}

fn slot(x: i32, y: i32, width: u32, align: Align, font: &str, color: &str, text: Option<&str>) -> SlotEntry {
    SlotEntry {
        x,
        y,
        width,
        align,
        font: font.into(),
        color: color.into(),
        shader_marker: None,
        shader_color: None,
        shadow: false,
        bold: false,
        italic: false,
        underlined: false,
        strikethrough: false,
        obfuscated: false,
        text: text.map(Into::into),
        overflow: None,
        lines: None,
        source: None,
    }
}

fn container(index: u32) -> SlotRefEntry {
    SlotRefEntry { area: SlotAreaEntry::Container, index }
}

/// A region whose clicks name the action `action`.
fn button(action: &str, x: i32, y: i32, width: u32, height: u32, slots: Vec<SlotRefEntry>) -> RegionEntry {
    RegionEntry {
        x,
        y,
        width,
        height,
        slots,
        fill_slots: None,
        action: Some(action.into()),
        default_action: None,
        hitbox: None,
        source: None,
    }
}

/// A handle of `kind` used by `uses`, each a role and an entry name.
fn handle(kind: HandleKind, uses: &[(HandleRole, &str)]) -> Handle {
    Handle {
        kind,
        values: Vec::new(),
        shape: Vec::new(),
        initial: None,
        selectable: false,
        only: Vec::new(),
        uses: uses
            .iter()
            .map(|(role, entry)| HandleUse { role: *role, entry: entry.to_string(), at: Vec::new(), value: None })
            .collect(),
    }
}

fn file_contents<'a>(files: &'a [OutputFile], path: &str) -> &'a str {
    let file = files.iter().find(|file| file.path == path).unwrap_or_else(|| panic!("{path} should be generated"));
    std::str::from_utf8(file.contents.as_bytes()).unwrap()
}

fn shop_window() -> WindowEntry {
    let mut shop = window("generic_9x3", [176, 168], [8, 6]);
    shop.slots = BTreeMap::from([
        ("balance".into(), slot(8, 24, 160, Align::Center, "window:y18", "#ffffff", None)),
        ("label_0".into(), slot(8, 6, 160, Align::Center, "window:y0", "#ffd700", Some("Black Market"))),
    ]);
    shop.regions = BTreeMap::from([
        ("buy".into(), button("buy", 8, 54, 18, 18, vec![container(18)])),
        (
            "window:close".into(),
            RegionEntry {
                default_action: Some(CLOSE_ACTION.into()),
                ..button(CLOSE_ACTION, 98, 54, 18, 18, vec![container(5)])
            },
        ),
    ]);
    shop.items = BTreeMap::from([("egg_info".into(), ItemEntry { slots: vec![container(7)] })]);
    let selected_cell = SpriteSlotEntry {
        x: 7,
        y: 17,
        width: 18,
        height: 18,
        align: Align::Left,
        font: "window:sprite_y11".into(),
        sprite: Some("slot_selected".into()),
        source: None,
    };
    shop.collections = BTreeMap::from([(
        "entries".into(),
        CollectionEntry { slots: vec![container(0)], action: true, selection: vec![selected_cell] },
    )]);
    let mut entries = handle(HandleKind::Collection, &[(HandleRole::Collection, "entries")]);
    entries.selectable = true;
    shop.handles = BTreeMap::from([
        ("balance".into(), handle(HandleKind::Text, &[(HandleRole::Slot, "balance")])),
        ("buy".into(), handle(HandleKind::Action, &[(HandleRole::Click, "buy")])),
        ("egg_info".into(), handle(HandleKind::Items, &[(HandleRole::Item, "egg_info")])),
        ("entries".into(), entries),
        ("window:close".into(), handle(HandleKind::Builtin, &[(HandleRole::Click, "window:close")])),
    ]);
    shop
}

#[test]
fn generates_typed_view() {
    let manifest = manifest(BTreeMap::from([("shop".into(), shop_window())]), BTreeMap::new());

    let files = generate_kotlin(&manifest, "com.chunkzero.window.example.generated", KotlinTarget::Minestom).unwrap();
    let paths: Vec<&str> = files.iter().map(|file| file.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "ShopView.kt",
            "WindowDefinitions.kt",
            "WindowEntries.kt",
            "WindowFonts.kt",
            "WindowHudDefinitions.kt",
            "WindowHudEntries.kt",
            "WindowPackData.kt",
            "WindowSpacers.kt",
            "WindowSprites.kt",
        ]
    );
    let content = file_contents(&files, "ShopView.kt");
    assert!(content.contains("protected abstract fun balance(): Component"), "{content}");
    assert!(content.contains("protected abstract fun onBuy(click: Click)"), "{content}");
    assert!(content.contains("protected abstract fun eggInfo(): ItemStack?"), "{content}");
    assert!(content.contains("protected abstract val entries: WindowCollection<ItemStack>"), "{content}");
    assert!(content.contains("slot(\"balance\") { balance() }"), "{content}");
    assert!(content.contains("button(\"buy\", ::onBuy)"), "{content}");
    assert!(content.contains("item(\"egg_info\") { eggInfo() }"), "{content}");
    assert!(!content.contains("window:close") && !content.contains("onWindow"), "{content}");
    assert!(!content.contains("label_0(): Component"));
}

fn status_hud() -> HudEntry {
    HudEntry {
        surface: HudSurfaceEntry { kind: "hud".into(), channel: "actionbar".into(), width: 120, height: 16 },
        static_text: String::new(),
        slots: BTreeMap::from([("coins".into(), slot(0, 0, 80, Align::Left, "window:y0", "#ffffff", None))]),
        shader: None,
        switches: BTreeMap::new(),
        layers: Vec::new(),
        handles: BTreeMap::from([("coins".into(), handle(HandleKind::Text, &[(HandleRole::Slot, "coins")]))]),
    }
}

#[test]
fn views_follow_the_target_and_huds_do_not() {
    let manifest =
        manifest(BTreeMap::from([("shop".into(), shop_window())]), BTreeMap::from([("status".into(), status_hud())]));
    let player_view = |host: &str| {
        format!(
            "public abstract class ShopView(protected val player: Player) : \
             WindowView<ItemStack>(WindowDefinitions.shop, {host}(player)) {{"
        )
    };
    let cases = [
        (KotlinTarget::Minestom, "com.chunkzero.window.minestom.MinestomHost", "MinestomHost"),
        (KotlinTarget::Multistom, "com.chunkzero.window.multistom.MultistomHost", "MultistomHost"),
    ];
    let agnostic = generate_kotlin(&manifest, "golden", KotlinTarget::Agnostic).unwrap();
    let hud = file_contents(&agnostic, "StatusHud.kt");
    assert!(hud.contains("public abstract class StatusHud : HudView(WindowHudDefinitions.status) {"), "{hud}");
    assert!(hud.contains("final override fun HudScope.bind() {"));
    assert!(!hud.contains("Player") && !hud.contains("host") && !hud.contains("Host"), "{hud}");
    for (target, import, host) in cases {
        let files = generate_kotlin(&manifest, "golden", target).unwrap();
        let view = file_contents(&files, "ShopView.kt");
        assert!(view.contains(&player_view(host)), "{view}");
        assert!(view.contains(&format!("import {import}\n")));
        assert!(view.contains("import net.minestom.server.entity.Player\n"));
        assert!(view.contains("import net.minestom.server.item.ItemStack\n"));
        assert!(view.contains("final override fun WindowScope<ItemStack>.bind() {"));
        assert!(view.contains("protected abstract fun eggInfo(): ItemStack?"));
        assert_eq!(file_contents(&files, "StatusHud.kt"), hud);
    }

    let view = file_contents(&agnostic, "ShopView.kt");
    assert!(view.contains(
        "public abstract class ShopView<I : Any>(host: WindowHost<I>) : WindowView<I>(WindowDefinitions.shop, host) {"
    ));
    assert!(view.contains("final override fun WindowScope<I>.bind() {"));
    assert!(view.contains("protected abstract fun eggInfo(): I?"));
    assert!(view.contains("protected abstract val entries: WindowCollection<I>"));
    assert!(view.contains("import com.chunkzero.window.host.WindowHost\n"));
    assert!(!view.contains("minestom") && !view.contains("multistom"), "{view}");
}

#[test]
fn only_views_huds_and_definitions_are_public() {
    let manifest =
        manifest(BTreeMap::from([("shop".into(), shop_window())]), BTreeMap::from([("status".into(), status_hud())]));
    let files = generate_kotlin(&manifest, "golden", KotlinTarget::Agnostic).unwrap();
    for file in &files {
        let contents = std::str::from_utf8(file.contents.as_bytes()).unwrap();
        let public =
            ["View.kt", "Hud.kt", "Definitions.kt", "WindowSprite.kt"].iter().any(|suffix| file.path.ends_with(suffix));
        assert_eq!(contents.contains("public "), public, "{}", file.path);
    }
    let windows = file_contents(&files, "WindowDefinitions.kt");
    assert!(windows.contains("public object WindowDefinitions {"));
    assert!(
        windows.contains("public val shop: WindowDefinition = WindowDefinition(WindowPackData.manifest, \"shop\")")
    );
    let huds = file_contents(&files, "WindowHudDefinitions.kt");
    assert!(huds.contains("public object WindowHudDefinitions {"));
    assert!(huds.contains("public val status: HudDefinition = HudDefinition(WindowPackData.manifest, \"status\")"));
}

#[test]
fn colliding_definition_names_are_rejected() {
    let windows = BTreeMap::from([("a_b".into(), shop_window()), ("a__b".into(), shop_window())]);
    let err = generate_kotlin(&manifest(windows, BTreeMap::new()), "golden", KotlinTarget::Agnostic).unwrap_err();
    assert!(err.to_string().contains("both map to `WindowDefinitions.aB`"), "{err}");
}

#[test]
fn collection_selections_alone_import_sprite_slot_types() {
    let mut window = window("generic_9x3", [176, 166], [8, 6]);
    let selected = shop_window().collections.remove("entries").unwrap();
    window.collections = BTreeMap::from([("entries".into(), selected)]);
    let manifest = manifest(BTreeMap::from([("shop".into(), window)]), BTreeMap::new());

    let files = generate_kotlin(&manifest, "com.chunkzero.window.example.generated", KotlinTarget::Minestom).unwrap();
    let content = file_contents(&files, "WindowEntries.kt");
    assert!(content.contains("import com.chunkzero.window.manifest.SpriteSlotEntry"));
    assert!(content.contains("import com.chunkzero.window.manifest.Align"));
}

#[test]
fn switches_bind_typed_enums_and_booleans() {
    let switch = |values: &[&str]| SwitchEntry {
        cases: values.iter().map(|value| SwitchCaseEntry { value: value.to_string(), ..Default::default() }).collect(),
        ..Default::default()
    };
    let mut shop = window("generic_9x3", [176, 166], [8, 6]);
    shop.switches =
        BTreeMap::from([("mode".into(), switch(&["buy", "sell"])), ("on_sale".into(), switch(&["true", "false"]))]);
    let mut mode = handle(HandleKind::Value, &[(HandleRole::Switch, "mode")]);
    mode.values = vec!["buy".into(), "sell".into()];
    shop.handles = BTreeMap::from([
        ("mode".into(), mode),
        ("on_sale".into(), handle(HandleKind::Flag, &[(HandleRole::Switch, "on_sale")])),
    ]);
    let manifest = manifest(BTreeMap::from([("shop".into(), shop)]), BTreeMap::new());

    let files = generate_kotlin(&manifest, "com.chunkzero.window.generated", KotlinTarget::Minestom).unwrap();
    let view = file_contents(&files, "ShopView.kt");
    assert!(view.contains(
        "public enum class Mode(public val value: String) {\n        BUY(\"buy\"),\n        SELL(\"sell\"),"
    ));
    assert!(view.contains("protected abstract fun mode(): Mode"));
    assert!(view.contains("switch(\"mode\") { mode().value }"));
    assert!(view.contains("protected abstract fun onSale(): Boolean"));
    assert!(view.contains("switch(\"on_sale\") { onSale().toString() }"));
    let definitions = file_contents(&files, "WindowEntries.kt");
    assert!(definitions.contains("import com.chunkzero.window.manifest.SwitchCaseEntry"));
    assert!(definitions.contains("SwitchCaseEntry(\n"));
}

#[test]
fn sprite_slots_return_typed_runtime_sprites() {
    let mut shop = window("generic_9x3", [176, 166], [8, 6]);
    shop.sprite_slots = BTreeMap::from([(
        "badge".into(),
        SpriteSlotEntry {
            x: 0,
            y: 0,
            width: 8,
            height: 8,
            align: Align::Left,
            font: "window:sprite_y0".into(),
            sprite: None,
            source: None,
        },
    )]);
    shop.handles = BTreeMap::from([("badge".into(), handle(HandleKind::Sprite, &[(HandleRole::SpriteSlot, "badge")]))]);
    let mut manifest = manifest(BTreeMap::from([("shop".into(), shop)]), BTreeMap::new());
    let sprite = SpriteEntry { width: 8, height: 8, x_offset: 0, glyph_width: 8, advance: 9, glyph: "\u{e000}".into() };
    manifest.sprites = BTreeMap::from([("card_empty".into(), sprite)]);

    let files = generate_kotlin(&manifest, "golden", KotlinTarget::Agnostic).unwrap();
    let sprites = file_contents(&files, "WindowSprite.kt");
    assert!(
        sprites.contains("public enum class WindowSprite(public val id: String) {\n    CARD_EMPTY(\"card_empty\"),")
    );
    let view = file_contents(&files, "ShopView.kt");
    assert!(view.contains("protected abstract fun badgeSprite(): WindowSprite?"), "{view}");
    assert!(view.contains("sprite(\"badge\") { badgeSprite()?.id }"), "{view}");

    let mut shadowed = manifest.clone();
    let shop = shadowed.windows.get_mut("shop").unwrap();
    let case = |value: &str| SwitchCaseEntry { value: value.into(), ..Default::default() };
    shop.switches = BTreeMap::from([(
        "window_sprite".into(),
        SwitchEntry { cases: vec![case("a"), case("b")], ..Default::default() },
    )]);
    let mut value = handle(HandleKind::Value, &[(HandleRole::Switch, "window_sprite")]);
    value.values = vec!["a".into(), "b".into()];
    shop.handles.insert("window_sprite".into(), value);
    let err = generate_kotlin(&shadowed, "golden", KotlinTarget::Agnostic).unwrap_err();
    assert!(err.to_string().contains("reserved WindowView member `WindowSprite`"), "{err}");
}

#[test]
fn generates_typed_anvil_input_binding() {
    let mut search = window("anvil", [176, 166], [60, 6]);
    search.inputs = BTreeMap::from([(
        "query".into(),
        AnvilInputEntry { slot: container(0), initial: "Search...".into(), item_model: Some("demo:gui/search".into()) },
    )]);
    search.handles = BTreeMap::from([("query".into(), handle(HandleKind::Input, &[(HandleRole::Input, "query")]))]);
    let manifest = manifest(BTreeMap::from([("search".into(), search)]), BTreeMap::new());

    let files = generate_kotlin(&manifest, "com.chunkzero.window.generated", KotlinTarget::Minestom).unwrap();
    let content = file_contents(&files, "SearchView.kt");
    assert!(content.contains("protected abstract fun onQueryChanged(value: String)"));
    assert!(content.contains("anvilInput(\"query\", ::onQueryChanged)"));
}

#[test]
fn hud_lifecycle_members_are_reserved() {
    let hud = HudEntry {
        surface: HudSurfaceEntry { kind: "hud".into(), channel: "actionbar".into(), width: 120, height: 16 },
        static_text: String::new(),
        slots: BTreeMap::from([("on_show".into(), slot(0, 0, 80, Align::Left, "window:y0", "#ffffff", None))]),
        shader: None,
        switches: BTreeMap::new(),
        layers: Vec::new(),
        handles: BTreeMap::from([("on_show".into(), handle(HandleKind::Text, &[(HandleRole::Slot, "on_show")]))]),
    };
    let manifest = manifest(BTreeMap::new(), BTreeMap::from([("status".into(), hud)]));

    let err = generate_kotlin(&manifest, "com.chunkzero.window.generated", KotlinTarget::Minestom).unwrap_err();
    assert!(err.to_string().contains("reserved WindowView member `onShow`"), "{err}");
}

#[test]
fn hud_slots_cannot_shadow_hud_view_members() {
    for name in ["render", "channel"] {
        let hud = HudEntry {
            surface: HudSurfaceEntry { kind: "hud".into(), channel: "actionbar".into(), width: 120, height: 16 },
            static_text: String::new(),
            slots: BTreeMap::from([(name.into(), slot(0, 0, 80, Align::Left, "window:y0", "#ffffff", None))]),
            shader: None,
            switches: BTreeMap::new(),
            layers: Vec::new(),
            handles: BTreeMap::from([(name.into(), handle(HandleKind::Text, &[(HandleRole::Slot, name)]))]),
        };
        let manifest = manifest(BTreeMap::new(), BTreeMap::from([("status".into(), hud)]));

        let err = generate_kotlin(&manifest, "com.chunkzero.window.generated", KotlinTarget::Minestom).unwrap_err();
        assert!(err.to_string().contains(&format!("reserved WindowView member `{name}`")), "{err}");
    }
}

#[test]
fn fonts_with_identical_metrics_share_one_table() {
    let small_caps = FontMetricsEntry {
        advances: BTreeMap::from([('a', 6)]),
        glyph_widths: BTreeMap::from([('a', 5)]),
        bold_advance: 1,
    };
    let mut manifest = manifest(BTreeMap::new(), BTreeMap::new());
    manifest.font_metrics = BTreeMap::from([
        ("window:small_caps/y0".into(), small_caps.clone()),
        ("window:small_caps/y9".into(), small_caps),
    ]);

    let files = generate_kotlin(&manifest, "com.chunkzero.window.example.generated", KotlinTarget::Minestom).unwrap();
    let content = file_contents(&files, "WindowFonts.kt");
    assert_eq!(content.matches("private val fontMetrics").count(), 1);
    assert!(content.contains("\"window:small_caps/y0\" to fontMetrics0"));
    assert!(content.contains("\"window:small_caps/y9\" to fontMetrics0"));
}

#[test]
fn runtime_actions_keep_their_entry_action() {
    let manifest = manifest(BTreeMap::from([("shop".into(), shop_window())]), BTreeMap::new());
    let files = generate_kotlin(&manifest, "com.chunkzero.window.generated", KotlinTarget::Minestom).unwrap();
    let entries = file_contents(&files, "WindowEntries.kt");
    assert!(entries.contains("action = \"window:close\",\n"), "{entries}");
    assert!(entries.contains("defaultAction = \"window:close\",\n"), "{entries}");
}
