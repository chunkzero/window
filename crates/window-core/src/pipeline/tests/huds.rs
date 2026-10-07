use super::*;
use crate::ir::{HudChannel, HudShader};

#[test]
fn shader_placed_hud_requires_hud_shaders() {
    let hud = LaidOutHud {
        name: "status".into(),
        channel: HudChannel::ActionBar,
        width: 120,
        height: 20,
        shader: Some(HudShader {
            source_bottom: 59,
            origin_x: 0.5,
            origin_y: 0.1,
            anchor_x: 0.5,
            anchor_y: 0.0,
            offset_x: 0,
            offset_y: 0,
        }),
        draws: vec![],
        slots: vec![],
        switches: vec![],
        indexed: BTreeMap::new(),
        warnings: vec![],
    };
    let fonts = text_font::resolve(&BTreeMap::new(), &BTreeMap::new()).unwrap();
    let assets = Assets { textures: &BTreeMap::new(), runtime_sprites: &BTreeMap::new(), text_fonts: &fonts };
    let target = PackTarget { pack_format: Some(88) };

    let err = compile_layouts(&[], &[hud], &assets, "demo", &target, &BuildOptions::default()).unwrap_err();

    assert!(err.to_string().contains("HUD `status` uses `shader` placement, but hudShaders is disabled"), "{err}");
}
