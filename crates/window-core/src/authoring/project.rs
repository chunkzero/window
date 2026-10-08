use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::art::{ArtRefDto, ArtUse, intern};
use super::element::{ElementDto, check_shared, collect_handles, convert_children, intern_art};
use super::hud::HudDto;
use super::insets::InsetsDto;
use super::parse::{sprite_key, validate_name};
use super::{BuildOptions, PackTarget, ParsedProject};
use crate::ir::Handle;
use crate::model::{Art, FontDef, Hud, Window};
use crate::surface::ContainerKind;
use crate::{Error, Result};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProjectDto {
    /// The runtime sprite catalog: art values by sprite name.
    #[serde(default)]
    sprites: BTreeMap<String, ArtRefDto>,
    /// Bitmap text fonts by name.
    #[serde(default)]
    fonts: BTreeMap<String, FontDto>,
    #[serde(default)]
    windows: Vec<WindowDto>,
    #[serde(default)]
    huds: Vec<HudDto>,
    #[serde(default)]
    options: OptionsDto,
    #[serde(default)]
    target: TargetDto,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct OptionsDto {
    #[serde(default)]
    hud_shaders: bool,
    #[serde(default)]
    anvil_field_sprite: Option<String>,
    #[serde(default)]
    experimental_anvil_updates: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FontDto {
    texture: String,
    chars: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetDto {
    pack_format: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WindowDto {
    name: String,
    container: String,
    #[serde(default)]
    bleed: InsetsDto,
    frame: Option<ArtRefDto>,
    #[serde(default)]
    children: Vec<ElementDto>,
    debug_name: Option<String>,
}

impl ProjectDto {
    pub(super) fn into_project(self) -> Result<ParsedProject> {
        let mut art = Art::default();
        add_catalog(self.sprites, &mut art)?;
        let mut fonts = BTreeMap::new();
        for (name, font) in self.fonts {
            validate_name(&name, "font")?;
            fonts.insert(name, FontDef { texture: font.texture, chars: font.chars });
        }

        let windows = convert_windows(self.windows, &mut art)?;
        let huds = convert_huds(self.huds, &mut art)?;
        let surfaces = windows
            .iter()
            .map(|w| (format!("window `{}`", w.name), &w.handles))
            .chain(huds.iter().map(|h| (format!("hud `{}`", h.name), &h.handles)));
        for (owner, handles) in surfaces {
            check_only(&owner, handles, &art)?;
        }
        check_shared(
            windows
                .iter()
                .map(|w| (format!("window `{}`", w.name), &w.handles))
                .chain(huds.iter().map(|h| (format!("hud `{}`", h.name), &h.handles)))
                .collect::<Vec<_>>()
                .iter()
                .map(|(owner, handles)| (owner.as_str(), *handles)),
        )?;
        let anvil_field_sprite =
            self.options.anvil_field_sprite.map(|name| sprite_key(&name, "anvilFieldSprite")).transpose()?;
        Ok(ParsedProject {
            art,
            fonts,
            windows,
            huds,
            options: BuildOptions {
                hud_shaders: self.options.hud_shaders,
                anvil_field_sprite,
                experimental_anvil_updates: self.options.experimental_anvil_updates,
            },
            target: PackTarget { pack_format: self.target.pack_format },
        })
    }
}

/// Adds the runtime sprite catalog to the project's art.
fn add_catalog(sprites: BTreeMap<String, ArtRefDto>, art: &mut Art) -> Result<()> {
    let mut declared = BTreeMap::new();
    for (authored, value) in sprites {
        let name = sprite_key(&authored, "sprite")?;
        if let Some(first) = declared.insert(name.clone(), authored.clone()) {
            return Err(Error::Validation(format!("catalog sprites `{first}` and `{authored}` both become `{name}`")));
        }
        let ArtRefDto::Inline(inline) = value else {
            return Err(Error::Validation(format!("catalog sprite `{name}` must be an art value")));
        };
        let sprite = inline.sprite().map_err(|error| error.with_debug_name(Some(&name)))?;
        art.sprites.insert(name.clone(), sprite);
        art.runtime.insert(name);
    }
    Ok(())
}

/// Rejects a sprite handle narrowed to a sprite outside the runtime sprite catalog.
fn check_only(owner: &str, handles: &BTreeMap<String, Handle>, art: &Art) -> Result<()> {
    for (id, handle) in handles {
        if let Some(sprite) = handle.only.iter().find(|sprite| !art.sprites.contains_key(*sprite)) {
            return Err(Error::Validation(format!(
                "{owner}: sprite `{id}` lists `{sprite}` in `only`, which is not a catalog sprite"
            )));
        }
    }
    Ok(())
}

fn convert_windows(windows: Vec<WindowDto>, art: &mut Art) -> Result<Vec<Window>> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(windows.len());
    for window in windows {
        validate_name(&window.name, "window")?;
        if !seen.insert(window.name.clone()) {
            return Err(Error::Validation(format!("duplicate window name `{}`", window.name)));
        }
        let debug_name = window.debug_name.clone();
        out.push(window.into_window(art).map_err(|error| error.with_debug_name(debug_name.as_deref()))?);
    }
    Ok(out)
}

fn convert_huds(huds: Vec<HudDto>, art: &mut Art) -> Result<Vec<Hud>> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(huds.len());
    for hud in huds {
        validate_name(&hud.name, "hud")?;
        if !seen.insert(hud.name.clone()) {
            return Err(Error::Validation(format!("duplicate hud name `{}`", hud.name)));
        }
        let debug_name = hud.debug_name.clone();
        out.push(hud.into_hud(art).map_err(|error| error.with_debug_name(debug_name.as_deref()))?);
    }
    Ok(out)
}

impl WindowDto {
    fn into_window(mut self, art: &mut Art) -> Result<Window> {
        let container = ContainerKind::parse(&self.container).ok_or_else(|| {
            let valid: Vec<&str> = ContainerKind::ALL.iter().map(|k| k.id()).collect();
            Error::Validation(format!(
                "window `{}` uses unknown container `{}`; valid ids: {}",
                self.name,
                self.container,
                valid.join(", ")
            ))
        })?;
        let owner = format!("window `{}`", self.name);
        let in_window = |error: Error| match error {
            Error::Validation(message) => Error::Validation(format!("{owner}: {message}")),
            other => other,
        };
        intern(art, &mut self.frame, ArtUse::Frame).map_err(in_window)?;
        intern_art(&mut self.children, art).map_err(in_window)?;
        let mut children = self.children;
        let handles = collect_handles(&mut children, &owner, false)?;
        Ok(Window {
            name: self.name,
            container,
            bleed: self.bleed.into_insets(),
            frame: self.frame.map(|frame| frame.name()).transpose()?,
            children: convert_children(children)?,
            handles,
            debug_name: self.debug_name,
        })
    }
}
