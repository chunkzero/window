use serde::Deserialize;

use super::element::{ElementDto, collect_handles, convert_children, intern_art};
use crate::ir::{FullscreenMode, Rgb};
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
        let backdrop = self
            .backdrop
            .map(|color| {
                Rgb::parse_hex(&color).ok_or_else(|| {
                    Error::Validation(format!("{owner} has invalid backdrop `{color}`; expected #rrggbb"))
                })
            })
            .transpose()?;
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
