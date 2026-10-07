use super::*;
use crate::inventory::InventorySlotRef;
use crate::ir::{CollectionIr, SpriteSlotIr};
use crate::model::SpriteDef;
use crate::pipeline::metrics::{GlyphMetrics, bitmap_metrics};
use crate::pipeline::sprites::runtime_sprite_assets;

#[test]
fn runtime_sprite_metrics_match_minecraft_alpha_trimmed_width() {
    let full = solid(12, 12, [255, 255, 255, 255]);
    let trimmed = with_transparent_right_edge(full.clone(), 2);

    assert_eq!(bitmap_metrics(&full, Size::new(12, 12)), GlyphMetrics { x_offset: 0, glyph_width: 12, advance: 13 });
    assert_eq!(bitmap_metrics(&trimmed, Size::new(12, 12)), GlyphMetrics { x_offset: 0, glyph_width: 10, advance: 11 });
}

#[test]
fn runtime_sprite_advance_rounds_scaled_trimmed_width_like_minecraft() {
    let texture = with_transparent_right_edge(solid(10, 10, [255, 255, 255, 255]), 1);

    assert_eq!(bitmap_metrics(&texture, Size::new(10, 8)).advance, 8);
}

#[test]
fn runtime_sprite_metrics_keep_left_padding_separate_from_visible_width() {
    let mut texture = with_transparent_right_edge(solid(9, 9, [255, 255, 255, 255]), 1);
    for y in 0..texture.height {
        let i = (y * texture.width * 4 + 3) as usize;
        texture.rgba[i] = 0;
    }

    assert_eq!(bitmap_metrics(&texture, Size::new(9, 9)), GlyphMetrics { x_offset: 1, glyph_width: 7, advance: 9 });
}

#[test]
fn resource_runtime_sprite_uses_decoded_pack_texture_metrics_when_available() {
    let mut texture = solid(32, 32, [0, 0, 0, 0]);
    for y in 5..=28 {
        for x in 7..=25 {
            let i = ((y * texture.width + x) * 4) as usize;
            texture.rgba[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let mut textures = BTreeMap::new();
    textures.insert("assets/example/textures/backpack/galactic_bucket.png".into(), texture);
    let mut project = ParsedProject::default();
    project.theme.sprites.insert(
        "backpack_galactic_bucket".into(),
        SpriteDef::Texture { texture: "example:backpack/galactic_bucket.png".into(), size: Some(Size::new(18, 18)) },
    );

    let sprites = runtime_sprite_assets(&project, &textures, "window").unwrap();
    let sprite = sprites.get("backpack_galactic_bucket").unwrap();

    assert_eq!(sprite.file, "example:backpack/galactic_bucket.png");
    assert_eq!(sprite.output, None);
    assert_eq!(sprite.x_offset, 4);
    assert_eq!(sprite.glyph_width, 11);
    assert_eq!(sprite.advance, 16);
}

fn empty_sprite_slot(name: &str, rect: Rect) -> SpriteSlotIr {
    SpriteSlotIr { name: name.into(), rect, align: Align::Center, sprite: None, repeat: None, binding: None }
}

fn pickaxe_sprite() -> BTreeMap<String, RuntimeSpriteAsset> {
    let asset = RuntimeSpriteAsset {
        size: Size::new(16, 16),
        x_offset: 0,
        glyph_width: 16,
        advance: 17,
        file: "window:font/sprites/pickaxe.png".into(),
        output: Some(OutputFile::binary(
            "assets/window/textures/font/sprites/pickaxe.png",
            solid(16, 16, [255, 128, 0, 255]).encode_png().unwrap(),
        )),
    };
    BTreeMap::from([("pickaxe".into(), asset)])
}

#[test]
fn runtime_sprites_are_duplicated_by_y_font_not_texture() {
    let (mut w, textures) = sample_window();
    w.buttons[0].states.get_mut("disabled").unwrap().sprite = Some("pickaxe".into());
    w.sprite_slots = vec![
        empty_sprite_slot("card_icon", Rect::new(20, 6, 20, 16)),
        empty_sprite_slot("detail_icon", Rect::new(80, 24, 20, 16)),
    ];
    let runtime_sprites = pickaxe_sprite();

    let assets = Assets { textures: &textures, runtime_sprites: &runtime_sprites, text_fonts: &TextFonts::new() };
    let out = compile_layouts(&[w], &[], &assets, "window", &PackTarget::default(), &BuildOptions::default()).unwrap();
    let paths: Vec<&str> = out.files.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"assets/window/font/sprite_y0.json"));
    assert!(paths.contains(&"assets/window/font/sprite_y18.json"));
    assert!(paths.contains(&"assets/window/font/sprite_y12.json"));
    assert_eq!(paths.iter().filter(|path| **path == "assets/window/textures/font/sprites/pickaxe.png").count(), 1);

    for path in ["assets/window/font/sprite_y0.json", "assets/window/font/sprite_y18.json"] {
        let doc: serde_json::Value = serde_json::from_slice(find(&out, path).contents.as_bytes()).unwrap();
        let provider = &doc["providers"].as_array().unwrap()[0];
        assert_eq!(provider["file"], "window:font/sprites/pickaxe.png");
        assert_eq!(provider["chars"][0].as_str().unwrap(), out.manifest.sprites["pickaxe"].glyph);
    }
    assert_eq!(out.manifest.windows["shop"].sprite_slots["card_icon"].font, "window:sprite_y0");
    assert_eq!(out.manifest.windows["shop"].sprite_slots["detail_icon"].font, "window:sprite_y18");
    assert_eq!(out.manifest.windows["shop"].buttons["buy"].sprite_font.as_deref(), Some("window:sprite_y12"));
    let descriptor =
        crate::debug::DebugDescriptor::from_json(find(&out, "assets/window/window/debug.json").contents.as_bytes())
            .unwrap();
    assert_eq!(descriptor.sprites["pickaxe"].resource.as_deref(), Some("window:font/sprites/pickaxe.png"));
}

#[test]
fn collection_selection_places_its_sprite_over_each_cell_box() {
    let (mut w, textures) = sample_window();
    w.collections = vec![CollectionIr {
        name: "products".into(),
        slots: vec![InventorySlotRef::container(9), InventorySlotRef::container(10)],
        selected_sprite: Some("pickaxe".into()),
        action: true,
        repeat: None,
    }];

    let runtime_sprites = pickaxe_sprite();
    let assets = Assets { textures: &textures, runtime_sprites: &runtime_sprites, text_fonts: &TextFonts::new() };
    let out = compile_layouts(&[w], &[], &assets, "window", &PackTarget::default(), &BuildOptions::default()).unwrap();
    let cells = &out.manifest.windows["shop"].collections["products"].selection;
    let boxes: Vec<_> = cells.iter().map(|cell| (cell.x, cell.y, cell.width, cell.height)).collect();
    assert_eq!(boxes, [(7, 35, 18, 18), (25, 35, 18, 18)]);
    assert!(cells.iter().all(|cell| cell.font == "window:sprite_y29" && cell.sprite.as_deref() == Some("pickaxe")));
    assert!(out.files.iter().any(|f| f.path == "assets/window/font/sprite_y29.json"));
}
