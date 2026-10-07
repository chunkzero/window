use std::collections::BTreeMap;

use crate::ir::{Align, ButtonDefault, IndexedBinding, IndexedKind};
use crate::manifest::{
    AnvilInputEntry, ButtonEntry, CollectionEntry, FontMetricsEntry, HudEntry, HudSurfaceEntry, ItemEntry, Manifest,
    RepeatGroupEntry, SlotAreaEntry, SlotEntry, SlotRefEntry, SpriteEntry, SpriteSlotEntry, SurfaceEntry,
    SwitchCaseEntry, SwitchEntry, VERSION, WindowEntry,
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
        colors: BTreeMap::new(),
    }
}

fn window(container: &str, size: [u32; 2], title_origin: [i32; 2]) -> WindowEntry {
    WindowEntry {
        surface: SurfaceEntry { kind: "container".into(), container: container.into(), size, title_origin },
        static_text: String::new(),
        slots: BTreeMap::new(),
        sprite_slots: BTreeMap::new(),
        buttons: BTreeMap::new(),
        items: BTreeMap::new(),
        collections: BTreeMap::new(),
        inputs: BTreeMap::new(),
        slot_rects: BTreeMap::new(),
        groups: BTreeMap::new(),
        switches: BTreeMap::new(),
        indexed: BTreeMap::new(),
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
        binding: None,
        overflow: None,
        lines: None,
    }
}

fn container(index: u32) -> SlotRefEntry {
    SlotRefEntry { area: SlotAreaEntry::Container, index }
}

fn button(x: i32, y: i32, width: u32, height: u32, slots: Vec<SlotRefEntry>) -> ButtonEntry {
    ButtonEntry {
        x,
        y,
        width,
        height,
        slots,
        fill_slots: None,
        default: None,
        action: true,
        tooltip: None,
        states: BTreeMap::new(),
        sprite_font: None,
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
        ("entry_price_0".into(), slot(8, 54, 40, Align::Center, "window:y48", "#ffffff", None)),
        ("entry_price_1".into(), slot(62, 54, 40, Align::Center, "window:y48", "#ffffff", None)),
    ]);
    shop.buttons = BTreeMap::from([
        (
            "entry_0".into(),
            ButtonEntry {
                fill_slots: Some(vec![container(19)]),
                ..button(8, 54, 52, 34, vec![container(18), container(19)])
            },
        ),
        (
            "entry_1".into(),
            ButtonEntry {
                fill_slots: Some(vec![container(22)]),
                ..button(62, 54, 52, 34, vec![container(21), container(22)])
            },
        ),
        (
            "exit".into(),
            ButtonEntry { default: Some(ButtonDefault::Close), ..button(98, 54, 18, 18, vec![container(5)]) },
        ),
    ]);
    shop.items = BTreeMap::from([
        ("egg_info".into(), ItemEntry { slots: vec![container(7)] }),
        ("entry_icon_0".into(), ItemEntry { slots: vec![container(18)] }),
        ("entry_icon_1".into(), ItemEntry { slots: vec![container(21)] }),
    ]);
    let selected_cell = SpriteSlotEntry {
        x: 7,
        y: 17,
        width: 18,
        height: 18,
        align: Align::Left,
        font: "window:sprite_y11".into(),
        sprite: Some("slot_selected".into()),
        binding: None,
    };
    shop.collections = BTreeMap::from([(
        "entries".into(),
        CollectionEntry { slots: vec![container(0)], action: true, selection: vec![selected_cell] },
    )]);
    shop.groups = BTreeMap::from([(
        "entry".into(),
        RepeatGroupEntry {
            count: 2,
            slots: BTreeMap::from([("price".into(), vec!["entry_price_0".into(), "entry_price_1".into()])]),
            sprite_slots: BTreeMap::new(),
            items: BTreeMap::from([("icon".into(), vec!["entry_icon_0".into(), "entry_icon_1".into()])]),
            buttons: vec!["entry_0".into(), "entry_1".into()],
        },
    )]);
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
    assert!(content.contains("protected abstract fun balance(): Component"));
    assert!(content.contains("protected open fun onExit(click: Click): Unit = close()"));
    assert!(content.contains("protected abstract fun eggInfoItem(): ItemStack?"));
    assert!(content.contains("protected abstract fun entriesItem(index: Int): ItemStack?"));
    assert!(content.contains("protected abstract fun onEntries(click: IndexedClick)"));
    assert!(content.contains("protected abstract fun entryPrice(index: Int): Component"));
    assert!(content.contains("protected abstract fun onEntry(click: IndexedClick)"));
    assert!(content.contains("slot(\"balance\") { balance() }"));
    assert!(content.contains("for (index in 0 until 2) {"));
    assert!(content.contains("slot(\"entry_price_$index\") { entryPrice(index) }"));
    assert!(content.contains("button(\"entry_$index\") { click ->"));
    assert!(content.contains("IndexedClick(click.slot, index, click.shift, click.right)"));
    assert!(content.contains("item(\"egg_info\") { eggInfoItem() }"));
    // A repeater cell item is exposed as one grouped indexed member, not as
    // per-cell members, and the flattened per-cell names are bound for it.
    assert!(content.contains("protected abstract fun entryIconItem(index: Int): ItemStack?"));
    assert!(content.contains("item(\"entry_icon_$index\") { entryIconItem(index) }"));
    assert!(!content.contains("entryIcon0"));
    assert!(content.contains("collection(\"entries\", ::entriesItem, ::onEntries)"));
    assert!(content.contains("protected open fun entriesSelected(): Int? = null"));
    assert!(content.contains("collectionSelection(\"entries\", ::entriesSelected)"));
    assert!(!content.contains("entryPrice0"));
    assert!(!content.contains("label_0(): Component"));
}

fn status_hud() -> HudEntry {
    HudEntry {
        surface: HudSurfaceEntry { kind: "hud".into(), channel: "actionbar".into(), width: 120, height: 16 },
        static_text: String::new(),
        slots: BTreeMap::from([("coins".into(), slot(0, 0, 80, Align::Left, "window:y0", "#ffffff", None))]),
        shader: None,
        switches: BTreeMap::new(),
        indexed: BTreeMap::new(),
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
        assert!(view.contains("protected abstract fun eggInfoItem(): ItemStack?"));
        assert_eq!(file_contents(&files, "StatusHud.kt"), hud);
    }

    let view = file_contents(&agnostic, "ShopView.kt");
    assert!(view.contains(
        "public abstract class ShopView<I : Any>(host: WindowHost<I>) : WindowView<I>(WindowDefinitions.shop, host) {"
    ));
    assert!(view.contains("final override fun WindowScope<I>.bind() {"));
    assert!(view.contains("protected abstract fun eggInfoItem(): I?"));
    assert!(view.contains("protected abstract fun entriesItem(index: Int): I?"));
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
        let public = ["View.kt", "Hud.kt", "Definitions.kt", "WindowColors.kt", "WindowSprite.kt"]
            .iter()
            .any(|suffix| file.path.ends_with(suffix));
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
fn theme_palette_generates_window_colors() {
    let empty = generate_kotlin(&manifest(BTreeMap::new(), BTreeMap::new()), "golden", KotlinTarget::Agnostic).unwrap();
    assert!(empty.iter().all(|file| file.path != "WindowColors.kt"));

    let mut palette = manifest(BTreeMap::new(), BTreeMap::new());
    palette.colors = BTreeMap::from([("muted".into(), "#a9d9b5".into()), ("accent_gold".into(), "#ffd75e".into())]);
    let files = generate_kotlin(&palette, "golden", KotlinTarget::Agnostic).unwrap();
    assert_eq!(
        file_contents(&files, "WindowColors.kt"),
        "// Generated by window-codegen — do not edit.
package golden

import net.kyori.adventure.text.format.TextColor

/** Theme palette colors of this Window pack. */
public object WindowColors {
    public val accentGold: TextColor = TextColor.color(0xffd75e)
    public val muted: TextColor = TextColor.color(0xa9d9b5)
}
"
    );

    palette.colors = BTreeMap::from([("a_b".into(), "#000000".into()), ("a__b".into(), "#ffffff".into())]);
    let err = generate_kotlin(&palette, "golden", KotlinTarget::Agnostic).unwrap_err();
    assert!(err.to_string().contains("both map to `WindowColors.aB`"), "{err}");
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
        cases: values
            .iter()
            .map(|value| SwitchCaseEntry {
                value: value.to_string(),
                static_text: String::new(),
                slots: vec![],
                sprite_slots: vec![],
            })
            .collect(),
    };
    let mut shop = window("generic_9x3", [176, 166], [8, 6]);
    shop.switches =
        BTreeMap::from([("mode".into(), switch(&["buy", "sell"])), ("on_sale".into(), switch(&["true", "false"]))]);
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
fn indexed_families_bind_one_member_in_loops() {
    let case = |value: &str| SwitchCaseEntry {
        value: value.into(),
        static_text: String::new(),
        slots: vec![],
        sprite_slots: vec![],
    };
    let mut hud = status_hud();
    for i in 0..3 {
        hud.slots.insert(format!("power[{i}]"), slot(0, 0, 9, Align::Left, "window:y0", "#ffffff", None));
    }
    for (i, j) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
        hud.switches.insert(format!("cell[{i}][{j}]"), SwitchEntry { cases: vec![case("off"), case("on")] });
    }
    hud.indexed = BTreeMap::from([
        ("power".into(), IndexedBinding { kind: IndexedKind::Slot, shape: vec![3] }),
        ("cell".into(), IndexedBinding { kind: IndexedKind::Switch, shape: vec![2, 2] }),
    ]);
    let manifest = manifest(BTreeMap::new(), BTreeMap::from([("status".into(), hud)]));

    let files = generate_kotlin(&manifest, "golden", KotlinTarget::Agnostic).unwrap();
    let view = file_contents(&files, "StatusHud.kt");
    assert!(view.contains("protected abstract fun power(index: Int): Component"), "{view}");
    assert!(
        view.contains("for (index in 0 until 3) {\n            slot(\"power[$index]\") { power(index) }"),
        "{view}"
    );
    assert!(view.contains("public enum class Cell(public val value: String) {"), "{view}");
    assert!(view.contains("protected abstract fun cell(row: Int, column: Int): Cell"), "{view}");
    assert!(view.contains("switch(\"cell[$row][$column]\") { cell(row, column).value }"), "{view}");
    assert!(!view.contains("power0") && !view.contains("cell00"), "{view}");
}

#[test]
fn bindings_shared_across_switch_cases_bind_one_member() {
    let mut hud = status_hud();
    for case in ["good", "bad"] {
        let copy =
            SlotEntry { binding: Some("status".into()), ..slot(0, 0, 80, Align::Left, "window:y0", "#ffffff", None) };
        hud.slots.insert(format!("status.{case}"), copy);
    }
    let manifest = manifest(BTreeMap::new(), BTreeMap::from([("status".into(), hud)]));

    let files = generate_kotlin(&manifest, "golden", KotlinTarget::Agnostic).unwrap();
    let view = file_contents(&files, "StatusHud.kt");
    assert_eq!(view.matches("protected abstract fun status(): Component").count(), 1, "{view}");
    assert_eq!(view.matches("slot(\"status\") { status() }").count(), 1, "{view}");
    assert!(file_contents(&files, "WindowHudEntries.kt").contains("binding = \"status\","));
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
            binding: None,
        },
    )]);
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
    let case = |value: &str| SwitchCaseEntry {
        value: value.into(),
        static_text: String::new(),
        slots: vec![],
        sprite_slots: vec![],
    };
    shop.switches = BTreeMap::from([("window_sprite".into(), SwitchEntry { cases: vec![case("a"), case("b")] })]);
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
        indexed: BTreeMap::new(),
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
            indexed: BTreeMap::new(),
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
