use super::*;

fn sample() -> Manifest {
    Manifest {
        version: VERSION,
        namespace: "window".into(),
        font: "window:ui".into(),
        spacers: BTreeMap::from([(0xF0000, -1024), (0xF0015, 1024)]),
        text_advances: BTreeMap::from([('A', 6), (' ', 4)]),
        text_glyph_widths: BTreeMap::from([('A', 5), (' ', 0)]),
        font_metrics: BTreeMap::from([("window:y0".into(), sample_font_metrics())]),
        sprites: BTreeMap::new(),
        windows: BTreeMap::from([("shop".into(), sample_window())]),
        huds: BTreeMap::new(),
        colors: BTreeMap::new(),
    }
}

fn sample_font_metrics() -> FontMetricsEntry {
    FontMetricsEntry {
        advances: BTreeMap::from([('A', 6), (' ', 4)]),
        glyph_widths: BTreeMap::from([('A', 5), (' ', 0)]),
        bold_advance: 1,
    }
}

fn sample_window() -> WindowEntry {
    WindowEntry {
        surface: SurfaceEntry {
            kind: "container".into(),
            container: "generic_9x6".into(),
            size: [176, 222],
            title_origin: [8, 6],
        },
        static_text: "\u{E000}".into(),
        slots: BTreeMap::from([("title".into(), sample_title_slot())]),
        sprite_slots: BTreeMap::new(),
        buttons: BTreeMap::from([("buy".into(), sample_buy_button())]),
        items: BTreeMap::new(),
        collections: BTreeMap::new(),
        inputs: BTreeMap::new(),
        slot_rects: BTreeMap::new(),
        groups: BTreeMap::new(),
        switches: BTreeMap::new(),
        indexed: BTreeMap::new(),
    }
}

fn sample_title_slot() -> SlotEntry {
    SlotEntry {
        x: 8,
        y: 6,
        width: 160,
        align: Align::Center,
        font: "window:y0".into(),
        color: "#404040".into(),
        shader_marker: None,
        shader_color: None,
        shadow: false,
        bold: false,
        italic: false,
        underlined: false,
        strikethrough: false,
        obfuscated: false,
        text: None,
        binding: None,
        overflow: None,
        lines: None,
    }
}

fn sample_buy_button() -> ButtonEntry {
    ButtonEntry {
        x: 26,
        y: 36,
        width: 36,
        height: 18,
        slots: vec![
            SlotRefEntry { area: SlotAreaEntry::Container, index: 10 },
            SlotRefEntry { area: SlotAreaEntry::Container, index: 11 },
        ],
        fill_slots: None,
        default: None,
        action: true,
        tooltip: None,
        states: BTreeMap::new(),
        sprite_font: None,
        hover: None,
    }
}

#[test]
fn round_trips() {
    let m = sample();
    let bytes = m.to_json_bytes().unwrap();
    let back = Manifest::from_json(&bytes).unwrap();
    assert_eq!(m, back);
}

#[test]
fn serialization_is_deterministic() {
    let a = sample().to_json_bytes().unwrap();
    let b = sample().to_json_bytes().unwrap();
    assert_eq!(a, b);
}

#[test]
fn rejects_unknown_version() {
    let mut m = sample();
    m.version = 99;
    let bytes = serde_json::to_vec(&m).unwrap();
    assert!(Manifest::from_json(&bytes).is_err());
}
