use serde::Deserialize;

use super::element::{ElementDto, collect_handles, convert_children, intern_art};
use crate::ir::{Backdrop, FullscreenMode, Rgb};
use crate::model::{Art, Fullscreen};
use crate::{Error, Result};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FullscreenDto {
    pub(super) name: String,
    mode: Option<String>,
    backdrop: Option<String>,
    #[serde(default)]
    children: Vec<ElementDto>,
    pub(super) debug_name: Option<String>,
}

impl FullscreenDto {
    pub(super) fn into_fullscreen(mut self, art: &mut Art) -> Result<Fullscreen> {
        let owner = format!("fullscreen `{}`", self.name);
        let in_fullscreen = |error: Error| match error {
            Error::Validation(message) => Error::Validation(format!("{owner}: {message}")),
            other => other,
        };
        intern_art(&mut self.children, art).map_err(in_fullscreen)?;
        let mut children = self.children;
        let handles = collect_handles(&mut children, &owner, false)?;
        let mode = match self.mode.as_deref() {
            None | Some("unlocked") => FullscreenMode::Unlocked,
            Some("horse") => FullscreenMode::Horse,
            Some(other) => {
                return Err(Error::Validation(format!(
                    "{owner} has unknown mode `{other}`; valid modes: unlocked, horse"
                )));
            }
        };
        let backdrop = self.backdrop.map(|color| parse_backdrop(&color, &owner)).transpose()?;
        Ok(Fullscreen {
            name: self.name,
            mode,
            backdrop,
            children: convert_children(children)?,
            handles,
            debug_name: self.debug_name,
        })
    }
}

/// Parses `#rrggbb`, which hides the world, or `#rrggbbaa`, which dims it. The text shader drops fragments below 10%
/// opacity, so a dimming alpha starts at `1a`.
fn parse_backdrop(color: &str, owner: &str) -> Result<Backdrop> {
    let invalid =
        || Error::Validation(format!("{owner} has invalid backdrop `{color}`; expected #rrggbb or #rrggbbaa"));
    let (rgb, alpha) = match color.len() {
        7 => (color, 0xff),
        9 => (&color[..7], u8::from_str_radix(&color[7..], 16).map_err(|_| invalid())?),
        _ => return Err(invalid()),
    };
    let color_rgb = Rgb::parse_hex(rgb).ok_or_else(invalid)?;
    if alpha < MIN_BACKDROP_ALPHA {
        return Err(Error::Validation(format!(
            "{owner} has backdrop `{color}` below the lowest alpha the text shader draws, `{MIN_BACKDROP_ALPHA:02x}`"
        )));
    }
    Ok(Backdrop { color: color_rgb, alpha })
}

/// The lowest backdrop alpha whose fragments pass the text shader's 0.1 alpha cutoff.
const MIN_BACKDROP_ALPHA: u8 = 0x1a;
