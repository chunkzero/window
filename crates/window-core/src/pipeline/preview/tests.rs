use serde_json::{Value, json};

use super::*;

const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

fn solid(rgba: [u8; 4]) -> Vec<u8> {
    Texture { width: 4, height: 4, rgba: rgba.repeat(16) }.encode_png().unwrap()
}

/// Renders one preview of a `generic_9x1` window `shop` with `children` and returns its pixels.
fn preview(children: Value, state: Value) -> Result<Texture> {
    let project = json!({ "windows": [{ "name": "shop", "container": "generic_9x1", "children": children }] });
    let request = json!({ "name": "shot", "target": { "kind": "window", "name": "shop" }, "state": state });
    render_one(project, request)
}

/// Renders one preview request of `project` and returns its pixels.
fn render_one(project: Value, request: Value) -> Result<Texture> {
    render_with(project, request, Vec::new())
}

/// Like [`render_one`], with `extra` textures beside the solid 4x4 ones.
fn render_with(project: Value, request: Value, extra: Vec<(&str, Texture)>) -> Result<Texture> {
    let project = crate::authoring::project_from_json(project.to_string().as_bytes())?;
    let files = [("red.png", RED), ("green.png", GREEN), ("blue.png", BLUE)];
    let mut files: Vec<(String, Vec<u8>)> =
        files.into_iter().map(|(path, rgba)| (path.to_string(), solid(rgba))).collect();
    files.extend(extra.into_iter().map(|(path, texture)| (path.to_string(), texture.encode_png().unwrap())));
    let input = CompileInput::new(files.into_iter().collect());
    let files = render(&project, &input, &requests_from_json(json!([request]).to_string().as_bytes())?)?;
    Texture::decode_png(files[0].contents.as_bytes())
}

/// The columns holding a pixel of `rgba`, as `(leftmost, rightmost)`.
fn columns(texture: &Texture, rgba: [u8; 4]) -> Option<(u32, u32)> {
    let holds = |x: &u32| (0..texture.height).any(|y| texture.get(*x, y) == rgba);
    Some(((0..texture.width).find(holds)?, (0..texture.width).rev().find(holds)?))
}

fn image(texture: &str) -> Value {
    json!({ "type": "sprite", "art": { "art": "texture", "texture": texture } })
}

#[test]
fn a_preview_draws_the_active_case_of_each_switch() {
    let mode = json!({ "kind": "selection", "id": "mode", "values": ["buy", "sell"], "initial": "buy" });
    let lit = json!({ "type": "switch", "handle": { "kind": "flag", "id": "lit" }, "children": [
        { "type": "case", "value": "true", "children": [image("green.png")] },
        { "type": "case", "value": "false", "children": [] }
    ] });
    let children = json!([{ "type": "switch", "handle": mode, "x": 10, "y": 20, "children": [
        { "type": "case", "value": "buy", "children": [image("red.png")] },
        { "type": "case", "value": "sell", "children": [image("blue.png"), lit] }
    ] }]);
    let shows = |state: Value| {
        let shot = preview(children.clone(), state).unwrap();
        [RED, GREEN, BLUE].map(|rgba| columns(&shot, rgba).is_some())
    };

    assert_eq!(shows(json!([])), [true, false, false], "a selection starts at its initial value");
    assert_eq!(
        shows(json!([{ "handle": "lit", "value": true }])),
        [true, false, false],
        "a hidden case hides its switches"
    );
    assert_eq!(shows(json!([{ "handle": "mode", "value": "sell" }])), [false, false, true]);
    let both = json!([{ "handle": "mode", "value": "sell" }, { "handle": "lit", "value": true }]);
    assert_eq!(shows(both), [false, true, true]);
}

#[test]
fn text_is_aligned_and_ellipsized_in_its_slot_like_the_runtime() {
    let slot = |overflow: Option<&str>| {
        json!([{ "type": "slot", "handle": { "kind": "text", "id": "title" }, "x": 20, "y": 30, "width": 40,
                 "align": "right", "color": "#ff0000", "shadow": false, "overflow": overflow }])
    };
    let ink = |overflow, text: &str| {
        let shot = preview(slot(overflow), json!([{ "handle": "title", "value": text }])).unwrap();
        columns(&shot, RED).unwrap()
    };

    // "Hi" advances 6 + 2 pixels, the last of them a gap, so its 7 pixels of ink end at the slot's right edge.
    let (left, right) = ink(None, "Hi");
    assert!(left >= 53 && right == 59, "{left}..{right}");
    let (left, right) = ink(None, "Overflowing words");
    assert!(left < 20 && right <= 59, "unfitted text keeps its alignment and spills: {left}..{right}");
    let (left, right) = ink(Some("ellipsis"), "Overflowing words");
    assert!(left >= 20 && right == 59, "ellipsized text stays in the slot: {left}..{right}");
}

#[test]
fn state_for_an_unknown_handle_or_of_the_wrong_kind_names_the_preview_and_handle() {
    let children = json!([
        { "type": "slot", "handle": { "kind": "text", "id": "title" }, "x": 8, "y": 30, "width": 40 },
        { "type": "switch", "handle": { "kind": "value", "id": "mode", "values": ["buy", "sell"] }, "x": 8, "y": 40,
          "children": [{ "type": "case", "value": "buy", "children": [] }, { "type": "case", "value": "sell", "children": [] }] }
    ]);
    let error = |state: Value| preview(children.clone(), state).unwrap_err().to_string();

    assert_eq!(
        error(json!([{ "handle": "titel", "value": "Shop" }])),
        "preview `shot`: window `shop` has no handle `titel`; its handles: mode, title"
    );
    assert_eq!(
        error(json!([{ "handle": "title", "value": true }])),
        "preview `shot`: handle `title` is a text and takes a string; got true"
    );
    assert_eq!(
        error(json!([{ "handle": "mode", "value": "rent" }])),
        "preview `shot`: handle `mode` is a value and takes one of buy, sell; got \"rent\""
    );
    assert_eq!(
        error(json!([{ "handle": "title", "at": [1], "value": "Shop" }])),
        "preview `shot`: `title[1]` does not name a member of handle `title`, which is not indexed"
    );
}

#[test]
fn an_image_over_the_pixel_limit_fails_before_it_is_allocated() {
    let project = json!({ "huds": [{ "name": "huge", "width": 65536, "height": 65536, "children": [] }] });
    let request = json!({ "name": "big", "target": { "kind": "hud", "name": "huge" }, "scale": 16 });
    let error = render_one(project, request).unwrap_err().to_string();

    assert_eq!(error, "preview `big`: its 65536x65536 image at scale 16 exceeds the limit of 16777216 pixels");
}

#[test]
fn sprite_state_takes_the_authored_camel_case_name() {
    let lamp = json!({ "art": "texture", "texture": "green.png" });
    let handle = json!({ "kind": "sprite", "id": "lamp", "only": ["lampOn"] });
    let project = json!({
        "sprites": { "lampOn": lamp },
        "windows": [{ "name": "shop", "container": "generic_9x1", "children": [
            { "type": "sprite_slot", "handle": handle, "x": 10, "y": 10, "width": 4, "height": 4 },
        ] }],
    });
    let request = json!({
        "name": "shot", "target": { "kind": "window", "name": "shop" }, "state": [{ "handle": "lamp", "value": "lampOn" }],
    });

    assert_eq!(columns(&render_one(project, request).unwrap(), GREEN), Some((10, 13)));
}

#[test]
fn a_runtime_sprite_keeps_its_aspect_ratio_at_the_authored_height() {
    let wide = json!({ "art": "texture", "texture": "red.png", "width": 8, "height": 4 });
    let handle = json!({ "kind": "sprite", "id": "icon" });
    let project = json!({
        "sprites": { "icon": wide },
        "windows": [{ "name": "shop", "container": "generic_9x1", "children": [
            { "type": "sprite_slot", "handle": handle, "x": 10, "y": 10, "width": 20, "height": 4 },
        ] }],
    });
    let request = json!({
        "name": "shot", "target": { "kind": "window", "name": "shop" }, "state": [{ "handle": "icon", "value": "icon" }],
    });

    assert_eq!(columns(&render_one(project, request).unwrap(), RED), Some((10, 13)));
}

#[test]
fn legacy_codes_are_consumed_with_bold_and_color_drawn() {
    let slot = json!([{ "type": "slot", "handle": { "kind": "text", "id": "title" }, "x": 20, "y": 30, "width": 60,
                        "color": "#ff0000", "shadow": false }]);
    let shot = |text: &str| preview(slot.clone(), json!([{ "handle": "title", "value": text }])).unwrap();
    let plain = shot("Hi");

    for consumed in ["§kHi", "§oHi", "§zHi", "Hi§", "§rHi"] {
        assert_eq!(shot(consumed).rgba, plain.rgba, "{consumed:?} draws as plain text");
    }
    let bold = shot("§lHi");
    assert_ne!(bold.rgba, plain.rgba);
    assert_eq!(columns(&bold, RED), columns(&shot("§LHi"), RED));
    assert_eq!(columns(&shot("§l§rHi"), RED), columns(&plain, RED), "`§r` restores the slot's style");
    let blue = shot("§9Hi");
    assert!(columns(&blue, RED).is_none() && columns(&blue, [0x55, 0x55, 0xff, 255]).is_some());
    assert!(columns(&shot("§9§rHi"), RED).is_some(), "`§r` restores the slot's color");
}

/// A `width` x `height` texture whose columns `opaque` are red and the rest transparent.
fn striped(width: u32, height: u32, opaque: std::ops::Range<u32>) -> Texture {
    let mut texture = Texture::transparent(width, height);
    for y in 0..height {
        for x in opaque.clone() {
            let i = ((y * width + x) * 4) as usize;
            texture.rgba[i..i + 4].copy_from_slice(&RED);
        }
    }
    texture
}

/// Previews `generic_9x1` window `shop` holding a sprite slot at (`x`, 10) of `slot` size, showing sprite `icon`.
fn runtime_sprite(art: Value, texture: Texture, x: u32, slot: (u32, u32)) -> Result<Texture> {
    let handle = json!({ "kind": "sprite", "id": "icon" });
    let project = json!({
        "sprites": { "icon": art },
        "windows": [{ "name": "shop", "container": "generic_9x1", "children": [
            { "type": "sprite_slot", "handle": handle, "x": x, "y": 10, "width": slot.0, "height": slot.1 },
        ] }],
    });
    let request = json!({
        "name": "shot", "target": { "kind": "window", "name": "shop" }, "state": [{ "handle": "icon", "value": "icon" }],
    });
    render_with(project, request, vec![("icon.png", texture)])
}

#[test]
fn a_runtime_sprite_wider_than_its_slot_grows_the_canvas() {
    let art = json!({ "art": "texture", "texture": "icon.png", "width": 32, "height": 4 });
    let shot = runtime_sprite(art, striped(32, 4, 0..32), 160, (16, 4)).unwrap();

    assert_eq!(shot.width, 192);
    assert_eq!(columns(&shot, RED), Some((160, 191)));
}

#[test]
fn a_padded_runtime_sprite_paints_the_columns_its_metrics_report() {
    let art = json!({ "art": "texture", "texture": "icon.png", "width": 5, "height": 5 });
    let shot = runtime_sprite(art, striped(9, 9, 1..8), 10, (20, 5)).unwrap();

    // `x_offset` 1 and `glyph_width` 3, with the ink starting at the slot's left edge.
    assert_eq!(columns(&shot, RED), Some((10, 12)));
}

#[test]
fn a_runtime_sprite_resampled_past_the_pixel_limit_fails_without_allocating() {
    let art = json!({ "art": "texture", "texture": "icon.png", "width": 131072, "height": 512 });
    let error = runtime_sprite(art, striped(256, 1, 0..256), 10, (16, 16)).unwrap_err().to_string();

    assert!(error.starts_with("preview `shot`: its 1310"), "{error}");
    assert!(error.contains("exceeds the limit"), "{error}");
}

#[test]
fn generated_runtime_sprites_past_the_pixel_limit_fail_before_they_are_rendered() {
    let art = json!({ "art": "shape", "kind": "panel", "width": 8192, "height": 4096 });
    let error = runtime_sprite(art, striped(1, 1, 0..1), 10, (16, 16)).unwrap_err().to_string();

    assert_eq!(error, "preview `shot`: its generated sprites exceed the limit of 16777216 pixels at sprite `icon`");
}

#[test]
fn a_sprite_at_an_extreme_position_fails_instead_of_overflowing() {
    let children = json!([{ "type": "sprite", "x": i32::MIN, "y": 0, "art": { "art": "texture", "texture": "red.png", "width": 1, "height": 1 } }]);
    let error = preview(children, json!([])).unwrap_err().to_string();

    assert!(error.starts_with("preview `shot`: its ") && error.contains("exceeds the limit"), "{error}");
}

#[test]
fn a_preview_of_an_unknown_target_names_the_preview_and_target() {
    let project = json!({ "windows": [{ "name": "shop", "container": "generic_9x1", "children": [] }] });
    let request = json!({ "name": "shot", "target": { "kind": "window", "name": "bank" } });

    assert_eq!(render_one(project, request).unwrap_err().to_string(), "preview `shot`: there is no window `bank`");
}

#[test]
fn long_text_ellipsizes_to_the_longest_fitting_prefix() {
    let slot = json!([{ "type": "slot", "handle": { "kind": "text", "id": "title" }, "x": 20, "y": 30, "width": 40,
                        "color": "#ff0000", "shadow": false, "overflow": "ellipsis" }]);
    let shot = |text: &str| preview(slot.clone(), json!([{ "handle": "title", "value": text }])).unwrap();
    let short = shot("Overflowing");

    assert_eq!(shot(&"Overflowing".repeat(2000)).rgba, short.rgba);
}
