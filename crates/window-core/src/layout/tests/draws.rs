use super::*;

#[test]
fn sprite_intrinsic_size_and_draw() {
    let w = solve_one(
        themed(
            json!({ "sprites": { "coin": { "texture": "coin.png" } } }),
            json!([{ "type": "sprite", "name": "coin", "x": 8, "y": 6 }]),
        ),
        &sizes(&[("coin.png", 12, 12)]),
    );
    match &w.draws[0] {
        Draw::Sprite { dest, texture } => {
            assert_eq!(*dest, Rect::new(8, 6, 12, 12));
            assert_eq!(texture.0, "coin.png");
        }
        other => panic!("expected sprite, got {other:?}"),
    }
}

#[test]
fn generated_sprite_intrinsic_size_and_draw() {
    let w = solve_one(
        themed(
            json!({
                "sprites": {
                    "badge": {
                        "kind": "badge",
                        "width": 18,
                        "height": 14,
                        "fill": "#ff8700"
                    }
                }
            }),
            json!([{ "type": "sprite", "name": "badge", "x": 8, "y": 6 }]),
        ),
        &sizes(&[]),
    );
    match &w.draws[0] {
        Draw::Generated { dest, style } => {
            assert_eq!(*dest, Rect::new(8, 6, 18, 14));
            assert_eq!(style.fill, Rgb::new(0xff, 0x87, 0x00));
        }
        other => panic!("expected generated sprite, got {other:?}"),
    }
}

#[test]
fn missing_sprite_texture_errors() {
    let project = themed(
        json!({ "sprites": { "coin": { "texture": "coin.png" } } }),
        json!([{ "type": "sprite", "name": "coin", "x": 8, "y": 6 }]),
    );
    let err = solve(&project, &sizes(&[])).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("missing texture"), "{msg}");
    assert!(msg.contains("coin.png"), "{msg}");
}

#[test]
fn generated_frame_draws_without_texture_lookup() {
    let w = solve_one(
        themed(
            json!({
                "frames": {
                    "panel": {
                        "kind": "panel",
                        "fill": "#123456",
                        "border_color": "#abcdef"
                    }
                }
            }),
            json!([{ "type": "panel", "frame": "panel", "x": 0, "y": 0, "width": 40, "height": 20 }]),
        ),
        &sizes(&[]),
    );
    match &w.draws[0] {
        Draw::Generated { dest, style } => {
            assert_eq!(*dest, Rect::new(0, 0, 40, 20));
            assert_eq!(style.border_color, Rgb::new(0xab, 0xcd, 0xef));
        }
        other => panic!("expected generated frame, got {other:?}"),
    }
}

#[test]
fn frame_too_small_for_insets_errors() {
    // insets uniform 8 → needs >=17x17, but texture is 10x10.
    let project = themed(
        json!({ "frames": { "p": { "texture": "p.png", "insets": 8 } } }),
        json!([{ "type": "panel", "frame": "p", "x": 0, "y": 0, "width": 176, "height": 40 }]),
    );
    let err = solve(&project, &sizes(&[("p.png", 10, 10)])).unwrap_err();
    assert!(err.to_string().contains("too small"), "{err}");
}

#[test]
fn draw_overflow_is_a_warning() {
    // A sprite extending past the right edge warns but does not error.
    let w = solve_one(
        themed(
            json!({ "sprites": { "wide": { "texture": "wide.png" } } }),
            json!([{ "type": "sprite", "name": "wide", "x": 170, "y": 6 }]),
        ),
        &sizes(&[("wide.png", 20, 8)]),
    );
    assert_eq!(w.slots.len(), 0);
    assert_eq!(w.draws.len(), 1);
    assert_eq!(w.warnings.len(), 1);
    assert!(w.warnings[0].contains("extends outside"), "{:?}", w.warnings);
}

#[test]
fn bleed_allows_visual_overflow_but_not_text_overflow() {
    let visual_project = project(json!({
        "theme": {
            "frames": {
                "panel": { "kind": "panel" }
            }
        },
        "windows": [{
            "name": "s",
            "container": "generic_9x3",
            "bleed": { "top": 10, "left": 6 },
            "children": [
                { "type": "panel", "frame": "panel", "x": -6, "y": -10, "width": 20, "height": 20 }
            ]
        }]
    }));
    let w = solve(&visual_project, &sizes(&[])).unwrap().pop().unwrap();
    assert!(w.warnings.is_empty(), "bleed should suppress visual overflow warnings");

    let text_project = project(json!({
        "windows": [{
            "name": "s",
            "container": "generic_9x3",
            "bleed": { "left": 10 },
            "children": [
                { "type": "slot", "name": "bad", "x": -4, "y": 6, "width": 10 }
            ]
        }]
    }));
    let err = solve(&text_project, &sizes(&[])).unwrap_err();
    assert!(err.to_string().contains("outside"), "{err}");
}
