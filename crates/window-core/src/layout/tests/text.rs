use super::*;

#[test]
fn label_auto_width_from_text() {
    let w = solve_one(
        window(json!([
            { "type": "label", "text": "Hi", "x": 8, "y": 6 }
        ])),
        &sizes(&[]),
    );
    let l = &w.slots[0];
    assert_eq!(l.name, "label_0");
    assert_eq!(l.text.as_deref(), Some("Hi"));
    assert_eq!(l.rect.width, vanilla::text_visible_width("Hi"));
}

#[test]
fn bold_label_auto_width_uses_bold_visible_width() {
    let w = solve_one(
        window(json!([
            { "type": "label", "text": "Hi", "x": 8, "y": 6, "bold": true }
        ])),
        &sizes(&[]),
    );
    let l = &w.slots[0];
    assert!(l.bold);
    assert_eq!(l.rect.width, vanilla::bold_text_visible_width("Hi"));
}

#[test]
fn generated_label_names_in_order() {
    let w = solve_one(
        window(json!([{
            "type": "flex", "x": 8, "y": 6, "style": { "direction": "column" },
            "children": [
                { "type": "label", "text": "a" },
                { "type": "label", "text": "b" }
            ]
        }])),
        &sizes(&[]),
    );
    assert_eq!(w.slots[0].name, "label_0");
    assert_eq!(w.slots[1].name, "label_1");
}

#[test]
fn slot_outside_gui_errors() {
    let project = window(json!([
        { "type": "slot", "handle": text("a"), "width": 10, "x": 200, "y": 6 }
    ]));
    let err = solve(&project, &sizes(&[])).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("outside"), "{msg}");
}

#[test]
fn text_above_ascent_limit_errors() {
    // title_origin.y = 6 → min y is 5; y=0 is too high.
    let project = window(json!([
        { "type": "slot", "handle": text("a"), "width": 10, "x": 8, "y": 0 }
    ]));
    let err = solve(&project, &sizes(&[])).unwrap_err();
    assert!(err.to_string().contains("ascent limit"), "{err}");
}

#[test]
fn text_fonts_measure_labels_and_reject_unknown_names() {
    let label = |font: &str| window(json!([{ "type": "label", "text": "Lv 12", "x": 8, "y": 6, "font": font }]));
    let w = solve_one(label("small_caps"), &sizes(&[]));
    assert_eq!(w.slots[0].font.as_deref(), Some("small_caps"));
    // L (5) + v (6) + space (4) + 1 (4) + the 3px-wide 2.
    assert_eq!(w.slots[0].rect.width, 22);
    let err = solve(&label("runes"), &sizes(&[])).unwrap_err();
    assert!(err.to_string().contains("unknown font `runes`"), "{err}");
}

#[test]
fn widthless_slot_outside_a_box_is_rejected() {
    let error = solve(&window(json!([{ "type": "slot", "handle": text("caption"), "x": 8, "y": 18 }])), &sizes(&[]))
        .unwrap_err();
    assert!(error.to_string().contains("requires `width` unless a box lays it out"), "{error}");
}

#[test]
fn static_label_width_overflow_is_reported() {
    let w = solve_one(
        window(json!([{
            "type": "label", "text": "OVERFLOW", "x": 8, "y": 18, "width": 8
        }])),
        &sizes(&[]),
    );
    assert!(w.warnings.iter().any(|warning| warning.contains("text `OVERFLOW`") && warning.contains("reserves 8px")));
}

#[test]
fn static_label_warns_for_unsupported_shifted_font_glyphs() {
    let w = solve_one(
        window(json!([{
            "type": "label", "text": "STATUS 😀", "x": 8, "y": 18
        }])),
        &sizes(&[]),
    );
    assert!(
        w.warnings
            .iter()
            .any(|warning| { warning.contains("unsupported shifted-font glyph") && warning.contains("U+1F600") })
    );
}

#[test]
fn static_label_accepts_standard_ui_markers() {
    let w = solve_one(
        window(json!([{
            "type": "label", "text": "× ▲ ▼ ◆ ● ·", "x": 8, "y": 18
        }])),
        &sizes(&[]),
    );
    assert!(w.warnings.is_empty(), "{:?}", w.warnings);
}
