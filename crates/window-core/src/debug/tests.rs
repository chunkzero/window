use super::*;
use crate::pipeline::{CompileInput, compile_project_json};

const PROJECT: &str = r##"{
  "theme":{"frames":{"panel":{"kind":"panel","fill":"#123456","border_width":0,"radius":0,"inset_depth":0}}},
  "windows":[{"name":"sample","container":"generic_9x3","children":[
    {"type":"panel","frame":"panel","x":8,"y":0,"width":16,"height":16},
    {"type":"label","text":"Hi","x":8,"y":20,"width":30},
    {"type":"slot","name":"value","x":8,"y":30,"width":30}
  ]}]
}"##;

#[test]
fn emitted_descriptor_parses_and_describes_semantic_layers() {
    let output = compile_project_json(PROJECT.as_bytes(), &CompileInput::new(BTreeMap::new())).unwrap();
    let file = output.files.iter().find(|file| file.path == output_path("window")).unwrap();
    let descriptor = DebugDescriptor::from_json(&file.contents).unwrap();
    assert_eq!(descriptor.pack_fingerprint.algorithm, "sha256");
    assert_eq!(descriptor.pack_fingerprint.value.len(), 64);
    assert_eq!(descriptor.resources["assets/window/font/ui.json"].resource_id.as_deref(), Some("window:ui"));
    assert_eq!(descriptor.windows["sample"].cursor_origin, [8, 6]);
    assert_eq!(
        descriptor.windows["sample"].cursor_contract,
        DebugCursorContract {
            convention: "independent_net_zero_segments".into(),
            independent_layer_net_advance: Some(0),
            composed_advance: 0,
        }
    );
    assert_eq!(
        descriptor.windows["sample"].layers.iter().map(|layer| layer.id.as_str()).collect::<Vec<_>>(),
        vec!["static", "text:label_0", "text:value"]
    );
    assert!(descriptor.windows["sample"].layers.iter().all(|layer| layer.net_advance == Some(0)));
    assert!(
        descriptor
            .resources
            .values()
            .any(|resource| { resource.image.as_ref().is_some_and(|image| !image.alpha_mask.data.is_empty()) })
    );
}

#[test]
fn descriptor_is_byte_identical_when_rebuilt() {
    let output = compile_project_json(PROJECT.as_bytes(), &CompileInput::new(BTreeMap::new())).unwrap();
    let emitted = output.files.iter().find(|file| file.path == output_path("window")).unwrap();
    let rebuilt = DebugDescriptor::build(&output.manifest, &output.files).unwrap().to_json_bytes().unwrap();
    assert_eq!(emitted.contents, rebuilt);

    let mut reversed = output.files.clone();
    reversed.reverse();
    let rebuilt_from_reversed = DebugDescriptor::build(&output.manifest, &reversed).unwrap().to_json_bytes().unwrap();
    assert_eq!(rebuilt, rebuilt_from_reversed);
}

#[test]
fn hud_descriptor_distinguishes_shared_and_independent_fixed_width_overlays() {
    let project = r##"{
      "theme":{"frames":{"panel":{"kind":"panel","fill":"#123456","border_width":0,"radius":0,"inset_depth":0}}},
      "huds":[
        {"name":"shared","channel":"actionbar","width":40,"height":12,"children":[
          {"type":"panel","frame":"panel","x":0,"y":0,"width":8,"height":8},
          {"type":"slot","name":"value","x":2,"y":2,"width":20}
        ]},
        {"name":"shader","channel":"actionbar","width":48,"height":12,
         "shader":{"source_bottom":59,"origin":{"x":0.5,"y":0.0},"anchor":{"x":0.5,"y":0.0}},
         "children":[{"type":"slot","name":"value","x":3,"y":2,"width":20}]}
      ]
    }"##;
    let output = compile_project_json(project.as_bytes(), &CompileInput::new(BTreeMap::new())).unwrap();
    let file = output.files.iter().find(|file| file.path == output_path("window")).unwrap();
    let descriptor = DebugDescriptor::from_json(&file.contents).unwrap();

    let shared = &descriptor.huds["shared"];
    assert_eq!(shared.cursor_contract.convention, "fixed_width_shared_cursor");
    assert_eq!(shared.cursor_contract.independent_layer_net_advance, None);
    assert_eq!(shared.cursor_contract.composed_advance, 40);
    assert_eq!(shared.layers[0].kind, "static_chrome");
    assert_eq!(shared.layers[0].net_advance, Some(40));
    assert_eq!(shared.layers[1].net_advance, None);

    let shader = &descriptor.huds["shader"];
    assert_eq!(shader.cursor_contract.convention, "fixed_width_with_independent_overlays");
    assert_eq!(shader.cursor_contract.independent_layer_net_advance, Some(0));
    assert_eq!(shader.cursor_contract.composed_advance, 48);
    assert_eq!(shader.layers[0].net_advance, Some(0));
}
