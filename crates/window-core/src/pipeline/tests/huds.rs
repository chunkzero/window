use super::*;
use crate::authoring::CoreShaders;
use crate::font::MAX_TEXT_ROW;
use crate::ir::{HudChannel, HudShader, SwitchCaseIr, SwitchIr};

const SHADER: HudShader = HudShader {
    source_bottom: 59,
    origin_x: 0.5,
    origin_y: 0.1,
    anchor_x: 0.5,
    anchor_y: 0.0,
    offset_x: 0,
    offset_y: 0,
};

/// A HUD with one text slot on each of `rows` and a frame `art_height` pixels tall (none when `0`).
fn hud(name: &str, shader: Option<HudShader>, rows: &[i32], art_height: u32) -> LaidOutHud {
    let draws = match art_height {
        0 => vec![],
        height => vec![Draw::NineSlice {
            texture: TextureKey("frame.png".into()),
            insets: Insets::uniform(1),
            dest: Rect::new(0, 0, 80, height),
        }],
    };
    let slots = rows
        .iter()
        .map(|&y| text_slot(&format!("row{y}"), None, Rect::new(4, y, 40, 8), Align::Left, Rgb::DEFAULT_TEXT))
        .collect();
    LaidOutHud {
        name: name.into(),
        channel: HudChannel::ActionBar,
        width: 80,
        height: art_height.max(40),
        shader,
        draws,
        slots,
        switches: vec![],
        art: Vec::new(),
        layers: vec![],
        handles: BTreeMap::new(),
        warnings: vec![],
    }
}

fn compile_huds(huds: &[LaidOutHud], hud_shaders: bool) -> Result<CompileOutput> {
    let textures = BTreeMap::from([("frame.png".to_string(), solid(4, 4, [200, 200, 200, 255]))]);
    let fonts = text_font::resolve(&BTreeMap::new(), &BTreeMap::new()).unwrap();
    let assets = Assets { textures: &textures, runtime_sprites: &BTreeMap::new(), text_fonts: &fonts };
    let target = PackTarget { pack_format: Some(88) };
    let options = BuildOptions { core_shaders: CoreShaders { hud: hud_shaders }, ..Default::default() };
    compile_layouts(&[], huds, &assets, "demo", &target, &options)
}

fn text_row_fonts(out: &CompileOutput) -> Vec<&str> {
    let fonts = out.files.iter().filter_map(|file| file.path.strip_prefix("assets/demo/font/y"));
    fonts.filter_map(|name| name.strip_suffix(".json")).collect()
}

#[test]
fn shader_placed_hud_requires_hud_shaders() {
    let err = compile_huds(&[hud("status", Some(SHADER), &[], 0)], false).unwrap_err();

    assert!(
        err.to_string().contains("HUD `status` uses `shader` placement, but `coreShaders.hud` is disabled"),
        "{err}"
    );
}

#[test]
fn hud_art_is_limited_to_one_font_atlas_page() {
    compile_huds(&[hud("tall", Some(SHADER), &[300], 256)], true).expect("256 px of art fits one glyph");

    let err = compile_huds(&[hud("tall", Some(SHADER), &[], 257)], true).unwrap_err().to_string();
    assert!(err.contains("HUD `tall`: art `hud_tall` bakes into one 80x257 px font glyph"), "{err}");
    assert!(err.contains("font atlas pages are 256x256 px"), "{err}");
}

#[test]
fn shader_placed_huds_share_text_rows_without_moving() {
    let huds = [
        hud("a", Some(SHADER), &[0, 10], 0),
        hud("b", Some(SHADER), &[3, 13], 20),
        hud("c", Some(SHADER), &[4, 14, 24], 0),
        hud("fallback", None, &[5], 20),
    ];
    let out = compile_huds(&huds, true).unwrap();

    // Seven distinct shader-placed rows collapse onto three; the fallback HUD keeps its own row.
    assert_eq!(text_row_fonts(&out), ["0", "10", "20", "5"]);
    let row_offsets = ["a", "b", "c"].map(|name| out.manifest.huds[name].shader.as_ref().unwrap().row_offset);
    assert_eq!(row_offsets, [0, -3, -4]);
    assert_eq!(out.manifest.huds["b"].slots["row13"].font, "demo:y10");
    assert_eq!(out.manifest.huds["c"].slots["row24"].font, "demo:y20");
    assert_eq!(out.manifest.huds["fallback"].slots["row5"].font, "demo:y5");

    // Art moves with its HUD's text, and each shader rule looks for its HUD where it was baked.
    let ui: serde_json::Value =
        serde_json::from_slice(find(&out, "assets/demo/font/ui.json").contents.as_bytes()).unwrap();
    let ascents: Vec<i64> =
        ui["providers"].as_array().unwrap().iter().filter_map(|provider| provider["ascent"].as_i64()).collect();
    assert_eq!(ascents, [10, 7]);
    let shader = find(&out, "assets/minecraft/shaders/core/text.vsh").contents.as_bytes();
    let shader = std::str::from_utf8(shader).unwrap();
    assert!(shader.contains("const int WINDOW_HUD_SOURCE_BOTTOMS[WINDOW_HUD_RULES] = int[](59, 62, 63);"), "{shader}");
}

#[test]
fn styled_text_keeps_its_hud_unshifted() {
    for style in ["italic", "underlined", "strikethrough"] {
        let mut styled = hud("b", Some(SHADER), &[3, 13], 20);
        let label = &mut styled.slots[0];
        match style {
            "italic" => label.italic = true,
            "underlined" => label.underlined = true,
            _ => label.strikethrough = true,
        }
        // The styled text sits in a switch case, as it would anywhere in the tree.
        styled.switches = vec![SwitchIr {
            name: "mode".into(),
            source: None,
            cases: vec![SwitchCaseIr { value: "on".into(), slots: vec![label.name.clone()], ..Default::default() }],
        }];
        let out = compile_huds(&[hud("a", Some(SHADER), &[0, 10], 0), styled], true).unwrap();

        assert_eq!(out.manifest.huds["b"].shader.as_ref().unwrap().row_offset, 0, "{style}");
        assert_eq!(text_row_fonts(&out), ["0", "10", "13", "3"], "{style}");
    }
}

#[test]
fn row_offsets_stay_within_the_font_limits() {
    let huds = [hud("a", Some(SHADER), &[32760, 32765], 0), hud("b", Some(SHADER), &[0, 5, 100], 0)];
    let out = compile_huds(&huds, true).expect("no chosen offset leaves the supported text rows");

    let offset = out.manifest.huds["b"].shader.as_ref().unwrap().row_offset;
    assert!((-1..=MAX_TEXT_ROW - 100).contains(&offset), "{offset}");
}
