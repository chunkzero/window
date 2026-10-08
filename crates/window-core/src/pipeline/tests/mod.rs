use std::collections::BTreeMap;

use super::*;
use crate::geometry::{Insets, Rect};
use crate::ir::{Align, CLOSE_ACTION, Draw, Hitbox, RegionIr, Rgb, SlotIr, TextureKey, Tooltip};
use crate::surface::{ContainerKind, Surface};

mod huds;
mod sprites;
mod windows;

fn compile_windows(
    windows: &[LaidOutWindow],
    textures: &BTreeMap<String, Texture>,
    namespace: &str,
) -> Result<CompileOutput> {
    let fonts = text_font::resolve(&BTreeMap::new(), &BTreeMap::new()).unwrap();
    let assets = Assets { textures, runtime_sprites: &BTreeMap::new(), text_fonts: &fonts };
    compile_layouts(
        windows,
        &[],
        &assets,
        &BTreeMap::new(),
        namespace,
        &PackTarget::default(),
        &BuildOptions::default(),
    )
}

/// A solid RGBA texture for tests.
fn solid(width: u32, height: u32, rgba: [u8; 4]) -> Texture {
    Texture { width, height, rgba: rgba.iter().copied().cycle().take((width * height * 4) as usize).collect() }
}

fn with_transparent_right_edge(mut texture: Texture, columns: u32) -> Texture {
    for y in 0..texture.height {
        for x in texture.width.saturating_sub(columns)..texture.width {
            let i = ((y * texture.width + x) * 4 + 3) as usize;
            texture.rgba[i] = 0;
        }
    }
    texture
}

/// A window with no controls on `kind`.
fn bare_window(name: &str, kind: ContainerKind, draws: Vec<Draw>) -> LaidOutWindow {
    LaidOutWindow {
        name: name.into(),
        surface: Surface::Container(kind),
        draws,
        slots: vec![],
        sprite_slots: vec![],
        regions: vec![],
        items: vec![],
        collections: vec![],
        inputs: vec![],
        switches: vec![],
        layers: vec![],
        indexed: BTreeMap::new(),
        handles: BTreeMap::new(),
        warnings: vec![],
    }
}

/// A plain text slot with no styling.
fn text_slot(name: &str, text: Option<&str>, rect: Rect, align: Align, color: Rgb) -> SlotIr {
    SlotIr {
        name: name.into(),
        text: text.map(Into::into),
        rect,
        align,
        color,
        shadow: false,
        bold: false,
        italic: false,
        underlined: false,
        strikethrough: false,
        obfuscated: false,
        font: None,
        repeat: None,
        binding: None,
        fit: Default::default(),
        source: None,
    }
}

/// A window with one nine-slice draw, one label slot, one dynamic slot, and
/// one button over container slots.
fn sample_window() -> (LaidOutWindow, BTreeMap<String, Texture>) {
    let mut textures = BTreeMap::new();
    textures.insert("frame.png".to_string(), solid(8, 8, [200, 200, 200, 255]));

    let mut window = bare_window(
        "shop",
        ContainerKind::Generic9x6,
        vec![Draw::NineSlice {
            texture: TextureKey("frame.png".into()),
            insets: Insets::uniform(2),
            dest: Rect::new(0, 0, 176, 16),
        }],
    );
    window.slots = vec![
        text_slot("title", None, Rect::new(8, 6, 160, 8), Align::Center, Rgb::DEFAULT_TEXT),
        text_slot("buy_label", Some("Buy"), Rect::new(58, 30, 60, 8), Align::Center, Rgb { r: 0xff, g: 0xff, b: 0xff }),
    ];
    // Over container slots: rect at (8,18) covers slot 0.
    window.regions = vec![RegionIr {
        action: Some("buy".into()),
        default_action: Some(CLOSE_ACTION.into()),
        hitbox: Some(Hitbox {
            item_model: None,
            tooltip: Some(Tooltip { title: "Buy".into(), lines: vec!["Spend coins".into()] }),
        }),
        ..region("buy", Rect::new(8, 18, 16, 16), None)
    }];
    window.warnings = vec!["overlay overflow".into()];
    (window, textures)
}

/// A region over `rect`, or over explicit `slots`, with no action or hitbox.
fn region(name: &str, rect: Rect, slots: Option<Vec<crate::inventory::InventorySlotRef>>) -> RegionIr {
    RegionIr {
        name: name.into(),
        rect,
        slots,
        yielded_slots: Vec::new(),
        unowned: false,
        action: None,
        default_action: None,
        hitbox: None,
        repeat: None,
        source: format!("button `{name}`"),
    }
}

fn find<'a>(out: &'a CompileOutput, path: &str) -> &'a OutputFile {
    out.files.iter().find(|f| f.path == path).unwrap_or_else(|| panic!("missing output file {path}"))
}
