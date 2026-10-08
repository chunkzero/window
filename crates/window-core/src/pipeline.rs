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
use crate::font::{shifted_suffix, spacer_table, text_font_suffix};
use crate::geometry::Size;
use crate::ir::{LaidOutHud, LaidOutWindow, SlotIr};
use crate::manifest::{HudEntry, Manifest, SpriteEntry, VERSION, WindowEntry};
use crate::text_font::TextFonts;
use crate::{Error, Result, text_font, vanilla};

mod anvil;
mod fonts;
mod glyphs;
mod hud;
mod inventory;
mod metrics;
mod sprites;
mod switches;
mod window;

#[cfg(test)]
mod tests;

use sprites::RuntimeSpriteAsset;
pub(crate) use sprites::texture_source_path;

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
    pub contents: FileContents,
}

impl OutputFile {
    pub fn text(path: impl Into<String>, contents: String) -> Self {
        Self { path: path.into(), contents: FileContents::Text(contents) }
    }

    pub fn binary(path: impl Into<String>, contents: Vec<u8>) -> Self {
        Self { path: path.into(), contents: FileContents::Binary(contents) }
    }
}

/// Output file contents, kept as text when the compiler generates text so hosts can skip byte handling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileContents {
    Text(String),
    Binary(Vec<u8>),
}

impl FileContents {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Text(text) => text.as_bytes(),
            Self::Binary(bytes) => bytes,
        }
    }
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
    let text_fonts = text_font::resolve(&project.fonts, &textures)?;
    let windows = crate::layout::solve(project, &texture_size, &text_fonts)?;
    let huds = crate::layout::solve_huds(project, &texture_size, &text_fonts)?;
    let runtime_sprites = sprites::runtime_sprite_assets(project, &textures, &input.namespace)?;

    let assets = Assets { textures: &textures, runtime_sprites: &runtime_sprites, text_fonts: &text_fonts };
    let output = compile_layouts(&windows, &huds, &assets, &input.namespace, &project.target, &project.options)?;
    let validation = crate::validation::validate_compiled(&output, &input.files);
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

/// Decoded assets the backend draws laid-out windows and HUDs with.
struct Assets<'a> {
    textures: &'a BTreeMap<String, Texture>,
    runtime_sprites: &'a BTreeMap<String, RuntimeSpriteAsset>,
    text_fonts: &'a TextFonts,
}

/// Compile laid-out windows and HUDs into pack artifacts. Windows and HUDs are
/// processed by name; output files are sorted by path.
fn compile_layouts(
    windows: &[LaidOutWindow],
    huds: &[LaidOutHud],
    assets: &Assets<'_>,
    namespace: &str,
    target: &PackTarget,
    options: &BuildOptions,
) -> Result<CompileOutput> {
    validate_namespace(namespace)?;
    let mut windows: Vec<&LaidOutWindow> = windows.iter().collect();
    windows.sort_by(|a, b| a.name.cmp(&b.name));
    let mut huds: Vec<&LaidOutHud> = huds.iter().collect();
    huds.sort_by(|a, b| a.name.cmp(&b.name));
    if !options.hud_shaders
        && let Some(hud) = huds.iter().find(|h| h.shader.is_some())
    {
        return Err(Error::Validation(format!(
            "HUD `{}` uses `shader` placement, but hudShaders is disabled; set `hudShaders: true` in the Window \
             plugin options or remove the HUD `shader` placement",
            hud.name
        )));
    }

    let Assets { textures, runtime_sprites, text_fonts } = *assets;
    let uses_runtime_sprites = uses_runtime_sprites(&windows);
    if uses_runtime_sprites && runtime_sprites.is_empty() {
        return Err(Error::Validation("runtime sprite slots require at least one catalog sprite".into()));
    }
    let mut composites = glyphs::compose_layers(&windows, &huds, textures)?;
    let mut field_warnings = Vec::new();
    for (w, layers) in windows.iter().zip(&mut composites.windows) {
        anvil::check_title(w, options.experimental_anvil_updates, &mut field_warnings)?;
        anvil::open_field(w, layers, &mut field_warnings);
    }
    let used_sprites = uses_runtime_sprites.then_some(runtime_sprites);
    let codepoints = glyphs::allocate_codepoints(&windows, &huds, &composites, used_sprites)?;
    let mut ctx = CompileContext::new(namespace, runtime_sprites, text_fonts, codepoints);
    ctx.warnings.extend(field_warnings);
    if windows.iter().any(|w| !w.inputs.is_empty()) {
        ctx.files.extend(anvil::hidden_vanilla_art()?);
    }
    if let Some(sprite) = &options.anvil_field_sprite {
        ctx.files.extend(anvil::field_sprites(runtime_sprites, sprite)?);
    }

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
        !w.sprite_slots.is_empty() || w.collections.iter().any(|collection| collection.selected_sprite.is_some())
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
    text_fonts: &'a TextFonts,
    codepoints: BTreeMap<String, u32>,
    files: Vec<OutputFile>,
    warnings: Vec<String>,
    bitmap_providers: Vec<serde_json::Value>,
    shift_offsets: BTreeSet<i32>,
    text_font_offsets: BTreeSet<(&'a str, i32)>,
    sprite_offsets: BTreeSet<i32>,
}

impl<'a> CompileContext<'a> {
    fn new(
        namespace: &'a str,
        runtime_sprites: &'a BTreeMap<String, RuntimeSpriteAsset>,
        text_fonts: &'a TextFonts,
        codepoints: BTreeMap<String, u32>,
    ) -> Self {
        Self {
            namespace,
            runtime_sprites,
            text_fonts,
            codepoints,
            files: Vec::new(),
            warnings: Vec::new(),
            bitmap_providers: Vec::new(),
            shift_offsets: BTreeSet::new(),
            text_font_offsets: BTreeSet::new(),
            sprite_offsets: BTreeSet::new(),
        }
    }

    /// Registers the font `slot` draws with at vertical offset `k` and returns its id.
    fn text_font(&mut self, slot: &SlotIr, k: i32) -> Result<String> {
        let Some(name) = slot.font.as_deref() else {
            self.shift_offsets.insert(k);
            return Ok(format!("{}:{}", self.namespace, shifted_suffix(k)));
        };
        let (name, _) = self
            .text_fonts
            .get_key_value(name)
            .ok_or_else(|| Error::Validation(format!("slot `{}` uses unknown font `{name}`", slot.name)))?;
        self.text_font_offsets.insert((name, k));
        Ok(format!("{}:{}", self.namespace, text_font_suffix(name, k)))
    }

    /// Registers the sprite font drawn at vertical offset `k` and returns its id.
    fn sprite_font(&mut self, k: i32) -> String {
        self.sprite_offsets.insert(k);
        format!("{}:sprite_{}", self.namespace, shifted_suffix(k))
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
        let font_metrics = fonts::font_metrics(self, &text_advances, &text_glyph_widths);
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
        files.push(OutputFile::text(crate::debug::output_path(self.namespace), debug_descriptor.to_json()?));
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(CompileOutput { files, warnings: self.warnings, manifest })
    }
}
