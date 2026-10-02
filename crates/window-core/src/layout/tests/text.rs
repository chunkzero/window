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
            "type": "column", "x": 8, "y": 6,
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
fn duplicate_names_within_window_error() {
    let project = themed(
        json!({}),
        json!([{
            "type": "column", "x": 8, "y": 6,
            "children": [
                { "type": "slot", "name": "dup", "width": 10 },
                { "type": "button", "name": "dup", "width": 10, "height": 10 }
            ]
        }]),
    );
    let err = solve(&project, &sizes(&[])).unwrap_err();
    assert!(err.to_string().contains("duplicate name `dup`"), "{err}");
}

#[test]
fn slot_outside_gui_errors() {
    let project = window(json!([
        { "type": "slot", "name": "a", "width": 10, "x": 200, "y": 6 }
    ]));
    let err = solve(&project, &sizes(&[])).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("outside"), "{msg}");
}

#[test]
fn text_above_ascent_limit_errors() {
    // title_origin.y = 6 → min y is 5; y=0 is too high.
    let project = window(json!([
        { "type": "slot", "name": "a", "width": 10, "x": 8, "y": 0 }
    ]));
    let err = solve(&project, &sizes(&[])).unwrap_err();
    assert!(err.to_string().contains("ascent limit"), "{err}");
}

#[test]
fn widthless_slot_outside_button_is_rejected() {
    let error =
        solve(&window(json!([{ "type": "slot", "name": "caption", "x": 8, "y": 18 }])), &sizes(&[])).unwrap_err();
    assert!(error.to_string().contains("direct, unpositioned button child"));
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
