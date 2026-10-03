use std::collections::BTreeMap;

use crate::ir::{Align, ButtonDefault};
use crate::manifest::{
    AnvilInputEntry, ButtonEntry, CollectionEntry, FontMetricsEntry, HudEntry, HudSurfaceEntry, ItemEntry, Manifest,
    RepeatGroupEntry, SlotAreaEntry, SlotEntry, SlotRefEntry, SpriteSlotEntry, SurfaceEntry, VERSION, WindowEntry,
};
use crate::pipeline::OutputFile;

use super::generate_kotlin;

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
        buttons: BTreeMap::new(),
        items: BTreeMap::new(),
        collections: BTreeMap::new(),
        inputs: BTreeMap::new(),
        slot_rects: BTreeMap::new(),
        groups: BTreeMap::new(),
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
    std::str::from_utf8(&file.contents).unwrap()
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

    let files = generate_kotlin(&manifest, "dev.oglass.window.example.generated").unwrap();
    assert_eq!(files.len(), 7);
    assert!(files.iter().any(|file| file.path == "WindowPack.kt"));
    assert!(files.iter().any(|file| file.path == "WindowFonts.kt"));
    assert!(files.iter().any(|file| file.path == "WindowSprites.kt"));
    assert!(files.iter().any(|file| file.path == "WindowSpacers.kt"));
    assert!(files.iter().any(|file| file.path == "WindowDefinitions.kt"));
    assert!(files.iter().any(|file| file.path == "WindowHudDefinitions.kt"));
    let content = file_contents(&files, "ShopView.kt");
    assert!(content.contains("protected abstract fun balance(): Component"));
    assert!(content.contains("protected open fun onExit(click: Click): Unit = close()"));
    assert!(content.contains("protected abstract fun eggInfoItem(): ItemStack?"));
    assert!(content.contains("protected abstract fun entriesItem(index: Int): ItemStack?"));
    assert!(content.contains("protected abstract fun onEntries(click: IndexedClick)"));
    assert!(content.contains("protected abstract fun entryPrice(index: Int): Component"));
    assert!(content.contains("protected abstract fun onEntry(click: IndexedClick)"));
    assert!(content.contains("slot(\"balance\") { balance() }"));
    assert!(content.contains("slot(\"entry_price_0\") { entryPrice(0) }"));
    assert!(content.contains("button(\"entry_0\") { click ->"));
    assert!(content.contains("IndexedClick(click.player, click.slot, 0, click.shift, click.right)"));
    assert!(content.contains("item(\"egg_info\") { eggInfoItem() }"));
    // A repeater cell item is exposed as one grouped indexed member, not as
    // per-cell members, and the flattened per-cell names are bound for it.
    assert!(content.contains("protected abstract fun entryIconItem(index: Int): ItemStack?"));
    assert!(content.contains("item(\"entry_icon_0\") { entryIconItem(0) }"));
    assert!(content.contains("item(\"entry_icon_1\") { entryIconItem(1) }"));
    assert!(!content.contains("entryIcon0"));
    assert!(content.contains("collection(\"entries\", ::entriesItem, ::onEntries)"));
    assert!(content.contains("protected open fun entriesSelected(): Int? = null"));
    assert!(content.contains("collectionSelection(\"entries\", ::entriesSelected)"));
    assert!(!content.contains("entryPrice0"));
    assert!(!content.contains("label_0(): Component"));
}

#[test]
fn collection_selections_alone_import_sprite_slot_types() {
    let mut window = window("generic_9x3", [176, 166], [8, 6]);
    let selected = shop_window().collections.remove("entries").unwrap();
    window.collections = BTreeMap::from([("entries".into(), selected)]);
    let manifest = manifest(BTreeMap::from([("shop".into(), window)]), BTreeMap::new());

    let files = generate_kotlin(&manifest, "dev.oglass.window.example.generated").unwrap();
    let content = file_contents(&files, "WindowDefinitions.kt");
    assert!(content.contains("import dev.oglass.window.manifest.SpriteSlotEntry"));
    assert!(content.contains("import dev.oglass.window.manifest.Align"));
}

#[test]
fn generates_typed_anvil_input_binding() {
    let mut search = window("anvil", [176, 166], [60, 6]);
    search.inputs = BTreeMap::from([(
        "query".into(),
        AnvilInputEntry { slot: container(0), initial: "Search...".into(), item_model: Some("demo:gui/search".into()) },
    )]);
    let manifest = manifest(BTreeMap::from([("search".into(), search)]), BTreeMap::new());

    let files = generate_kotlin(&manifest, "dev.oglass.window.generated").unwrap();
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
    };
    let manifest = manifest(BTreeMap::new(), BTreeMap::from([("status".into(), hud)]));

    let err = generate_kotlin(&manifest, "dev.oglass.window.generated").unwrap_err();
    assert!(err.to_string().contains("reserved WindowView member `onShow`"), "{err}");
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

    let files = generate_kotlin(&manifest, "dev.oglass.window.example.generated").unwrap();
    let content = file_contents(&files, "WindowFonts.kt");
    assert_eq!(content.matches("private val fontMetrics").count(), 1);
    assert!(content.contains("\"window:small_caps/y0\" to fontMetrics0"));
    assert!(content.contains("\"window:small_caps/y9\" to fontMetrics0"));
}
