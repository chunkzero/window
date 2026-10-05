use std::collections::BTreeSet;
use std::iter;

use serde::Deserialize;

use super::element::{ElementDto, convert_children};
use super::hud::HudDto;
use super::insets::InsetsDto;
use super::parse::validate_name;
use super::theme::ThemeDto;
use super::{BuildOptions, PackTarget, ParsedProject};
use crate::model::{Hud, Theme, Window};
use crate::surface::ContainerKind;
use crate::{Error, Result};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProjectDto {
    #[serde(default)]
    theme: ThemeDto,
    #[serde(default)]
    themes: Vec<ThemeDto>,
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
    frame: Option<String>,
    #[serde(default)]
    children: Vec<ElementDto>,
}

impl ProjectDto {
    pub(super) fn into_project(self) -> Result<ParsedProject> {
        let mut theme = Theme::default();
        for theme_doc in iter::once(self.theme).chain(self.themes) {
            theme_doc.merge_into(&mut theme)?;
        }

        Ok(ParsedProject {
            theme,
            windows: convert_windows(self.windows)?,
            huds: convert_huds(self.huds)?,
            options: BuildOptions {
                hud_shaders: self.options.hud_shaders,
                anvil_field_sprite: self.options.anvil_field_sprite,
                experimental_anvil_updates: self.options.experimental_anvil_updates,
            },
            target: PackTarget { pack_format: self.target.pack_format },
        })
    }
}

fn convert_windows(windows: Vec<WindowDto>) -> Result<Vec<Window>> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(windows.len());
    for window in windows {
        validate_name(&window.name, "window")?;
        if !seen.insert(window.name.clone()) {
            return Err(Error::Validation(format!("duplicate window name `{}`", window.name)));
        }
        out.push(window.into_window()?);
    }
    Ok(out)
}

fn convert_huds(huds: Vec<HudDto>) -> Result<Vec<Hud>> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(huds.len());
    for hud in huds {
        validate_name(&hud.name, "hud")?;
        if !seen.insert(hud.name.clone()) {
            return Err(Error::Validation(format!("duplicate hud name `{}`", hud.name)));
        }
        out.push(hud.into_hud()?);
    }
    Ok(out)
}

impl WindowDto {
    fn into_window(self) -> Result<Window> {
        let container = ContainerKind::parse(&self.container).ok_or_else(|| {
            let valid: Vec<&str> = ContainerKind::ALL.iter().map(|k| k.id()).collect();
            Error::Validation(format!(
                "window `{}` uses unknown container `{}`; valid ids: {}",
                self.name,
                self.container,
                valid.join(", ")
            ))
        })?;
        let children = convert_children(self.children)?;
        Ok(Window { name: self.name, container, bleed: self.bleed.into_insets(), frame: self.frame, children })
    }
}
