use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{ArtEntry, SlotEntry, SwitchEntry};
use crate::ir::{FullscreenMode, Handle, Layer};

/// One compiled full-screen window. Its coordinates are canvas pixels; see [`crate::fullscreen`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullscreenEntry {
    /// Full-screen surface metadata.
    pub surface: FullscreenSurfaceEntry,
    /// Net-zero baked static art from the canvas origin, sent first in [`super::Manifest::font`].
    #[serde(rename = "static")]
    pub static_text: String,
    /// The marker color [`Self::static_text`] is sent in, lowercase `#rrggbb`.
    pub static_marker: String,
    /// Text regions by name (entries with `text` are static labels).
    pub slots: BTreeMap<String, SlotEntry>,
    /// Clickable and hoverable regions by key, which the runtime hit-tests the cursor against.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub controls: BTreeMap<String, ControlEntry>,
    /// Runtime-selected cases by key; see [`super::WindowEntry::switches`]. A case's `regions` name its controls.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub switches: BTreeMap<String, SwitchEntry>,
    /// Art layers by name; see [`super::WindowEntry::art`]. A tinted layer's `tint_markers` hold its marker per
    /// palette entry.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub art: BTreeMap<String, ArtEntry>,
    /// Tint palettes by tint id; see [`super::WindowEntry::tints`].
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tints: BTreeMap<String, BTreeMap<String, String>>,
    /// Every slot, switch, and art layer in authored tree order; see [`super::WindowEntry::layers`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<Layer>,
    /// The marker color each of [`Self::layers`] is sent in, by name. A tinted slot or art layer is instead sent in
    /// the `tint_markers` entry of its current palette entry.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub markers: BTreeMap<String, String>,
    /// The marker color [`FullscreenCanvasEntry::backdrop`] is sent in, present when the window has a backdrop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backdrop_marker: Option<String>,
    /// Typed handles by id, each with the entries that use it; see [`super::WindowEntry::handles`].
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub handles: BTreeMap<String, Handle>,
}

/// Full-screen surface metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullscreenSurfaceEntry {
    /// Surface kind discriminator; always `"fullscreen"`.
    pub kind: String,
    /// How the cursor follows the mouse.
    pub mode: FullscreenMode,
    /// Canvas size `[width, height]`.
    pub size: [u32; 2],
}

/// A clickable or hoverable rect of a full-screen window.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlEntry {
    /// The control's index in the shader include's `WINDOW_FS_CONTROLS`, unique across the pack.
    pub index: u32,
    /// Left edge (canvas pixels).
    pub x: i32,
    /// Top edge (canvas pixels).
    pub y: i32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// The action id a click names; see [`super::RegionEntry::action`]. Absent for a hover-only control.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// The runtime action, such as `window:close`, run when no handler is bound to [`Self::action`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_action: Option<String>,
    /// Whether the control is outlined while the cursor is over it.
    pub hover: bool,
    /// The control's hover outline, net-zero from the line origin, which the runtime sends in the overlay marker
    /// while the cursor is over the control. Present when [`Self::hover`] is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outline: Option<String>,
    /// The authored element this control comes from, such as ``region `buy` ``, for diagnostics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// The full-screen canvas contract of [`crate::fullscreen`], which the runtime and the generated shaders share.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FullscreenCanvasEntry {
    /// Canvas width in canvas pixels.
    pub width: u32,
    /// Canvas height in canvas pixels.
    pub height: u32,
    /// Degrees of yaw or pitch per canvas pixel.
    pub degrees_per_pixel: f64,
    /// The yaw from 0 at which the cursor reaches the left or right canvas edge.
    pub edge_yaw: f64,
    /// The pitch from 0 at which the cursor reaches the top or bottom canvas edge.
    pub edge_pitch: f64,
    /// Added to each cursor coordinate before flooring; see [`crate::fullscreen::cursor`].
    pub snap_epsilon: f64,
    /// Canvas pixels a hit-test grows each control by.
    pub hit_margin: u32,
    /// The uniform scale of the text display carrying the runs.
    pub display_scale: f64,
    /// The text opacity of the text display carrying the runs.
    pub marker_alpha: u32,
    /// The built-in cursor, sent from the line origin in its marker. Its top-left texel is the hot spot.
    pub cursor: CursorEntry,
    /// The built-in backdrop glyph, net-zero from the line origin, which a window's backdrop marker stretches over
    /// the screen.
    pub backdrop: String,
    /// The marker of the runtime's own overlays, such as hover outlines: content above every window layer, white.
    pub overlay_marker: String,
    /// The world age the server sends while the HUD is hidden.
    pub hud_hidden_world_age: i64,
    /// The HUD strip's height in GUI pixels, from the bottom of the screen.
    pub hud_strip_height: u32,
    /// Half the HUD strip's width in GUI pixels, either side of the screen's center.
    pub hud_strip_half_width: u32,
    /// The most max health the client is told while the HUD is hidden.
    pub hud_max_health: u32,
}

/// The built-in cursor glyph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorEntry {
    /// The cursor glyph, net-zero from the line origin, in [`super::Manifest::font`].
    pub text: String,
    /// The marker color [`Self::text`] is sent in.
    pub marker: String,
    /// Width in canvas pixels.
    pub width: u32,
    /// Height in canvas pixels.
    pub height: u32,
}
