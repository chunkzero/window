use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{SlotEntry, SwitchEntry};
use crate::ir::{Handle, IndexedBinding};

/// One compiled HUD.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HudEntry {
    /// HUD surface/channel metadata.
    pub surface: HudSurfaceEntry,
    /// Fixed-width static segment. Send as-is in [`super::Manifest::font`] before slot
    /// overlays.
    #[serde(rename = "static")]
    pub static_text: String,
    /// Text regions by name (entries with `text` are static labels).
    pub slots: BTreeMap<String, SlotEntry>,
    /// Optional core-shader relocation settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shader: Option<HudShaderEntry>,
    /// Runtime-selected visual cases by binding name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub switches: BTreeMap<String, SwitchEntry>,
    /// Indexed binding families by name. Their flattened entries appear under their own names in `slots` and
    /// `switches`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub indexed: BTreeMap<String, IndexedBinding>,
    /// Typed handles by id, each with the entries that use it; see [`super::WindowEntry::handles`].
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub handles: BTreeMap<String, Handle>,
}

/// HUD surface/channel metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HudSurfaceEntry {
    /// Surface kind discriminator; v1: always `"hud"`.
    pub kind: String,
    /// Vanilla transport channel: `"actionbar"`, `"bossbar"`, or `"sidebar"`.
    pub channel: String,
    /// Fixed canvas/line width in GUI pixels.
    pub width: u32,
    /// Informational canvas height in GUI pixels.
    pub height: u32,
}

/// Optional generated core-shader relocation settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HudShaderEntry {
    /// Marker color used for the baked static HUD segment, lowercase `#rrggbb`.
    pub static_marker: String,
    /// Source surface top y-offset from the bottom of the GUI, in GUI pixels.
    pub source_bottom: i32,
    /// Normalized target origin in GUI space: 0.0 = left, 1.0 = right.
    pub origin_x: f32,
    /// Normalized target origin in GUI space: 0.0 = top, 1.0 = bottom.
    pub origin_y: f32,
    /// Normalized point inside the HUD surface placed on the target origin.
    pub anchor_x: f32,
    /// Normalized point inside the HUD surface placed on the target origin.
    pub anchor_y: f32,
    /// X nudge from the normalized target origin, in GUI pixels.
    pub offset_x: i32,
    /// Y nudge from the normalized target origin, in GUI pixels.
    pub offset_y: i32,
}
