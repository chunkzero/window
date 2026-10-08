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
        regions: BTreeMap::from([("buy.on".into(), sample_buy_region()), ("buy.off".into(), sample_buy_region())]),
        items: BTreeMap::new(),
        collections: BTreeMap::new(),
        inputs: BTreeMap::new(),
        groups: BTreeMap::new(),
        switches: BTreeMap::from([
            ("mode".into(), switch(None, &[("shop", &[], &["buy"])])),
            ("buy".into(), switch(Some("button `buy`"), &[("on", &["buy.on"], &[]), ("off", &["buy.off"], &[])])),
        ]),
        layers: vec![Layer::Switch("mode".into()), Layer::Switch("buy".into()), Layer::Slot("title".into())],
        indexed: BTreeMap::new(),
        handles: BTreeMap::new(),
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
        source: None,
    }
}

/// A switch with `(value, regions, switches)` cases.
fn switch(source: Option<&str>, cases: &[(&str, &[&str], &[&str])]) -> SwitchEntry {
    let names = |names: &[&str]| names.iter().map(|name| name.to_string()).collect();
    SwitchEntry {
        states: source.is_some(),
        source: source.map(Into::into),
        cases: cases
            .iter()
            .map(|(value, regions, switches)| SwitchCaseEntry {
                value: value.to_string(),
                regions: names(regions),
                switches: names(switches),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}

fn sample_buy_region() -> RegionEntry {
    RegionEntry {
        x: 26,
        y: 36,
        width: 36,
        height: 18,
        slots: vec![
            SlotRefEntry { area: SlotAreaEntry::Container, index: 10 },
            SlotRefEntry { area: SlotAreaEntry::Container, index: 11 },
        ],
        fill_slots: None,
        action: Some("buy".into()),
        default_action: None,
        hitbox: Some(Hitbox { item_model: Some("demo:gui/buy".into()), tooltip: None }),
        source: Some("button `buy`".into()),
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

#[test]
fn case_paths_follow_nested_switches() {
    let paths = case_paths(&sample_window().switches).regions;
    let on = &paths["buy.on"];
    assert_eq!(on, &vec![("mode".to_string(), "shop".to_string()), ("buy".to_string(), "on".to_string())]);
    assert!(exclusive_cases(on, &paths["buy.off"]));
    assert!(!exclusive_cases(on, &on[..1]));
    assert!(!exclusive_cases(on, &[]));
}
