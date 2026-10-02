//! The compiled Window pack data model shared by resource-pack generation,
//! Kotlin codegen, and server runtimes. Schema reference: `docs/MANIFEST.md`.
//! Bump [`VERSION`] on any breaking change and update the runtime and codegen
//! in the same change.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::inventory::{InventorySlotArea, InventorySlotRef};
use crate::ir::{Align, ButtonDefault, ButtonState, ButtonTooltip};
use crate::{Error, Result};

/// Current compiled definition schema version.
pub const VERSION: u32 = 5;

/// Root compiled pack definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    /// Schema version (== [`VERSION`]).
    pub version: u32,
    /// Resource namespace the fonts/textures were emitted under.
    pub namespace: String,
    /// Font id of the main font (static glyphs + spacers), e.g. `window:ui`.
    pub font: String,
    /// Spacer table: codepoint → horizontal advance in pixels.
    pub spacers: BTreeMap<u32, i32>,
    /// Vanilla-font advance widths (pixels) per character; applies to every
    /// shifted label font as well.
    pub text_advances: BTreeMap<char, u32>,
    /// Vanilla-font visible glyph widths (pixels) per character. Consumers use
    /// these for visual alignment while `text_advances` remains the cursor math.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub text_glyph_widths: BTreeMap<char, u32>,
    /// Font metrics keyed by font id. This includes every generated shifted
    /// text font so runtime consumers can measure styled Adventure components
    /// by their effective font instead of assuming one global table.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub font_metrics: BTreeMap<String, FontMetricsEntry>,
    /// Runtime sprite catalog. Each sprite is repeated into every generated
    /// sprite font that needs a distinct vertical offset, while sharing one
    /// underlying texture.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sprites: BTreeMap<String, SpriteEntry>,
    /// All compiled windows by name.
    pub windows: BTreeMap<String, WindowEntry>,
    /// All compiled HUDs by name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub huds: BTreeMap<String, HudEntry>,
}

/// Per-font text metrics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontMetricsEntry {
    /// Cursor advance widths (pixels) per character.
    pub advances: BTreeMap<char, u32>,
    /// Visible glyph widths (pixels) per character.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub glyph_widths: BTreeMap<char, u32>,
    /// Extra cursor advance added per character when bold is active.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub bold_advance: u32,
}

/// One compiled window.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowEntry {
    /// The surface this window targets.
    pub surface: SurfaceEntry,
    /// Net-zero baked static chrome: send as-is, styled with [`Manifest::font`].
    #[serde(rename = "static")]
    pub static_text: String,
    /// Text regions by name (entries with `text` are static labels).
    pub slots: BTreeMap<String, SlotEntry>,
    /// Runtime sprite regions by name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sprite_slots: BTreeMap<String, SpriteSlotEntry>,
    /// Clickable regions by name.
    pub buttons: BTreeMap<String, ButtonEntry>,
    /// Dynamic inventory item regions by name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub items: BTreeMap<String, ItemEntry>,
    /// Dynamic repeated inventory item regions by name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub collections: BTreeMap<String, CollectionEntry>,
    /// Native inventory text inputs by name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub inputs: BTreeMap<String, AnvilInputEntry>,
    /// Non-binding slot claims/fills by name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub slot_rects: BTreeMap<String, SlotRectEntry>,
    /// Group metadata for flattened repeater controls.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub groups: BTreeMap<String, RepeatGroupEntry>,
}

/// One compiled HUD.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HudEntry {
    /// HUD surface/channel metadata.
    pub surface: HudSurfaceEntry,
    /// Fixed-width static segment. Send as-is in [`Manifest::font`] before slot
    /// overlays.
    #[serde(rename = "static")]
    pub static_text: String,
    /// Text regions by name (entries with `text` are static labels).
    pub slots: BTreeMap<String, SlotEntry>,
    /// Optional core-shader relocation settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shader: Option<HudShaderEntry>,
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

/// Surface description.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceEntry {
    /// Surface kind discriminator; v1: always `"container"`.
    pub kind: String,
    /// Container kind id, e.g. `generic_9x6`.
    pub container: String,
    /// GUI texture size `[width, height]`.
    pub size: [u32; 2],
    /// Title cursor origin `[x, y]` in GUI space.
    pub title_origin: [i32; 2],
}

/// Which backing inventory a control slot belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotAreaEntry {
    /// Slot in the opened container inventory.
    Container,
    /// Slot in the viewing player's own inventory.
    Player,
}

impl From<InventorySlotArea> for SlotAreaEntry {
    fn from(value: InventorySlotArea) -> Self {
        match value {
            InventorySlotArea::Container => SlotAreaEntry::Container,
            InventorySlotArea::Player => SlotAreaEntry::Player,
        }
    }
}

/// A typed inventory slot owned by a window control.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SlotRefEntry {
    /// Backing inventory area.
    pub area: SlotAreaEntry,
    /// Slot index in that area.
    pub index: u32,
}

impl From<InventorySlotRef> for SlotRefEntry {
    fn from(value: InventorySlotRef) -> Self {
        Self { area: value.area.into(), index: value.index }
    }
}

/// A text region.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotEntry {
    /// Left edge of the reserved text rect (GUI space).
    pub x: i32,
    /// Top edge of the text line (GUI space).
    pub y: i32,
    /// Reserved width for alignment.
    pub width: u32,
    /// Horizontal alignment within the reserved width.
    pub align: Align,
    /// Shifted font id that places text at this `y`, e.g. `window:y12`.
    pub font: String,
    /// Text color, lowercase `#rrggbb`.
    pub color: String,
    /// Optional marker color used by generated HUD shaders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shader_marker: Option<String>,
    /// Deprecated optional near-identical marker color used by older generated HUD shaders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shader_color: Option<String>,
    /// Whether the text renders with a shadow.
    pub shadow: bool,
    /// Whether bold is enabled by default.
    #[serde(default, skip_serializing_if = "is_false")]
    pub bold: bool,
    /// Whether italic is enabled by default.
    #[serde(default, skip_serializing_if = "is_false")]
    pub italic: bool,
    /// Whether underline is enabled by default.
    #[serde(default, skip_serializing_if = "is_false")]
    pub underlined: bool,
    /// Whether strikethrough is enabled by default.
    #[serde(default, skip_serializing_if = "is_false")]
    pub strikethrough: bool,
    /// Whether obfuscated text is enabled by default.
    #[serde(default, skip_serializing_if = "is_false")]
    pub obfuscated: bool,
    /// Present for static labels; the runtime renders these automatically.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// One runtime-renderable sprite.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteEntry {
    /// Rendered sprite width in GUI pixels.
    pub width: u32,
    /// Rendered sprite height in GUI pixels.
    pub height: u32,
    /// Left transparent padding before visible sprite ink, in GUI pixels.
    #[serde(default)]
    pub x_offset: u32,
    /// Visible ink width used for visual alignment, in GUI pixels.
    #[serde(default)]
    pub glyph_width: u32,
    /// Cursor advance of this bitmap glyph in the generated sprite fonts.
    #[serde(default)]
    pub advance: u32,
    /// Allocated glyph as a one-character string.
    pub glyph: String,
}

/// A runtime sprite region.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteSlotEntry {
    /// Left edge of the reserved sprite rect (GUI space).
    pub x: i32,
    /// Top edge of the reserved sprite rect (GUI space).
    pub y: i32,
    /// Reserved width for alignment.
    pub width: u32,
    /// Reserved height, informational and used by tooling.
    pub height: u32,
    /// Horizontal alignment within the reserved width.
    pub align: Align,
    /// Generated sprite font id for this vertical offset.
    pub font: String,
    /// Fixed sprite id, or `None` when the runtime must bind this slot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sprite: Option<String>,
}

/// A clickable region.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ButtonEntry {
    /// Left edge (GUI space).
    pub x: i32,
    /// Top edge (GUI space).
    pub y: i32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Backing inventory slots whose clicks route to this button.
    pub slots: Vec<SlotRefEntry>,
    /// Slots this button fills with its own hitbox/state item. Absent means
    /// every slot in [`Self::slots`]; present means a strict subset, because a
    /// repeater cell yielded the remaining slots to item controls that own the
    /// real stack (and therefore the native hover tooltip) in those slots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_slots: Option<Vec<SlotRefEntry>>,
    /// Built-in default behavior, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<ButtonDefault>,
    /// Whether this region should generate/accept a click handler.
    #[serde(default = "default_action", skip_serializing_if = "is_true")]
    pub action: bool,
    /// Default tooltip for this button/hotspot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<ButtonTooltip>,
    /// Named item states for dynamic visual/tooltip toggles.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub states: BTreeMap<String, ButtonState>,
    /// Generated sprite font used by visual states, when any state has a sprite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sprite_font: Option<String>,
}

impl ButtonEntry {
    /// The slots this button paints with its own item: [`Self::fill_slots`] when
    /// present, otherwise every click slot.
    pub fn filled_slots(&self) -> &[SlotRefEntry] {
        self.fill_slots.as_deref().unwrap_or(&self.slots)
    }
}

/// A dynamic inventory item region.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemEntry {
    /// Backing inventory slots this item control populates.
    pub slots: Vec<SlotRefEntry>,
}

/// A dynamic repeated inventory item region.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionEntry {
    /// Backing inventory slots, one visible cell per slot in order.
    pub slots: Vec<SlotRefEntry>,
    /// Whether this collection should generate/accept a click handler.
    #[serde(default = "default_action", skip_serializing_if = "is_true")]
    pub action: bool,
}

/// One native anvil rename-field binding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnvilInputEntry {
    /// Input slot seeded to make the vanilla rename field editable.
    pub slot: SlotRefEntry,
    /// Initial field contents.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub initial: String,
    /// Optional item model for the seed item in the input slot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_model: Option<String>,
}

/// A non-binding slot claim/fill region.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotRectEntry {
    /// Backing inventory slots claimed/cleared by this primitive.
    pub slots: Vec<SlotRefEntry>,
}

/// Group metadata for controls flattened from a repeater.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepeatGroupEntry {
    /// Number of repeated cells.
    pub count: u32,
    /// Dynamic text slots by repeated child field, each vector in index order.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub slots: BTreeMap<String, Vec<String>>,
    /// Runtime sprite slots by repeated child field, each vector in index order.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sprite_slots: BTreeMap<String, Vec<String>>,
    /// Dynamic inventory item controls by repeated child field, each vector in
    /// index order.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub items: BTreeMap<String, Vec<String>>,
    /// Root cell buttons, in index order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub buttons: Vec<String>,
}

fn default_action() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

impl Manifest {
    /// Serialize deterministically (pretty, sorted keys, trailing newline).
    pub fn to_json_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|e| Error::Manifest(e.to_string()))?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// Parse a manifest, validating the schema version.
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let manifest: Manifest = serde_json::from_slice(bytes).map_err(|e| Error::Manifest(e.to_string()))?;
        if manifest.version != VERSION {
            return Err(Error::Manifest(format!(
                "unsupported manifest version {} (expected {VERSION})",
                manifest.version
            )));
        }
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests;
