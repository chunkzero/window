//! Generate resources consumed by the real Minecraft client validation mod.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use window_core::compose::Texture;
use window_core::pipeline::{CompileInput, compile_project_json};
use window_core::validation::validate_compile_output;

const PROJECT: &str = r##"{
  "options": {
    "hud_shaders": true
  },
  "target": {
    "pack_format": 88
  },
  "sprites": {
    "probe_marker": {
      "kind": "badge",
      "width": 8,
      "height": 8,
      "fill": "#ff00aa",
      "border_color": "#ff00aa",
      "border_width": 0,
      "radius": 0,
      "inset_depth": 0,
      "highlight_color": "#ff00aa",
      "shadow_color": "#ff00aa",
      "accent_color": "#ff00aa",
      "art": "shape"
    }
  },
  "windows": [
    {
      "name": "probe",
      "container": "generic_9x3",
      "children": [
        {
          "type": "flex",
          "frame": {
            "kind": "panel",
            "fill": "#174a5b",
            "border_color": "#d98b2b",
            "border_width": 2,
            "radius": 0,
            "inset_depth": 0,
            "art": "shape"
          },
          "x": 16,
          "y": 10,
          "style": {
            "width": 64,
            "height": 32
          }
        },
        {
          "type": "slot",
          "handle": {
            "kind": "text",
            "id": "probe_text"
          },
          "x": 24,
          "y": 52,
          "width": 48,
          "color": "#35a7ff",
          "shadow": false
        },
        {
          "type": "sprite_slot",
          "handle": {
            "kind": "sprite",
            "id": "probe_sprite"
          },
          "x": 88,
          "y": 20,
          "width": 8,
          "height": 8,
          "align": "left"
        },
        {
          "type": "collection",
          "handle": {
            "kind": "collection",
            "id": "probe_collection"
          },
          "frame": {
            "kind": "slot",
            "fill": "#0b536d",
            "border_color": "#22b8e6",
            "border_width": 2,
            "radius": 0,
            "inset_depth": 1,
            "art": "shape"
          },
          "action": false,
          "transform": {
            "section": "container",
            "x": 1,
            "y": 1,
            "width": 3,
            "height": 1
          }
        },
        {
          "type": "collection",
          "handle": {
            "kind": "collection",
            "id": "probe_card_icon"
          },
          "slots": [
            {
              "area": "container",
              "first": 5,
              "last": 5
            },
            {
              "area": "container",
              "first": 7,
              "last": 7
            }
          ]
        },
        {
          "type": "section",
          "section": "container",
          "claim": "none",
          "children": [
            {
              "type": "region",
              "on_click": {
                "kind": "action",
                "id": "probe_container_button"
              },
              "tooltip": "Container runtime probe",
              "layout": {
                "column": 1,
                "row": 1
              }
            },
            {
              "type": "flex",
              "frame": {
                "kind": "slot",
                "fill": "#0b536d",
                "border_color": "#22b8e6",
                "border_width": 2,
                "radius": 0,
                "inset_depth": 1,
                "art": "shape"
              },
              "layout": {
                "column": {
                  "start": 6,
                  "span": 2
                },
                "row": {
                  "start": 1,
                  "span": 2
                }
              }
            },
            {
              "type": "region",
              "on_click": {
                "kind": "action",
                "id": "probe_card",
                "shape": [
                  2
                ],
                "at": [
                  0
                ]
              },
              "layout": {
                "column": 7,
                "row": 1
              }
            },
            {
              "type": "region",
              "on_click": {
                "kind": "action",
                "id": "probe_card",
                "shape": [
                  2
                ],
                "at": [
                  0
                ]
              },
              "layout": {
                "column": {
                  "start": 6,
                  "span": 2
                },
                "row": 2
              }
            },
            {
              "type": "flex",
              "frame": {
                "kind": "slot",
                "fill": "#0b536d",
                "border_color": "#22b8e6",
                "border_width": 2,
                "radius": 0,
                "inset_depth": 1,
                "art": "shape"
              },
              "layout": {
                "column": {
                  "start": 8,
                  "span": 2
                },
                "row": {
                  "start": 1,
                  "span": 2
                }
              }
            },
            {
              "type": "region",
              "on_click": {
                "kind": "action",
                "id": "probe_card",
                "shape": [
                  2
                ],
                "at": [
                  1
                ]
              },
              "layout": {
                "column": 9,
                "row": 1
              }
            },
            {
              "type": "region",
              "on_click": {
                "kind": "action",
                "id": "probe_card",
                "shape": [
                  2
                ],
                "at": [
                  1
                ]
              },
              "layout": {
                "column": {
                  "start": 8,
                  "span": 2
                },
                "row": 2
              }
            }
          ]
        },
        {
          "type": "section",
          "section": "hotbar",
          "claim": "none",
          "children": [
            {
              "type": "region",
              "on_click": {
                "kind": "action",
                "id": "probe_hotbar_button"
              },
              "tooltip": "Player runtime probe",
              "layout": {
                "column": 1,
                "row": 1
              }
            }
          ]
        }
      ]
    },
    {
      "name": "search_probe",
      "container": "anvil",
      "children": [
        {
          "type": "flex",
          "frame": {
            "kind": "panel",
            "fill": "#087da1",
            "border_color": "#29d6ff",
            "border_width": 2,
            "radius": 0,
            "inset_depth": 1,
            "art": "shape"
          },
          "x": 54,
          "y": 18,
          "style": {
            "width": 116,
            "height": 20
          }
        },
        {
          "type": "label",
          "text": "Search maps",
          "x": 60,
          "y": 6,
          "width": 104,
          "align": "center",
          "color": "#ffffff"
        },
        {
          "type": "anvil_input",
          "handle": {
            "kind": "input",
            "id": "query"
          },
          "initial": "",
          "item_model": "window:gui/hitbox"
        }
      ]
    }
  ],
  "huds": [
    {
      "name": "probe_hud",
      "channel": "actionbar",
      "width": 40,
      "height": 32,
      "shader": {
        "source_bottom": 59,
        "origin": {
          "x": 0.5,
          "y": 0.0
        },
        "anchor": {
          "x": 0.5,
          "y": 0.0
        },
        "x": 0,
        "y": 8
      },
      "children": [
        {
          "type": "flex",
          "frame": {
            "kind": "panel",
            "fill": "#7f1d1d",
            "border_color": "#ffb000",
            "border_width": 2,
            "radius": 0,
            "inset_depth": 1,
            "art": "shape"
          },
          "x": 0,
          "y": 0,
          "style": {
            "width": 40,
            "height": 32
          }
        }
      ]
    }
  ]
}"##;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let destination =
        env::args_os().nth(1).map(PathBuf::from).ok_or("usage: generate_minecraft_test <resource-output-directory>")?;

    let output = compile_project_json(PROJECT.as_bytes(), &CompileInput::new(Default::default()))?;
    let validation = validate_compile_output(&output);
    write(&destination.join("window-test/compiler-validation.json"), &validation.to_json_pretty()?)?;
    if !validation.is_valid() {
        return Err(validation.to_string().into());
    }

    for file in &output.files {
        write(&destination.join(&file.path), file.contents.as_bytes())?;
    }
    write(&destination.join("window-test/manifest.json"), &output.manifest.to_json_bytes()?)?;

    let static_texture = output
        .files
        .iter()
        .find(|file| file.path == "assets/window/textures/font/probe.png")
        .ok_or("compiler did not emit the probe static texture")?;
    write(
        &destination.join("templates/window-static-probe.png"),
        &crop_png(static_texture.contents.as_bytes(), 0, 0, 64, 32)?,
    )?;
    write(
        &destination.join("templates/window-collection-cell.png"),
        &crop_png(static_texture.contents.as_bytes(), 10, 26, 16, 16)?,
    )?;
    let sprite_texture = output
        .files
        .iter()
        .find(|file| file.path == "assets/window/textures/font/sprites/probe/marker.png")
        .ok_or("compiler did not emit the probe runtime sprite texture")?;
    write(&destination.join("templates/window-sprite-probe.png"), sprite_texture.contents.as_bytes())?;
    let search_texture = output
        .files
        .iter()
        .find(|file| file.path == "assets/window/textures/font/search_probe.png")
        .ok_or("compiler did not emit the search UI texture")?;
    write(&destination.join("templates/window-search-probe.png"), search_texture.contents.as_bytes())?;
    let hud_texture = output
        .files
        .iter()
        .find(|file| file.path == "assets/window/textures/font/hud_probe_hud.png")
        .ok_or("compiler did not emit the shader HUD texture")?;
    write(&destination.join("templates/window-hud-probe.png"), hud_texture.contents.as_bytes())?;

    let metadata = serde_json::to_vec_pretty(&json!({
        "gui": { "width": 176, "height": 168 },
        "static_rect": { "x": 16, "y": 10, "width": 64, "height": 32 },
        "static_texture_rect": { "x": 16, "y": 10, "width": 64, "height": 42 },
        "text": "III×",
        "clicked_text": "WWW×",
        "text_color": "#35a7ff",
        "sprite": "probe_marker",
        "container_button_slot": 0,
        "hotbar_button_slot": 0,
        "hotbar_menu_slot": 54,
        "collection_slots": [10, 11, 12],
        "card": {
            "action": "probe_card",
            "cells": [[5, 6, 14, 15], [7, 8, 16, 17]],
            "item_slots": [5, 7],
            "items": ["minecraft:diamond", "minecraft:emerald"],
            "route_only_slot": 16
        },
        "collection_first_rect": { "x": 26, "y": 36, "width": 16, "height": 16 },
        "search": {
            "window": "search_probe",
            "query": "maps",
            "gui": { "width": 176, "height": 166 },
            "static_rect": { "x": 54, "y": 18, "width": 116, "height": 20 }
        },
        "hud": {
            "name": "probe_hud",
            "width": 40,
            "height": 32,
            "origin_x": 0.5,
            "origin_y": 0.0,
            "anchor_x": 0.5,
            "anchor_y": 0.0,
            "offset_x": 0,
            "offset_y": 8
        },
        "validated_elements": [
            "generated_panel",
            "collection_frame",
            "runtime_sprite",
            "dynamic_text_and_shifted_status_marker",
            "cell_item_and_click_routing",
            "native_anvil_input",
            "shader_hud"
        ]
    }))?;
    write(&destination.join("window-test/probe.json"), &metadata)?;
    Ok(())
}

fn crop_png(bytes: &[u8], x: u32, y: u32, width: u32, height: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let source = Texture::decode_png(bytes)?;
    if x + width > source.width || y + height > source.height {
        return Err("template crop exceeds compiler texture bounds".into());
    }
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for row in y..y + height {
        let start = ((row * source.width + x) * 4) as usize;
        let end = start + (width * 4) as usize;
        rgba.extend_from_slice(&source.rgba[start..end]);
    }
    Ok(Texture { width, height, rgba }.encode_png()?)
}

fn write(path: &Path, contents: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}
