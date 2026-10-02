use serde::Deserialize;

use crate::geometry::Insets;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum InsetsDto {
    Uniform(u32),
    Edges(InsetsEdgesDto),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InsetsEdgesDto {
    #[serde(default)]
    top: u32,
    #[serde(default)]
    right: u32,
    #[serde(default)]
    bottom: u32,
    #[serde(default)]
    left: u32,
}

impl Default for InsetsDto {
    fn default() -> Self {
        Self::Uniform(0)
    }
}

impl InsetsDto {
    pub(super) fn into_insets(self) -> Insets {
        match self {
            InsetsDto::Uniform(v) => Insets::uniform(v),
            InsetsDto::Edges(InsetsEdgesDto { top, right, bottom, left }) => Insets { top, right, bottom, left },
        }
    }
}
