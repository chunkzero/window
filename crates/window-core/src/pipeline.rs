//! The engine's entry point: bytes in, pack artifacts out.
//!
//! Orchestrates authored model → layout → compose → font → bake → compiled
//! definition. This is the only API the rpp plugin (or any other host) needs.
//!
//! The public entry points decode textures and solve layout, then hand the
//! laid-out IR to the backend, which runs in stages: compose static layers and
//! allocate glyph codepoints, compile each window then each HUD, emit fonts and
//! HUD shaders, and build the manifest and debug descriptor.

use std::collections::{BTreeMap, BTreeSet};

use crate::authoring::{BuildOptions, PackTarget, ParsedProject};
use crate::compose::Texture;
use crate::font::spacer_table;
use crate::geometry::Size;
use crate::ir::{LaidOutHud, LaidOutWindow};
use crate::manifest::{HudEntry, Manifest, SpriteEntry, VERSION, WindowEntry};
use crate::{Error, Result, vanilla};

mod fonts;
mod glyphs;
mod hud;
mod inventory;
mod metrics;
mod sprites;
mod window;

#[cfg(test)]
mod tests;

use sprites::RuntimeSpriteAsset;

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
    let runtime_sprites = sprites::runtime_sprite_assets(project, &textures, &input.namespace)?;

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

/// Compile laid-out windows and HUDs into pack artifacts. Windows and HUDs are
/// processed by name; output files are sorted by path.
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
    let mut windows: Vec<&LaidOutWindow> = windows.iter().collect();
    windows.sort_by(|a, b| a.name.cmp(&b.name));
    let mut huds: Vec<&LaidOutHud> = huds.iter().collect();
    huds.sort_by(|a, b| a.name.cmp(&b.name));

    let uses_runtime_sprites = uses_runtime_sprites(&windows);
    if uses_runtime_sprites && runtime_sprites.is_empty() {
        return Err(Error::Validation("runtime sprite slots require at least one theme sprite".into()));
    }
    let composites = glyphs::compose_layers(&windows, &huds, textures)?;
    let used_sprites = uses_runtime_sprites.then_some(runtime_sprites);
    let codepoints = glyphs::allocate_codepoints(&windows, &huds, &composites, used_sprites)?;
    let mut ctx = CompileContext::new(namespace, runtime_sprites, codepoints);

    let sprites = if uses_runtime_sprites { sprites::sprite_entries(&mut ctx) } else { BTreeMap::new() };
    let mut window_entries = BTreeMap::new();
    for (w, comp) in windows.iter().zip(&composites.windows) {
        window_entries.insert(w.name.clone(), window::compile_window(&mut ctx, w, comp)?);
    }
    let markers = crate::hud::segment_markers(&huds);
    let mut hud_entries = BTreeMap::new();
    for (h, comp) in huds.iter().zip(&composites.huds) {
        hud_entries.insert(h.name.clone(), hud::compile_hud(&mut ctx, h, comp, &markers)?);
    }

    fonts::emit_fonts(&mut ctx)?;
    if options.hud_shaders {
        hud::emit_shader_files(&mut ctx, target.pack_format, &huds)?;
    }
    let manifest = ctx.manifest(sprites, window_entries, hud_entries);
    ctx.finish(manifest)
}

fn uses_runtime_sprites(windows: &[&LaidOutWindow]) -> bool {
    windows.iter().any(|w| {
        !w.sprite_slots.is_empty()
            || w.buttons.iter().any(|button| button.states.values().any(|state| state.sprite.is_some()))
    })
}

/// Validate that `namespace` matches `^[a-z0-9_]+$`.
fn validate_namespace(namespace: &str) -> Result<()> {
    if !namespace.is_empty() && namespace.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') {
        Ok(())
    } else {
        Err(Error::Validation(format!("namespace `{namespace}` must match ^[a-z0-9_]+$")))
    }
}

/// Build-wide state shared by the compile stages. Static bitmap providers are
/// pushed in compile order (windows, then HUDs), which fixes the main font's
/// provider order.
struct CompileContext<'a> {
    namespace: &'a str,
    runtime_sprites: &'a BTreeMap<String, RuntimeSpriteAsset>,
    codepoints: BTreeMap<String, u32>,
    files: Vec<OutputFile>,
    warnings: Vec<String>,
    bitmap_providers: Vec<serde_json::Value>,
    shift_offsets: BTreeSet<i32>,
    sprite_offsets: BTreeSet<i32>,
}

impl<'a> CompileContext<'a> {
    fn new(
        namespace: &'a str,
        runtime_sprites: &'a BTreeMap<String, RuntimeSpriteAsset>,
        codepoints: BTreeMap<String, u32>,
    ) -> Self {
        Self {
            namespace,
            runtime_sprites,
            codepoints,
            files: Vec::new(),
            warnings: Vec::new(),
            bitmap_providers: Vec::new(),
            shift_offsets: BTreeSet::new(),
            sprite_offsets: BTreeSet::new(),
        }
    }

    fn manifest(
        &self,
        sprites: BTreeMap<String, SpriteEntry>,
        windows: BTreeMap<String, WindowEntry>,
        huds: BTreeMap<String, HudEntry>,
    ) -> Manifest {
        let namespace = self.namespace;
        let text_advances: BTreeMap<char, u32> = vanilla::advances().collect();
        let text_glyph_widths: BTreeMap<char, u32> = vanilla::glyph_widths().collect();
        let font_metrics = fonts::font_metrics(namespace, &self.shift_offsets, &text_advances, &text_glyph_widths);
        Manifest {
            version: VERSION,
            namespace: namespace.to_string(),
            font: format!("{namespace}:ui"),
            spacers: spacer_table(),
            text_advances,
            text_glyph_widths,
            font_metrics,
            sprites,
            windows,
            huds,
        }
    }

    /// Sort the files and append the debug descriptor built over them.
    fn finish(self, manifest: Manifest) -> Result<CompileOutput> {
        let mut files = self.files;
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let debug_descriptor = crate::debug::DebugDescriptor::build(&manifest, &files)?;
        files.push(OutputFile {
            path: crate::debug::output_path(self.namespace),
            contents: debug_descriptor.to_json_bytes()?,
        });
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(CompileOutput { files, warnings: self.warnings, manifest })
    }
}
