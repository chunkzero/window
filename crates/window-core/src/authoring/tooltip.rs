use serde::Deserialize;

use crate::ir::Tooltip;
use crate::{Error, Result};

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum TooltipDto {
    Title(String),
    Object(TooltipObjectDto),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TooltipObjectDto {
    title: String,
    #[serde(default)]
    lines: Vec<String>,
}

impl TooltipDto {
    pub(super) fn into_tooltip(self) -> Result<Tooltip> {
        let tooltip = match self {
            TooltipDto::Title(title) => Tooltip { title, lines: Vec::new() },
            TooltipDto::Object(TooltipObjectDto { title, lines }) => Tooltip { title, lines },
        };
        if tooltip.title.is_empty() {
            return Err(Error::Validation("tooltip title must not be empty".into()));
        }
        Ok(tooltip)
    }
}
