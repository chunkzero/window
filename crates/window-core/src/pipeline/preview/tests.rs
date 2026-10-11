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
    let project = crate::authoring::project_from_json(project.to_string().as_bytes())?;
    let files = [("red.png", RED), ("green.png", GREEN), ("blue.png", BLUE)];
    let input = CompileInput::new(files.into_iter().map(|(path, rgba)| (path.to_string(), solid(rgba))).collect());
    let request = json!([{ "name": "shot", "target": { "kind": "window", "name": "shop" }, "state": state }]);
    let files = render(&project, &input, &requests_from_json(request.to_string().as_bytes())?)?;
    assert_eq!(files[0].path, "shot.png");
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
