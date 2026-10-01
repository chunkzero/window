//! The engine's entry point: bytes in, pack artifacts out.
//!
//! Orchestrates authored model → layout → compose → font → bake → compiled
//! definition. This is the only API the rpp plugin (or any other host) needs.
//!
//! The bulk of the backend lives in [`compile_windows`], a pure function over
//! laid-out IR and decoded textures. The public entry points are thin wiring
//! layers: they decode textures, solve layout, then delegate to
//! [`compile_windows`].

use std::collections::{BTreeMap, BTreeSet};

use crate::authoring::ParsedProject;
use crate::authoring::{BuildOptions, PackTarget};
use crate::compose::{Composite, Texture, compose_draws, compose_window};
use crate::font::{
    allocate, bitmap_provider, bitmap_provider_file, main_font, provider_font, shifted_font, shifted_suffix,
    space_provider, spacer_table,
};
use crate::geometry::{Point, Size};
use crate::inventory::{InventorySlotArea, InventorySlotRef, SlotRectClaim};
use crate::ir::{ButtonDefault, HudShader, LaidOutHud, LaidOutWindow, RepeatBindingIr, Rgb, SlotIr, SpriteSlotIr};
use crate::manifest::{
    AnvilInputEntry, ButtonEntry, CollectionEntry, FontMetricsEntry, HudEntry, HudShaderEntry, HudSurfaceEntry,
    ItemEntry, Manifest, RepeatGroupEntry, SlotEntry, SlotRectEntry, SlotRefEntry, SpriteEntry, SpriteSlotEntry,
    SurfaceEntry, VERSION, WindowEntry,
};
use crate::model::{GeneratedStyle, SpriteDef};
use crate::surface::{ContainerKind, Surface};
use crate::{Error, Result, vanilla};

/// Everything the engine needs to compile a Window project.
#[derive(Clone, Debug)]
pub struct CompileInput {
    /// Resource namespace for emitted fonts/textures (default `"window"`).
    pub namespace: String,
    /// Pack-source files keyed by pack-source-relative forward-slash path. PNGs
    /// are decoded for sprite and frame references; TypeScript source files are
    /// handled by the rpp plugin before this input reaches the compiler.
    pub files: BTreeMap<String, Vec<u8>>,
}

impl CompileInput {
    /// A `CompileInput` with default namespace and manifest path.
    pub fn new(files: BTreeMap<String, Vec<u8>>) -> Self {
        Self { namespace: "window".into(), files }
    }
}

/// One file to write into the pack output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputFile {
    /// Pack-output-relative forward-slash path.
    pub path: String,
    /// File contents.
    pub contents: Vec<u8>,
}

/// The result of a successful compile.
#[derive(Clone, Debug)]
pub struct CompileOutput {
    /// Files to emit (font providers, glyph textures, item models, shaders).
    pub files: Vec<OutputFile>,
    /// Non-fatal findings to surface to the user.
    pub warnings: Vec<String>,
    /// The typed runtime/codegen contract built from the authored project.
    pub manifest: Manifest,
}

/// Compile a TypeScript-authored Window project into pack artifacts.
///
/// Fails on the first error; warnings are accumulated in the output. Decoding
/// every `.png` is simpler than tracing references and harmless: unused decodes
/// are dropped.
pub fn compile_project(project: &ParsedProject, input: &CompileInput) -> Result<CompileOutput> {
    let textures = decode_textures(&input.files)?;

    let texture_size = |path: &str| -> Option<Size> { textures.get(path).map(|t| Size::new(t.width, t.height)) };
    let windows = crate::layout::solve(project, &texture_size)?;
    let huds = crate::layout::solve_huds(project, &texture_size)?;
    let runtime_sprites = runtime_sprite_assets(project, &textures, &input.namespace)?;

    let output = compile_layouts(
        &windows,
        &huds,
        &textures,
        &runtime_sprites,
        &input.namespace,
        &project.target,
        &project.options,
    )?;
    let validation = crate::validation::validate_compile_output_with_pack(&output, &input.files);
    if !validation.is_valid() {
        return Err(Error::Validation(validation.to_string()));
    }
    Ok(output)
}

/// Compile a project JSON payload produced by the plugin.
pub fn compile_project_json(project_json: &[u8], input: &CompileInput) -> Result<CompileOutput> {
    let project = crate::authoring::project_from_json(project_json)?;
    compile_project(&project, input)
}

fn decode_textures(files: &BTreeMap<String, Vec<u8>>) -> Result<BTreeMap<String, Texture>> {
    let mut textures = BTreeMap::new();
    for (path, bytes) in files {
        if path.to_ascii_lowercase().ends_with(".png") {
            let texture = Texture::decode_png(bytes)
                .map_err(|_| Error::Texture { path: path.clone(), message: "failed to decode PNG".into() })?;
            textures.insert(path.clone(), texture);
        }
    }
    Ok(textures)
}

#[derive(Clone, Debug)]
struct RuntimeSpriteAsset {
    size: Size,
    x_offset: u32,
    glyph_width: u32,
    advance: u32,
    file: String,
    output: Option<OutputFile>,
}

fn runtime_sprite_assets(
    project: &ParsedProject,
    textures: &BTreeMap<String, Texture>,
    namespace: &str,
) -> Result<BTreeMap<String, RuntimeSpriteAsset>> {
    let mut sprites = BTreeMap::new();
    for (name, sprite) in &project.theme.sprites {
        let asset = match sprite {
            SpriteDef::Texture { texture, size } if is_resource_texture_id(texture) => {
                let size = size.ok_or_else(|| {
                    Error::Validation(format!(
                        "sprite `{name}` uses external texture `{texture}` and must set `width` and `height`"
                    ))
                })?;
                let metrics = resource_texture_source_path(texture)
                    .and_then(|path| textures.get(&path))
                    .map(|decoded| bitmap_sprite_metrics(decoded, size))
                    .unwrap_or(RuntimeSpriteMetrics { x_offset: 0, glyph_width: size.width, advance: size.width + 1 });
                RuntimeSpriteAsset {
                    size,
                    x_offset: metrics.x_offset,
                    glyph_width: metrics.glyph_width,
                    advance: metrics.advance,
                    file: resource_texture_file(texture),
                    output: None,
                }
            }
            SpriteDef::Texture { texture, size } => {
                let decoded = textures.get(texture).ok_or_else(|| Error::Texture {
                    path: texture.clone(),
                    message: format!("referenced by runtime sprite `{name}` but not provided"),
                })?;
                let size = size.unwrap_or_else(|| Size::new(decoded.width, decoded.height));
                let metrics = bitmap_sprite_metrics(decoded, size);
                let file_stem = sprite_file_stem(name);
                RuntimeSpriteAsset {
                    size,
                    x_offset: metrics.x_offset,
                    glyph_width: metrics.glyph_width,
                    advance: metrics.advance,
                    file: format!("{namespace}:font/sprites/{file_stem}.png"),
                    output: Some(OutputFile {
                        path: format!("assets/{namespace}/textures/font/sprites/{file_stem}.png"),
                        contents: decoded.encode_png()?,
                    }),
                }
            }
            SpriteDef::Generated { size, style } => {
                let file_stem = sprite_file_stem(name);
                let texture = render_sprite(style, *size, name)?;
                let metrics = bitmap_sprite_metrics(&texture, *size);
                RuntimeSpriteAsset {
                    size: *size,
                    x_offset: metrics.x_offset,
                    glyph_width: metrics.glyph_width,
                    advance: metrics.advance,
                    file: format!("{namespace}:font/sprites/{file_stem}.png"),
                    output: Some(OutputFile {
                        path: format!("assets/{namespace}/textures/font/sprites/{file_stem}.png"),
                        contents: texture.encode_png()?,
                    }),
                }
            }
        };
        sprites.insert(name.clone(), asset);
    }
    Ok(sprites)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimeSpriteMetrics {
    x_offset: u32,
    glyph_width: u32,
    advance: u32,
}

fn bitmap_sprite_metrics(texture: &Texture, rendered_size: Size) -> RuntimeSpriteMetrics {
    let Some((left, right)) = opaque_x_bounds(texture) else {
        return RuntimeSpriteMetrics { x_offset: 0, glyph_width: 0, advance: 0 };
    };
    let source_height = texture.height.max(1);
    let x_offset = scale_sprite_boundary(left, source_height, rendered_size.height);
    let right_edge = scale_sprite_boundary(right + 1, source_height, rendered_size.height);
    RuntimeSpriteMetrics {
        x_offset,
        glyph_width: right_edge.saturating_sub(x_offset),
        advance: right_edge.saturating_add(1),
    }
}

fn scale_sprite_boundary(value: u32, source_height: u32, rendered_height: u32) -> u32 {
    let value = u128::from(value);
    let source_height = u128::from(source_height.max(1));
    let rendered_height = u128::from(rendered_height);
    ((value * rendered_height * 2 + source_height) / (source_height * 2)).min(u128::from(u32::MAX)) as u32
}

fn opaque_x_bounds(texture: &Texture) -> Option<(u32, u32)> {
    let mut left: Option<u32> = None;
    let mut right: Option<u32> = None;
    for y in 0..texture.height {
        for x in 0..texture.width {
            let alpha_index = ((y * texture.width + x) * 4 + 3) as usize;
            if texture.rgba.get(alpha_index).copied().unwrap_or(0) != 0 {
                left = Some(left.map_or(x, |current| current.min(x)));
                right = Some(right.map_or(x, |current| current.max(x)));
            }
        }
    }
    left.zip(right)
}

fn render_sprite(style: &GeneratedStyle, size: Size, name: &str) -> Result<Texture> {
    crate::raster::render(style, size)
        .map_err(|error| Error::Texture { path: name.to_string(), message: error.to_string() })
}

fn is_resource_texture_id(texture: &str) -> bool {
    texture.contains(':')
}

fn resource_texture_file(texture: &str) -> String {
    if texture.ends_with(".png") { texture.to_string() } else { format!("{texture}.png") }
}

fn resource_texture_source_path(texture: &str) -> Option<String> {
    let (namespace, path) = texture.split_once(':')?;
    let path = if path.ends_with(".png") { path.to_string() } else { format!("{path}.png") };
    Some(format!("assets/{namespace}/textures/{path}"))
}

fn sprite_file_stem(name: &str) -> String {
    name.replace('_', "/")
}

/// Compile laid-out windows and decoded textures into pack artifacts.
///
/// This is the pure backend: it performs no parsing, layout, or I/O. For every
/// window it composites the static layer, emits the composite PNG, allocates a
/// stable glyph codepoint, bakes the net-zero static string, and builds the
/// compiled slot/button entries. It then emits the main font and the shifted
/// label fonts. The typed definition is returned in [`CompileOutput::manifest`].
///
/// - `windows` are the laid-out windows (any order; processed deterministically
///   by name).
/// - `textures` maps pack-source-relative texture paths to decoded RGBA8.
/// - `namespace` is the resource namespace; it must match `^[a-z0-9_]+$`.
///
/// Output files are sorted by path. Per-window [`LaidOutWindow::warnings`] are
/// merged into [`CompileOutput::warnings`].
pub fn compile_windows(
    windows: &[LaidOutWindow],
    textures: &BTreeMap<String, Texture>,
    namespace: &str,
) -> Result<CompileOutput> {
    compile_layouts(
        windows,
        &[],
        textures,
        &BTreeMap::new(),
        namespace,
        &PackTarget::default(),
        &BuildOptions::default(),
    )
}

fn compile_layouts(
    windows: &[LaidOutWindow],
    huds: &[LaidOutHud],
    textures: &BTreeMap<String, Texture>,
    runtime_sprites: &BTreeMap<String, RuntimeSpriteAsset>,
    namespace: &str,
    target: &PackTarget,
    options: &BuildOptions,
) -> Result<CompileOutput> {
    validate_namespace(namespace)?;

    // Deterministic order: sort window references by name.
    let mut ordered: Vec<&LaidOutWindow> = windows.iter().collect();
    ordered.sort_by(|a, b| a.name.cmp(&b.name));
    let mut ordered_huds: Vec<&LaidOutHud> = huds.iter().collect();
    ordered_huds.sort_by(|a, b| a.name.cmp(&b.name));

    let mut window_composites = Vec::with_capacity(ordered.len());
    let mut hud_composites = Vec::with_capacity(ordered_huds.len());
    let uses_runtime_sprites = ordered.iter().any(|w| {
        !w.sprite_slots.is_empty()
            || w.buttons.iter().any(|button| button.states.values().any(|state| state.sprite.is_some()))
    });
    if uses_runtime_sprites && runtime_sprites.is_empty() {
        return Err(Error::Validation("runtime sprite slots require at least one theme sprite".into()));
    }

    // Stable codepoint allocation across the whole build: one key per static
    // glyph with static content, plus one key per runtime sprite glyph.
    let mut glyph_keys: BTreeSet<String> = BTreeSet::new();
    for w in &ordered {
        let comp = compose_window(w, textures)?;
        if comp.has_content {
            glyph_keys.insert(static_key(&w.name));
        }
        window_composites.push(comp);
    }
    for h in &ordered_huds {
        let comp = compose_draws(&h.draws, &h.name, textures)?;
        if comp.has_content {
            glyph_keys.insert(hud_static_key(&h.name));
        }
        hud_composites.push(comp);
    }
    if uses_runtime_sprites {
        for sprite in runtime_sprites.keys() {
            glyph_keys.insert(sprite_key(sprite));
        }
    }
    let codepoints = allocate(&glyph_keys);

    let mut files: Vec<OutputFile> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut bitmap_providers: Vec<serde_json::Value> = Vec::new();
    let mut shift_offsets: BTreeSet<i32> = BTreeSet::new();
    let mut sprite_offsets: BTreeSet<i32> = BTreeSet::new();
    let mut window_entries: BTreeMap<String, WindowEntry> = BTreeMap::new();
    let mut hud_entries: BTreeMap<String, HudEntry> = BTreeMap::new();
    let mut sprite_entries: BTreeMap<String, SpriteEntry> = BTreeMap::new();
    let hud_shader_markers = crate::hud::segment_markers(&ordered_huds);

    if uses_runtime_sprites {
        for (name, asset) in runtime_sprites {
            if let Some(output) = &asset.output {
                files.push(output.clone());
            }
            let glyph = sprite_glyph(name, &codepoints);
            sprite_entries.insert(
                name.clone(),
                SpriteEntry {
                    width: asset.size.width,
                    height: asset.size.height,
                    x_offset: asset.x_offset,
                    glyph_width: asset.glyph_width,
                    advance: asset.advance,
                    glyph: glyph.to_string(),
                },
            );
        }
    }

    for (w, comp) in ordered.iter().zip(&window_composites) {
        warnings.extend(w.warnings.iter().cloned());
        let title_origin = w.surface.title_origin();

        let static_text = if comp.has_content {
            // Ascent places the composite's top edge: A = title_y + 7 - top_y.
            let glyph_top_y = comp.bounds.y;
            let ascent = title_origin.y + 7 - glyph_top_y;
            let texture = provider_texture(comp, ascent, &w.name)?;
            let glyph_advance = bitmap_sprite_metrics(&texture, Size::new(texture.width, texture.height)).advance;
            let glyph = emit_static_glyph(
                &w.name,
                &texture,
                ascent,
                static_key(&w.name),
                &codepoints,
                &mut StaticGlyphSink { namespace, bitmap_providers: &mut bitmap_providers, files: &mut files },
            )?;

            crate::bake::bake_static_with_advance(glyph, &comp.bounds, title_origin, glyph_advance)
        } else {
            crate::bake::bake_empty()
        };

        // Slots (dynamic + static labels).
        let mut slots: BTreeMap<String, SlotEntry> = BTreeMap::new();
        for slot in &w.slots {
            let k = slot.rect.y - title_origin.y;
            shift_offsets.insert(k);
            let entry = slot_entry(namespace, slot, k, None);
            slots.insert(slot.name.clone(), entry);
        }

        let mut sprite_slots: BTreeMap<String, SpriteSlotEntry> = BTreeMap::new();
        for sprite_slot in &w.sprite_slots {
            let k = sprite_slot.rect.y - title_origin.y;
            sprite_offsets.insert(k);
            sprite_slots.insert(sprite_slot.name.clone(), sprite_slot_entry(namespace, sprite_slot, k));
            if let Some(sprite) = &sprite_slot.sprite {
                let asset = runtime_sprites.get(sprite).ok_or_else(|| {
                    Error::Validation(format!(
                        "window `{}`: sprite slot `{}` references unknown sprite `{sprite}`",
                        w.name, sprite_slot.name
                    ))
                })?;
                if asset.size.width > sprite_slot.rect.width || asset.size.height > sprite_slot.rect.height {
                    return Err(Error::Validation(format!(
                        "window `{}`: sprite `{sprite}` does not fit sprite slot `{}`",
                        w.name, sprite_slot.name
                    )));
                }
            }
        }

        // Inventory controls.
        let mut buttons: BTreeMap<String, ButtonEntry> = BTreeMap::new();
        let mut items: BTreeMap<String, ItemEntry> = BTreeMap::new();
        let mut collections: BTreeMap<String, CollectionEntry> = BTreeMap::new();
        let mut inputs: BTreeMap<String, AnvilInputEntry> = BTreeMap::new();
        let mut slot_rects: BTreeMap<String, SlotRectEntry> = BTreeMap::new();
        let mut claimed_inventory_slots: BTreeMap<InventorySlotRef, String> = BTreeMap::new();
        let Surface::Container(kind) = w.surface;
        for button in &w.buttons {
            let slot_refs = button.slots.clone().unwrap_or_else(|| kind.slot_refs_overlapping(&button.rect));
            if slot_refs.is_empty() {
                return Err(Error::Validation(format!(
                    "window `{}`: button `{}` overlaps no inventory slot",
                    w.name, button.name
                )));
            }
            // Click routes cover every backing slot; the button only fills the
            // slots it has not yielded to a repeater-cell item control.
            let fill_refs: Vec<InventorySlotRef> =
                slot_refs.iter().copied().filter(|slot| !button.yielded_slots.contains(slot)).collect();
            if fill_refs.is_empty() {
                return Err(Error::Validation(format!(
                    "window `{}`: button `{}` yielded every backing slot; leave at least one slot \
                     for the button's own hitbox item",
                    w.name, button.name
                )));
            }
            let fill_slots = claim_slots(&w.name, kind, &button.name, &fill_refs, &mut claimed_inventory_slots)?;
            let mut slots = Vec::with_capacity(slot_refs.len());
            for slot in &slot_refs {
                validate_slot_ref(&w.name, kind, &button.name, slot)?;
                slots.push((*slot).into());
            }
            let fill_slots = if fill_slots == slots { None } else { Some(fill_slots) };
            let has_visual_state = button.states.values().any(|state| state.sprite.is_some());
            let sprite_font = if has_visual_state {
                let k = button.rect.y - title_origin.y;
                sprite_offsets.insert(k);
                for (state_name, state) in &button.states {
                    let Some(sprite) = &state.sprite else {
                        continue;
                    };
                    let asset = runtime_sprites.get(sprite).ok_or_else(|| {
                        Error::Validation(format!(
                            "window `{}`: button `{}` state `{state_name}` references unknown sprite `{sprite}`",
                            w.name, button.name
                        ))
                    })?;
                    if asset.size.width > button.rect.width || asset.size.height > button.rect.height {
                        return Err(Error::Validation(format!(
                            "window `{}`: sprite `{sprite}` does not fit button `{}` state `{state_name}`",
                            w.name, button.name
                        )));
                    }
                }
                Some(format!("{namespace}:sprite_{}", shifted_suffix(k)))
            } else {
                None
            };
            buttons.insert(
                button.name.clone(),
                ButtonEntry {
                    x: button.rect.x,
                    y: button.rect.y,
                    width: button.rect.width,
                    height: button.rect.height,
                    slots,
                    fill_slots,
                    default: button.default.map(map_button_default),
                    action: button.action,
                    tooltip: button.tooltip.clone(),
                    states: button.states.clone(),
                    sprite_font,
                },
            );
        }
        for item in &w.items {
            let slots = claim_slots(&w.name, kind, &item.name, &item.slots, &mut claimed_inventory_slots)?;
            items.insert(item.name.clone(), ItemEntry { slots });
        }
        for collection in &w.collections {
            let slots = claim_slots(&w.name, kind, &collection.name, &collection.slots, &mut claimed_inventory_slots)?;
            collections.insert(collection.name.clone(), CollectionEntry { slots, action: collection.action });
        }
        for input in &w.inputs {
            let input_slot = InventorySlotRef::container(0);
            let slot = claim_slots(&w.name, kind, &input.name, &[input_slot], &mut claimed_inventory_slots)?
                .into_iter()
                .next()
                .expect("anvil input slot is valid");
            inputs.insert(
                input.name.clone(),
                AnvilInputEntry { slot, initial: input.initial.clone(), item_model: input.item_model.clone() },
            );
        }
        for slot_rect in &w.slot_rects {
            let slots = match slot_rect.claim {
                SlotRectClaim::None => Vec::new(),
                SlotRectClaim::All => {
                    claim_slots(&w.name, kind, &slot_rect.name, &slot_rect.slots, &mut claimed_inventory_slots)?
                }
                SlotRectClaim::Unowned => {
                    claim_unowned_slots(&w.name, kind, &slot_rect.name, &slot_rect.slots, &mut claimed_inventory_slots)?
                }
            };
            if !slots.is_empty() {
                slot_rects.insert(slot_rect.name.clone(), SlotRectEntry { slots });
            }
        }

        let groups = repeat_groups(w);

        window_entries.insert(
            w.name.clone(),
            WindowEntry {
                surface: surface_entry(&w.surface),
                static_text,
                slots,
                sprite_slots,
                buttons,
                items,
                collections,
                inputs,
                slot_rects,
                groups,
            },
        );
    }

    for (h, comp) in ordered_huds.iter().zip(&hud_composites) {
        warnings.extend(h.warnings.iter().cloned());

        let static_text = if comp.has_content {
            let ascent = 7 - comp.bounds.y;
            let texture = provider_texture(comp, ascent, &h.name)?;
            let glyph_advance = bitmap_sprite_metrics(&texture, Size::new(texture.width, texture.height)).advance;
            let file_stem = hud_file_stem(&h.name);
            let glyph = emit_static_glyph(
                &file_stem,
                &texture,
                ascent,
                hud_static_key(&h.name),
                &codepoints,
                &mut StaticGlyphSink { namespace, bitmap_providers: &mut bitmap_providers, files: &mut files },
            )?;

            if h.shader.is_some() {
                crate::bake::bake_static_with_advance(glyph, &comp.bounds, Point::new(0, 0), glyph_advance)
            } else {
                crate::bake::bake_fixed_width_static_with_advance(glyph, &comp.bounds, h.width, glyph_advance)
            }
        } else {
            if h.shader.is_some() { crate::bake::bake_empty() } else { crate::bake::bake_fixed_width_empty(h.width) }
        };

        let mut slots: BTreeMap<String, SlotEntry> = BTreeMap::new();
        for slot in &h.slots {
            let k = slot.rect.y;
            shift_offsets.insert(k);
            slots.insert(
                slot.name.clone(),
                slot_entry(namespace, slot, k, h.shader.map(|_| hud_shader_markers.slot_marker(&h.name, &slot.name))),
            );
        }

        hud_entries.insert(
            h.name.clone(),
            HudEntry {
                surface: HudSurfaceEntry {
                    kind: "hud".into(),
                    channel: h.channel.id().into(),
                    width: h.width,
                    height: h.height,
                },
                static_text,
                slots,
                shader: h.shader.map(|shader| shader_entry(shader, hud_shader_markers.static_marker(&h.name))),
            },
        );
    }

    // Main font: space provider, then every window bitmap provider (already in
    // window-name order because `ordered` is sorted and we pushed in order).
    let main = main_font(space_provider(), bitmap_providers);
    files.push(OutputFile { path: format!("assets/{namespace}/font/ui.json"), contents: to_json_bytes(&main)? });
    emit_hitbox_item(namespace, &mut files)?;

    // Shifted label fonts, one per distinct vertical offset.
    for k in &shift_offsets {
        let doc = shifted_font(*k)?;
        let suffix = shifted_suffix(*k);
        files.push(OutputFile {
            path: format!("assets/{namespace}/font/{suffix}.json"),
            contents: to_json_bytes(&doc)?,
        });
    }

    for k in &sprite_offsets {
        files.push(OutputFile {
            path: format!("assets/{namespace}/font/sprite_{}.json", shifted_suffix(*k)),
            contents: to_json_bytes(&sprite_font(*k, runtime_sprites, &codepoints)?)?,
        });
    }

    if options.hud_shaders {
        emit_hud_shader_files(target.pack_format, &ordered_huds, &mut files, &mut warnings);
    }

    // Manifest.
    let text_advances: BTreeMap<char, u32> = vanilla::advances().collect();
    let text_glyph_widths: BTreeMap<char, u32> = vanilla::glyph_widths().collect();
    let font_metrics = font_metrics(namespace, &shift_offsets, &text_advances, &text_glyph_widths);
    let manifest = Manifest {
        version: VERSION,
        namespace: namespace.to_string(),
        font: format!("{namespace}:ui"),
        spacers: spacer_table(),
        text_advances,
        text_glyph_widths,
        font_metrics,
        sprites: sprite_entries,
        windows: window_entries,
        huds: hud_entries,
    };
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let debug_descriptor = crate::debug::DebugDescriptor::build(&manifest, &files)?;
    files.push(OutputFile { path: crate::debug::output_path(namespace), contents: debug_descriptor.to_json_bytes()? });
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(CompileOutput { files, warnings, manifest })
}

/// The codepoint allocation key for a window's static composite.
fn static_key(window: &str) -> String {
    format!("window/{window}/static")
}

fn hud_static_key(hud: &str) -> String {
    format!("hud/{hud}/static")
}

fn sprite_key(sprite: &str) -> String {
    format!("sprite/{sprite}")
}

fn sprite_glyph(sprite: &str, codepoints: &BTreeMap<String, u32>) -> char {
    char::from_u32(codepoints[&sprite_key(sprite)]).expect("sprite codepoint is a valid char")
}

fn claim_slots(
    window: &str,
    kind: ContainerKind,
    owner: &str,
    slots: &[InventorySlotRef],
    claimed: &mut BTreeMap<InventorySlotRef, String>,
) -> Result<Vec<SlotRefEntry>> {
    let mut out = Vec::with_capacity(slots.len());
    for slot in slots {
        validate_slot_ref(window, kind, owner, slot)?;
        if let Some(existing) = claimed.insert(*slot, owner.to_string()) {
            return Err(Error::Validation(format!(
                "window `{window}`: control `{owner}` and control `{existing}` both own {} slot {}",
                slot_area_name(slot.area),
                slot.index
            )));
        }
        out.push((*slot).into());
    }
    Ok(out)
}

fn claim_unowned_slots(
    window: &str,
    kind: ContainerKind,
    owner: &str,
    slots: &[InventorySlotRef],
    claimed: &mut BTreeMap<InventorySlotRef, String>,
) -> Result<Vec<SlotRefEntry>> {
    let mut out = Vec::new();
    for slot in slots {
        validate_slot_ref(window, kind, owner, slot)?;
        if claimed.contains_key(slot) {
            continue;
        }
        claimed.insert(*slot, owner.to_string());
        out.push((*slot).into());
    }
    Ok(out)
}

fn repeat_groups(window: &LaidOutWindow) -> BTreeMap<String, RepeatGroupEntry> {
    let mut groups: BTreeMap<String, RepeatGroupEntry> = BTreeMap::new();
    for slot in &window.slots {
        if slot.text.is_some() {
            continue;
        }
        if let Some(repeat) = &slot.repeat
            && let Some(field) = &repeat.field
        {
            let group = repeat_group(&mut groups, repeat);
            set_indexed_name(&mut group.slots, field, repeat.index, slot.name.clone());
        }
    }
    for sprite_slot in &window.sprite_slots {
        if sprite_slot.sprite.is_some() {
            continue;
        }
        if let Some(repeat) = &sprite_slot.repeat
            && let Some(field) = &repeat.field
        {
            let group = repeat_group(&mut groups, repeat);
            set_indexed_name(&mut group.sprite_slots, field, repeat.index, sprite_slot.name.clone());
        }
    }
    for item in &window.items {
        if let Some(repeat) = &item.repeat
            && let Some(field) = &repeat.field
        {
            let group = repeat_group(&mut groups, repeat);
            set_indexed_name(&mut group.items, field, repeat.index, item.name.clone());
        }
    }
    for button in &window.buttons {
        if !button.action {
            continue;
        }
        if let Some(repeat) = &button.repeat
            && repeat.field.is_none()
        {
            let group = repeat_group(&mut groups, repeat);
            ensure_len(&mut group.buttons, repeat.index);
            group.buttons[repeat.index as usize] = button.name.clone();
        }
    }
    groups
}

fn repeat_group<'a>(
    groups: &'a mut BTreeMap<String, RepeatGroupEntry>,
    repeat: &RepeatBindingIr,
) -> &'a mut RepeatGroupEntry {
    let group = groups.entry(repeat.group.clone()).or_default();
    group.count = group.count.max(repeat.index + 1);
    group
}

fn set_indexed_name(fields: &mut BTreeMap<String, Vec<String>>, field: &str, index: u32, name: String) {
    let names = fields.entry(field.to_string()).or_default();
    ensure_len(names, index);
    names[index as usize] = name;
}

fn ensure_len(values: &mut Vec<String>, index: u32) {
    let len = index as usize + 1;
    if values.len() < len {
        values.resize(len, String::new());
    }
}

fn validate_slot_ref(window: &str, kind: ContainerKind, owner: &str, slot: &InventorySlotRef) -> Result<()> {
    match slot.area {
        InventorySlotArea::Container if slot.index >= kind.slot_count() => Err(Error::Validation(format!(
            "window `{window}`: control `{owner}` references container slot {}, but `{}` has only {} slots",
            slot.index,
            kind.id(),
            kind.slot_count()
        ))),
        InventorySlotArea::Player if slot.index >= 36 => Err(Error::Validation(format!(
            "window `{window}`: control `{owner}` references player slot {}, but generic container screens expose only player slots 0..35",
            slot.index
        ))),
        _ => Ok(()),
    }
}

fn slot_area_name(area: InventorySlotArea) -> &'static str {
    match area {
        InventorySlotArea::Container => "container",
        InventorySlotArea::Player => "player",
    }
}

fn provider_texture(comp: &Composite, ascent: i32, name: &str) -> Result<Texture> {
    if ascent <= comp.texture.height as i32 {
        return Ok(comp.texture.clone());
    }
    let height = ascent as u32;
    if height > 512 {
        return Err(Error::Font(format!(
            "window `{name}`: generated visual bleed requires bitmap height {height}, \
             exceeding the 512px provider limit"
        )));
    }

    let mut texture = Texture::transparent(comp.texture.width, height);
    let row_len = (comp.texture.width * 4) as usize;
    for y in 0..comp.texture.height as usize {
        let start = y * row_len;
        texture.rgba[start..start + row_len].copy_from_slice(&comp.texture.rgba[start..start + row_len]);
    }
    Ok(texture)
}

struct StaticGlyphSink<'a> {
    namespace: &'a str,
    bitmap_providers: &'a mut Vec<serde_json::Value>,
    files: &'a mut Vec<OutputFile>,
}

fn emit_static_glyph(
    file_base: &str,
    texture: &Texture,
    ascent: i32,
    key: String,
    codepoints: &BTreeMap<String, u32>,
    sink: &mut StaticGlyphSink<'_>,
) -> Result<char> {
    let glyph_cp = codepoints[&key];
    let glyph = char::from_u32(glyph_cp).expect("glyph codepoint is a valid char");
    let provider = bitmap_provider(sink.namespace, file_base, texture.height, ascent, glyph)?;
    sink.bitmap_providers.push(provider);
    sink.files.push(OutputFile {
        path: format!("assets/{}/textures/font/{file_base}.png", sink.namespace),
        contents: texture.encode_png()?,
    });
    Ok(glyph)
}

fn hud_file_stem(hud: &str) -> String {
    format!("hud_{hud}")
}

fn shader_entry(shader: HudShader, static_marker: Rgb) -> HudShaderEntry {
    HudShaderEntry {
        static_marker: static_marker.to_hex(),
        source_bottom: shader.source_bottom,
        origin_x: shader.origin_x,
        origin_y: shader.origin_y,
        anchor_x: shader.anchor_x,
        anchor_y: shader.anchor_y,
        offset_x: shader.offset_x,
        offset_y: shader.offset_y,
    }
}

fn emit_hud_shader_files(
    pack_format: Option<u32>,
    huds: &[&LaidOutHud],
    files: &mut Vec<OutputFile>,
    warnings: &mut Vec<String>,
) {
    let output = crate::hud::emit(pack_format, huds);
    files.extend(
        output.files.into_iter().map(|file| OutputFile { path: file.path, contents: file.contents.into_bytes() }),
    );
    warnings.extend(output.warnings);
}

/// Validate that `namespace` matches `^[a-z0-9_]+$`.
fn validate_namespace(namespace: &str) -> Result<()> {
    if !namespace.is_empty() && namespace.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') {
        Ok(())
    } else {
        Err(Error::Validation(format!("namespace `{namespace}` must match ^[a-z0-9_]+$")))
    }
}

/// Build a [`SlotEntry`] for `slot` at vertical offset `k`.
fn slot_entry(namespace: &str, slot: &SlotIr, k: i32, shader_marker: Option<Rgb>) -> SlotEntry {
    SlotEntry {
        x: slot.rect.x,
        y: slot.rect.y,
        width: slot.rect.width,
        align: slot.align,
        font: format!("{namespace}:{}", shifted_suffix(k)),
        color: slot.color.to_hex(),
        shader_marker: shader_marker.map(Rgb::to_hex),
        shader_color: None,
        shadow: slot.shadow,
        bold: slot.bold,
        italic: slot.italic,
        underlined: slot.underlined,
        strikethrough: slot.strikethrough,
        obfuscated: slot.obfuscated,
        text: slot.text.clone(),
    }
}

fn sprite_slot_entry(namespace: &str, slot: &SpriteSlotIr, k: i32) -> SpriteSlotEntry {
    SpriteSlotEntry {
        x: slot.rect.x,
        y: slot.rect.y,
        width: slot.rect.width,
        height: slot.rect.height,
        align: slot.align,
        font: format!("{namespace}:sprite_{}", shifted_suffix(k)),
        sprite: slot.sprite.clone(),
    }
}

fn sprite_font(
    k: i32,
    runtime_sprites: &BTreeMap<String, RuntimeSpriteAsset>,
    codepoints: &BTreeMap<String, u32>,
) -> Result<serde_json::Value> {
    let ascent = 7 - k;
    let mut providers = Vec::with_capacity(runtime_sprites.len());
    for (name, asset) in runtime_sprites {
        providers.push(bitmap_provider_file(
            &asset.file,
            &format!("sprite `{name}` at y offset {k}"),
            asset.size.height,
            ascent,
            sprite_glyph(name, codepoints),
        )?);
    }
    Ok(provider_font(providers))
}

fn font_metrics(
    namespace: &str,
    shift_offsets: &BTreeSet<i32>,
    text_advances: &BTreeMap<char, u32>,
    text_glyph_widths: &BTreeMap<char, u32>,
) -> BTreeMap<String, FontMetricsEntry> {
    let mut metrics = BTreeMap::new();
    let entry = FontMetricsEntry {
        advances: text_advances.clone(),
        glyph_widths: text_glyph_widths.clone(),
        bold_advance: vanilla::BOLD_ADVANCE,
    };
    metrics.insert("minecraft:default".into(), entry.clone());
    for k in shift_offsets {
        metrics.insert(format!("{namespace}:{}", shifted_suffix(*k)), entry.clone());
    }
    metrics
}

/// Build a [`SurfaceEntry`] for a surface.
fn surface_entry(surface: &Surface) -> SurfaceEntry {
    let Surface::Container(kind) = surface;
    let size = surface.gui_size();
    let origin = surface.title_origin();
    SurfaceEntry {
        kind: "container".into(),
        container: kind.id().to_string(),
        size: [size.width, size.height],
        title_origin: [origin.x, origin.y],
    }
}

/// Map an IR button default to a manifest button default. (Currently the
/// identity, but kept as a seam in case the variants diverge.)
fn map_button_default(d: ButtonDefault) -> ButtonDefault {
    d
}

fn emit_hitbox_item(namespace: &str, files: &mut Vec<OutputFile>) -> Result<()> {
    files.push(OutputFile {
        path: format!("assets/{namespace}/items/gui/hitbox.json"),
        contents: to_json_bytes(&serde_json::json!({
            "model": {
                "type": "minecraft:model",
                "model": format!("{namespace}:gui/hitbox"),
            },
        }))?,
    });
    files.push(OutputFile {
        path: format!("assets/{namespace}/models/gui/hitbox.json"),
        contents: to_json_bytes(&serde_json::json!({
            "parent": "minecraft:item/generated",
            "textures": {
                "layer0": "minecraft:item/barrier",
            },
            "display": {
                "gui": {
                    "scale": [0, 0, 0],
                },
            },
        }))?,
    });
    files.push(OutputFile {
        path: format!("assets/{namespace}/textures/gui/hitbox.png"),
        contents: Texture { width: 1, height: 1, rgba: vec![0, 0, 0, 0] }.encode_png()?,
    });
    Ok(())
}

/// Serialize a JSON value deterministically (pretty, trailing newline).
fn to_json_bytes(value: &serde_json::Value) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| Error::Font(e.to_string()))?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::geometry::{Insets, Rect, Size};
    use crate::inventory::SlotRectClaim;
    use crate::ir::{Align, ButtonIr, ButtonState, ButtonTooltip, Draw, Rgb, SlotRectIr, SpriteSlotIr, TextureKey};
    use crate::surface::ContainerKind;

    /// A solid RGBA texture for tests.
    fn solid(width: u32, height: u32, rgba: [u8; 4]) -> Texture {
        Texture { width, height, rgba: rgba.iter().copied().cycle().take((width * height * 4) as usize).collect() }
    }

    fn with_transparent_right_edge(mut texture: Texture, columns: u32) -> Texture {
        for y in 0..texture.height {
            for x in texture.width.saturating_sub(columns)..texture.width {
                let i = ((y * texture.width + x) * 4 + 3) as usize;
                texture.rgba[i] = 0;
            }
        }
        texture
    }

    #[test]
    fn runtime_sprite_metrics_match_minecraft_alpha_trimmed_width() {
        let full = solid(12, 12, [255, 255, 255, 255]);
        let trimmed = with_transparent_right_edge(full.clone(), 2);

        assert_eq!(
            bitmap_sprite_metrics(&full, Size::new(12, 12)),
            RuntimeSpriteMetrics { x_offset: 0, glyph_width: 12, advance: 13 }
        );
        assert_eq!(
            bitmap_sprite_metrics(&trimmed, Size::new(12, 12)),
            RuntimeSpriteMetrics { x_offset: 0, glyph_width: 10, advance: 11 }
        );
    }

    #[test]
    fn runtime_sprite_advance_rounds_scaled_trimmed_width_like_minecraft() {
        let texture = with_transparent_right_edge(solid(10, 10, [255, 255, 255, 255]), 1);

        assert_eq!(bitmap_sprite_metrics(&texture, Size::new(10, 8)).advance, 8);
    }

    #[test]
    fn runtime_sprite_metrics_keep_left_padding_separate_from_visible_width() {
        let mut texture = with_transparent_right_edge(solid(9, 9, [255, 255, 255, 255]), 1);
        for y in 0..texture.height {
            let i = (y * texture.width * 4 + 3) as usize;
            texture.rgba[i] = 0;
        }

        assert_eq!(
            bitmap_sprite_metrics(&texture, Size::new(9, 9)),
            RuntimeSpriteMetrics { x_offset: 1, glyph_width: 7, advance: 9 }
        );
    }

    #[test]
    fn resource_runtime_sprite_uses_decoded_pack_texture_metrics_when_available() {
        let mut texture = solid(32, 32, [0, 0, 0, 0]);
        for y in 5..=28 {
            for x in 7..=25 {
                let i = ((y * texture.width + x) * 4) as usize;
                texture.rgba[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
        let mut textures = BTreeMap::new();
        textures.insert("assets/example/textures/backpack/galactic_bucket.png".into(), texture);
        let mut project = ParsedProject::default();
        project.theme.sprites.insert(
            "backpack_galactic_bucket".into(),
            SpriteDef::Texture {
                texture: "example:backpack/galactic_bucket.png".into(),
                size: Some(Size::new(18, 18)),
            },
        );

        let sprites = runtime_sprite_assets(&project, &textures, "window").unwrap();
        let sprite = sprites.get("backpack_galactic_bucket").unwrap();

        assert_eq!(sprite.file, "example:backpack/galactic_bucket.png");
        assert_eq!(sprite.output, None);
        assert_eq!(sprite.x_offset, 4);
        assert_eq!(sprite.glyph_width, 11);
        assert_eq!(sprite.advance, 16);
    }

    /// A window with one nine-slice draw, one label slot, one dynamic slot, and
    /// one button over container slots.
    fn sample_window() -> (LaidOutWindow, BTreeMap<String, Texture>) {
        let mut textures = BTreeMap::new();
        textures.insert("frame.png".to_string(), solid(8, 8, [200, 200, 200, 255]));

        let window = LaidOutWindow {
            name: "shop".into(),
            surface: Surface::Container(ContainerKind::Generic9x6),
            draws: vec![Draw::NineSlice {
                texture: TextureKey("frame.png".into()),
                insets: Insets::uniform(2),
                dest: Rect::new(0, 0, 176, 16),
            }],
            slots: vec![
                SlotIr {
                    name: "title".into(),
                    text: None,
                    rect: Rect::new(8, 6, 160, 8),
                    align: Align::Center,
                    color: Rgb::DEFAULT_TEXT,
                    shadow: false,
                    bold: false,
                    italic: false,
                    underlined: false,
                    strikethrough: false,
                    obfuscated: false,
                    repeat: None,
                },
                SlotIr {
                    name: "buy_label".into(),
                    text: Some("Buy".into()),
                    rect: Rect::new(58, 30, 60, 8),
                    align: Align::Center,
                    color: Rgb { r: 0xff, g: 0xff, b: 0xff },
                    shadow: false,
                    bold: false,
                    italic: false,
                    underlined: false,
                    strikethrough: false,
                    obfuscated: false,
                    repeat: None,
                },
            ],
            sprite_slots: vec![],
            buttons: vec![ButtonIr {
                name: "buy".into(),
                // Over container slots: rect at (8,18) covers slot 0.
                rect: Rect::new(8, 18, 16, 16),
                slots: None,
                yielded_slots: Vec::new(),
                default: Some(ButtonDefault::Close),
                action: true,
                tooltip: Some(ButtonTooltip { title: "Buy".into(), lines: vec!["Spend coins".into()] }),
                states: BTreeMap::from([(
                    "disabled".into(),
                    ButtonState { item_model: Some("demo:gui/buy_disabled".into()), sprite: None, tooltip: None },
                )]),
                repeat: None,
            }],
            items: vec![],
            collections: vec![],
            inputs: vec![],
            slot_rects: vec![],
            warnings: vec!["overlay overflow".into()],
        };
        (window, textures)
    }

    fn find<'a>(out: &'a CompileOutput, path: &str) -> &'a OutputFile {
        out.files.iter().find(|f| f.path == path).unwrap_or_else(|| panic!("missing output file {path}"))
    }

    #[test]
    fn emits_expected_paths() {
        let (w, textures) = sample_window();
        let out = compile_windows(&[w], &textures, "window").unwrap();
        let paths: Vec<&str> = out.files.iter().map(|f| f.path.as_str()).collect();
        // Two distinct slot offsets: title y=6 (k=0), label y=30 (k=24).
        assert_eq!(
            paths,
            vec![
                "assets/window/font/ui.json",
                "assets/window/font/y0.json",
                "assets/window/font/y24.json",
                "assets/window/items/gui/hitbox.json",
                "assets/window/models/gui/hitbox.json",
                "assets/window/textures/font/shop.png",
                "assets/window/textures/gui/hitbox.png",
                "assets/window/window/debug.json",
            ]
        );
        // Sorted by path.
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted);
        // Window warning propagated.
        assert_eq!(out.warnings, vec!["overlay overflow".to_string()]);
    }

    #[test]
    fn runtime_sprites_are_duplicated_by_y_font_not_texture() {
        let mut textures = BTreeMap::new();
        textures.insert("frame.png".to_string(), solid(8, 8, [200, 200, 200, 255]));
        let (mut w, _) = sample_window();
        w.buttons[0].states.get_mut("disabled").unwrap().sprite = Some("pickaxe".into());
        w.sprite_slots = vec![
            SpriteSlotIr {
                name: "card_icon".into(),
                rect: Rect::new(20, 6, 20, 16),
                align: Align::Center,
                sprite: None,
                repeat: None,
            },
            SpriteSlotIr {
                name: "detail_icon".into(),
                rect: Rect::new(80, 24, 20, 16),
                align: Align::Center,
                sprite: None,
                repeat: None,
            },
        ];
        let mut runtime_sprites = BTreeMap::new();
        runtime_sprites.insert(
            "pickaxe".into(),
            RuntimeSpriteAsset {
                size: Size::new(16, 16),
                x_offset: 0,
                glyph_width: 16,
                advance: 17,
                file: "window:font/sprites/pickaxe.png".into(),
                output: Some(OutputFile {
                    path: "assets/window/textures/font/sprites/pickaxe.png".into(),
                    contents: solid(16, 16, [255, 128, 0, 255]).encode_png().unwrap(),
                }),
            },
        );

        let out = compile_layouts(
            &[w],
            &[],
            &textures,
            &runtime_sprites,
            "window",
            &PackTarget::default(),
            &BuildOptions::default(),
        )
        .unwrap();
        let paths: Vec<&str> = out.files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"assets/window/font/sprite_y0.json"));
        assert!(paths.contains(&"assets/window/font/sprite_y18.json"));
        assert!(paths.contains(&"assets/window/font/sprite_y12.json"));
        assert_eq!(paths.iter().filter(|path| **path == "assets/window/textures/font/sprites/pickaxe.png").count(), 1);

        for path in ["assets/window/font/sprite_y0.json", "assets/window/font/sprite_y18.json"] {
            let doc: serde_json::Value = serde_json::from_slice(&find(&out, path).contents).unwrap();
            let provider = &doc["providers"].as_array().unwrap()[0];
            assert_eq!(provider["file"], "window:font/sprites/pickaxe.png");
            assert_eq!(provider["chars"][0].as_str().unwrap(), out.manifest.sprites["pickaxe"].glyph);
        }
        assert_eq!(out.manifest.windows["shop"].sprite_slots["card_icon"].font, "window:sprite_y0");
        assert_eq!(out.manifest.windows["shop"].sprite_slots["detail_icon"].font, "window:sprite_y18");
        assert_eq!(out.manifest.windows["shop"].buttons["buy"].sprite_font.as_deref(), Some("window:sprite_y12"));
        let descriptor =
            crate::debug::DebugDescriptor::from_json(&find(&out, "assets/window/window/debug.json").contents).unwrap();
        assert_eq!(descriptor.sprites["pickaxe"].resource.as_deref(), Some("window:font/sprites/pickaxe.png"));
    }

    #[test]
    fn bitmap_provider_ascent_math() {
        // Glyph top at y=0, title y=6 → ascent 13, height 16.
        let (w, textures) = sample_window();
        let out = compile_windows(&[w], &textures, "window").unwrap();
        let ui: serde_json::Value = serde_json::from_slice(&find(&out, "assets/window/font/ui.json").contents).unwrap();
        let providers = ui["providers"].as_array().unwrap();
        // [0] = space, [1] = shop bitmap glyph.
        assert_eq!(providers[0]["type"], "space");
        let bm = &providers[1];
        assert_eq!(bm["type"], "bitmap");
        assert_eq!(bm["file"], "window:font/shop.png");
        assert_eq!(bm["height"], 16);
        assert_eq!(bm["ascent"], 13);
    }

    #[test]
    fn static_bake_uses_alpha_trimmed_minecraft_advance() {
        let mut textures = BTreeMap::new();
        textures.insert("trimmed.png".to_string(), with_transparent_right_edge(solid(16, 8, [255, 255, 255, 255]), 6));
        let window = LaidOutWindow {
            name: "trimmed".into(),
            surface: Surface::Container(ContainerKind::Generic9x3),
            draws: vec![Draw::Sprite { texture: TextureKey("trimmed.png".into()), dest: Rect::new(8, 6, 16, 8) }],
            slots: vec![],
            sprite_slots: vec![],
            buttons: vec![],
            items: vec![],
            collections: vec![],
            inputs: vec![],
            slot_rects: vec![],
            warnings: vec![],
        };

        let output = compile_windows(&[window], &textures, "window").unwrap();
        crate::validation::validate_compile_output(&output).assert_valid();
    }

    #[test]
    fn negative_visual_top_pads_bitmap_provider_height() {
        let mut textures = BTreeMap::new();
        textures.insert("cap.png".to_string(), solid(4, 4, [255, 128, 0, 255]));
        let w = LaidOutWindow {
            name: "overhang".into(),
            surface: Surface::Container(ContainerKind::Generic9x3),
            draws: vec![Draw::Sprite { texture: TextureKey("cap.png".into()), dest: Rect::new(0, -20, 4, 4) }],
            slots: vec![],
            sprite_slots: vec![],
            buttons: vec![],
            items: vec![],
            collections: vec![],
            inputs: vec![],
            slot_rects: vec![],
            warnings: vec![],
        };

        let out = compile_windows(&[w], &textures, "window").unwrap();
        let ui: serde_json::Value = serde_json::from_slice(&find(&out, "assets/window/font/ui.json").contents).unwrap();
        let bm = &ui["providers"].as_array().unwrap()[1];
        assert_eq!(bm["height"], 33);
        assert_eq!(bm["ascent"], 33);
        let png = &find(&out, "assets/window/textures/font/overhang.png").contents;
        let texture = Texture::decode_png(png).unwrap();
        assert_eq!(texture.height, 33);
        assert_eq!(&texture.rgba[0..4], &[255, 128, 0, 255]);
    }

    #[test]
    fn manifest_parses_and_matches() {
        let (w, textures) = sample_window();
        let out = compile_windows(&[w], &textures, "window").unwrap();
        let m = &out.manifest;
        assert_eq!(m.version, VERSION);
        assert_eq!(m.namespace, "window");
        assert_eq!(m.font, "window:ui");
        assert_eq!(m.spacers.len(), 22);
        // Text metric fields exist (table contents come from vanilla, stubbed
        // here — only assert they are present / maps).
        let _ = &m.text_advances;
        let _ = &m.text_glyph_widths;
        assert_eq!(m.font_metrics["minecraft:default"].bold_advance, 1);
        assert_eq!(m.font_metrics["window:y0"].advances[&'H'], vanilla::text_width("H"));

        let shop = &m.windows["shop"];
        assert_eq!(shop.surface.container, "generic_9x6");
        assert_eq!(shop.surface.size, [176, 222]);
        assert_eq!(shop.surface.title_origin, [8, 6]);
        assert!(!shop.static_text.is_empty());

        // Slots.
        let title = &shop.slots["title"];
        assert_eq!(title.font, "window:y0");
        assert_eq!(title.color, "#404040");
        assert_eq!(title.align, Align::Center);
        assert!(title.text.is_none());
        let label = &shop.slots["buy_label"];
        assert_eq!(label.font, "window:y24");
        assert_eq!(label.color, "#ffffff");
        assert_eq!(label.text.as_deref(), Some("Buy"));

        // Button mapped to container slot 0.
        let buy = &shop.buttons["buy"];
        assert_eq!(buy.slots, vec![InventorySlotRef::container(0).into()]);
        assert_eq!(buy.default, Some(ButtonDefault::Close));
        assert!(buy.action);
        assert_eq!(buy.tooltip.as_ref().unwrap().title, "Buy");
        assert_eq!(buy.states["disabled"].item_model.as_deref(), Some("demo:gui/buy_disabled"));
    }

    #[test]
    fn unowned_slot_rect_claims_skip_existing_controls() {
        let (mut w, textures) = sample_window();
        w.slot_rects.push(SlotRectIr {
            name: "fill".into(),
            slots: vec![InventorySlotRef::container(0), InventorySlotRef::container(1), InventorySlotRef::player(0)],
            claim: SlotRectClaim::Unowned,
        });

        let out = compile_windows(&[w], &textures, "window").unwrap();
        let fill = &out.manifest.windows["shop"].slot_rects["fill"];
        assert_eq!(fill.slots, vec![InventorySlotRef::container(1).into(), InventorySlotRef::player(0).into(),]);
    }

    #[test]
    fn output_is_byte_identical_across_runs() {
        let (w1, t1) = sample_window();
        let (w2, t2) = sample_window();
        let a = compile_windows(&[w1], &t1, "window").unwrap();
        let b = compile_windows(&[w2], &t2, "window").unwrap();
        assert_eq!(a.files, b.files);
    }

    #[test]
    fn button_over_no_slots_errors() {
        let (mut w, textures) = sample_window();
        // Move the button below the container and player grids (no overlap).
        w.buttons[0].rect = Rect::new(52, 216, 72, 20);
        let err = compile_windows(&[w], &textures, "window").unwrap_err();
        match err {
            Error::Validation(msg) => {
                assert!(msg.contains("shop") && msg.contains("buy"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn overlapping_buttons_error() {
        let (mut w, textures) = sample_window();
        w.buttons.push(ButtonIr {
            name: "other".into(),
            rect: Rect::new(8, 18, 16, 16),
            slots: None,
            yielded_slots: Vec::new(),
            default: None,
            action: true,
            tooltip: None,
            states: BTreeMap::new(),
            repeat: None,
        });

        let err = compile_windows(&[w], &textures, "window").unwrap_err();
        match err {
            Error::Validation(msg) => {
                assert!(msg.contains("both own container slot 0"), "{msg}");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn oversized_composite_height_errors() {
        let mut textures = BTreeMap::new();
        textures.insert("big.png".to_string(), solid(4, 4, [1, 1, 1, 255]));
        // A nine-slice stretched to a 600px-tall composite exceeds the 512px
        // provider limit.
        let w = LaidOutWindow {
            name: "tall".into(),
            surface: Surface::Container(ContainerKind::Generic9x6),
            draws: vec![Draw::NineSlice {
                texture: TextureKey("big.png".into()),
                insets: Insets::uniform(1),
                dest: Rect::new(0, 0, 4, 600),
            }],
            slots: vec![],
            sprite_slots: vec![],
            buttons: vec![],
            items: vec![],
            collections: vec![],
            inputs: vec![],
            slot_rects: vec![],
            warnings: vec![],
        };
        let err = compile_windows(&[w], &textures, "window").unwrap_err();
        assert!(matches!(err, Error::Font(_)));
    }

    #[test]
    fn empty_draws_produce_empty_static_and_no_png() {
        let textures = BTreeMap::new();
        let w = LaidOutWindow {
            name: "bare".into(),
            surface: Surface::Container(ContainerKind::Generic9x3),
            draws: vec![],
            slots: vec![SlotIr {
                name: "title".into(),
                text: None,
                rect: Rect::new(8, 6, 100, 8),
                align: Align::Left,
                color: Rgb::DEFAULT_TEXT,
                shadow: false,
                bold: false,
                italic: false,
                underlined: false,
                strikethrough: false,
                obfuscated: false,
                repeat: None,
            }],
            sprite_slots: vec![],
            buttons: vec![],
            items: vec![],
            collections: vec![],
            inputs: vec![],
            slot_rects: vec![],
            warnings: vec![],
        };
        let out = compile_windows(&[w], &textures, "window").unwrap();
        // No composite PNG emitted.
        assert!(!out.files.iter().any(|f| f.path == "assets/window/textures/font/bare.png"));
        assert_eq!(out.manifest.windows["bare"].static_text, "");
        // ui.json has only the space provider (no bitmap).
        let ui: serde_json::Value = serde_json::from_slice(&find(&out, "assets/window/font/ui.json").contents).unwrap();
        assert_eq!(ui["providers"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn invalid_namespace_errors() {
        let (w, textures) = sample_window();
        let err = compile_windows(&[w], &textures, "Bad NS").unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[test]
    fn static_glyph_codepoint_in_private_use_area() {
        let (w, textures) = sample_window();
        let out = compile_windows(&[w], &textures, "window").unwrap();
        let glyph = out.manifest.windows["shop"].static_text.chars().find(|&c| {
            let cp = c as u32;
            (crate::font::GLYPH_BASE..crate::font::GLYPH_END).contains(&cp)
        });
        assert!(glyph.is_some(), "static string contains a glyph codepoint");
    }
}
