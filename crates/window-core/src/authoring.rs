//! TypeScript-authored project model.
//!
//! The rpp plugin loads `window/**/*.ts` definitions, merges their exported values,
//! JSON-encodes the result, and hands it to this module. Windows, HUDs, and full-screen windows carry a
//! tree of typed elements whose art is authored inline; the project adds the
//! runtime sprite catalog and the text fonts.

pub(crate) mod art;
mod element;
mod flex;
mod fullscreen;
mod hud;
mod insets;
mod parse;
mod patterns;
mod project;
mod style;
mod tooltip;

#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use self::project::ProjectDto;
use crate::model::{Art, FontDef, Fullscreen, Hud, Window};
use crate::{Error, Result};
pub(crate) use parse::sprite_key;

/// Everything parsed from a TypeScript-authored Window project.
#[derive(Clone, Debug, Default)]
pub struct ParsedProject {
    /// Every frame and sprite the project draws, by interned name.
    pub art: Art,
    /// Text fonts by name, which text selects with `font`.
    pub fonts: BTreeMap<String, FontDef>,
    /// All windows (duplicate window names are errors).
    pub windows: Vec<Window>,
    /// All HUDs (duplicate HUD names are errors).
    pub huds: Vec<Hud>,
    /// All full-screen windows (duplicate full-screen names are errors).
    pub fullscreens: Vec<Fullscreen>,
    /// Build/runtime options supplied by the rpp host.
    pub options: BuildOptions,
    /// Target Minecraft/resource-pack version metadata supplied by the rpp host.
    pub target: PackTarget,
}

/// Build options supplied by the rpp plugin host.
#[derive(Clone, Debug, Default)]
pub struct BuildOptions {
    /// `coreShaders`: generated core shader overrides.
    pub core_shaders: CoreShaders,
    /// `vanilla`: replacements for vanilla textures.
    pub vanilla: Vanilla,
    /// `experimental`: options that may change or break.
    pub experimental: Experimental,
    /// `size`: the output size report and budget.
    pub size: SizeOptions,
}

/// `size` options.
#[derive(Clone, Debug, Default)]
pub struct SizeOptions {
    /// `size.report`: print the output size breakdown after a successful build.
    pub report: bool,
    /// `size.budget`: fail the build when the output exceeds this many bytes.
    pub budget: Option<u64>,
}

/// `coreShaders` options.
#[derive(Clone, Debug, Default)]
pub struct CoreShaders {
    /// `coreShaders.hud`: emit core shader overrides for HUD relocation.
    pub hud: bool,
    /// `coreShaders.hover`: emit core shader overrides that outline the hovered region.
    pub hover: bool,
}

/// `experimental` options.
#[derive(Clone, Debug, Default)]
pub struct Experimental {
    /// `experimental.anvilUpdates`: allow anvil input windows to change their title at runtime. Each change
    /// reopens the anvil, which races the player's typing.
    pub anvil_updates: bool,
}

/// What replaces a vanilla texture.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Replacement {
    /// Vanilla's texture stays.
    #[default]
    Keep,
    /// A transparent texture of vanilla's size.
    Blank,
    /// A runtime catalog sprite.
    Sprite(String),
}

/// `vanilla` options.
#[derive(Clone, Debug, Default)]
pub struct Vanilla {
    pub anvil: VanillaAnvil,
    /// `vanilla.hud`: the HUD groups whose GUI sprites are blanked.
    pub blank_hud: BTreeSet<HudGroup>,
    /// `vanilla.slotHighlight`: the 24x24 highlight drawn over the hovered slot of every container.
    pub slot_highlight: Replacement,
}

/// `vanilla.anvil` options.
#[derive(Clone, Debug, Default)]
pub struct VanillaAnvil {
    /// `vanilla.anvil.field`: the 110x16 text-field sprites.
    pub field: Replacement,
    /// `vanilla.anvil.background`: the background and error icon. `None` blanks both exactly when some
    /// window has an anvil input.
    pub background: Option<Replacement>,
}

/// A group of vanilla HUD sprites, named as in `vanilla.hud`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HudGroup {
    Hotbar,
    Health,
    Food,
    Armor,
    Air,
    Experience,
    Crosshair,
}

/// Target pack metadata supplied by the rpp plugin host.
#[derive(Clone, Debug, Default)]
pub struct PackTarget {
    /// Resource pack format, when known.
    pub pack_format: Option<u32>,
}

/// Parse the plugin's JSON project payload into the authored model.
pub fn project_from_json(bytes: &[u8]) -> Result<ParsedProject> {
    let dto: ProjectDto = serde_json::from_slice(bytes)
        .map_err(|error| Error::Parse { path: "window project".into(), message: error.to_string() })?;
    dto.into_project()
}
