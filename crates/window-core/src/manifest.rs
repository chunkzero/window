//! The compiled Window pack data model shared by resource-pack generation,
//! Kotlin codegen, and server runtimes. Schema reference: `docs/MANIFEST.md`.
//! Bump [`VERSION`] on any breaking change and update the runtime and codegen
//! in the same change.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::inventory::{InventorySlotArea, InventorySlotRef};
use crate::ir::{Align, Handle, Hitbox, IndexedBinding, Layer};
use crate::{Error, Result};

mod hud;

pub use hud::{HudEntry, HudShaderEntry, HudSurfaceEntry};

/// Current compiled definition schema version.
pub const VERSION: u32 = 8;

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
    /// Theme palette colors by name, lowercase `#rrggbb`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub colors: BTreeMap<String, String>,
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
    /// Inventory regions by key: slots claimed and filled with a hitbox item, and the action clicks name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub regions: BTreeMap<String, RegionEntry>,
    /// Dynamic inventory item regions by name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub items: BTreeMap<String, ItemEntry>,
    /// Dynamic repeated inventory item regions by name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub collections: BTreeMap<String, CollectionEntry>,
    /// Native inventory text inputs by name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub inputs: BTreeMap<String, AnvilInputEntry>,
    /// Group metadata for flattened repeater controls.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub groups: BTreeMap<String, RepeatGroupEntry>,
    /// Runtime-selected cases by key. Switches named by a case's `switches` are active only while that case is.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub switches: BTreeMap<String, SwitchEntry>,
    /// Every slot, sprite slot, switch, and selectable collection, once each, in authored tree order: the order the
    /// runtime composes them above the static chrome.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<Layer>,
    /// Indexed binding families by name. Their flattened entries appear under their own names in `slots`,
    /// `sprite_slots`, and `switches`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub indexed: BTreeMap<String, IndexedBinding>,
    /// Typed handles by id, each with the entries that use it. Kotlin codegen declares one member per handle and
    /// binds every use; the runtime ignores this table.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub handles: BTreeMap<String, Handle>,
}

/// Cases of which the runtime draws and claims only the one its binding names.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchEntry {
    /// The binding name when this switch is one case's copy of a binding shared across mutually exclusive switch
    /// cases; the switch is then keyed `{binding}.{case path}`. Absent when the key is the binding name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<String>,
    /// Whether this switch selects the named states of the control it is keyed by. Codegen declares no member for
    /// it; `buttonState` and the toggle, choice, and enabled helpers select its case.
    #[serde(default, skip_serializing_if = "is_false")]
    pub states: bool,
    /// The case active while the switch is unbound; absent means no case is active until it is bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial: Option<String>,
    /// The authored element a derived switch comes from, such as ``tab `category_new` ``, for diagnostics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Cases in authoring order.
    pub cases: Vec<SwitchCaseEntry>,
}

/// One case of a [`SwitchEntry`]. It lists the entries directly inside it; entries of nested switches are listed by
/// their own cases.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchCaseEntry {
    /// Case value the binding returns to select this case.
    pub value: String,
    /// Net-zero baked art of this case in [`Manifest::font`], drawn at the switch's position in
    /// [`WindowEntry::layers`]. Window cases start and end at the title origin; HUD cases at the HUD's left edge.
    #[serde(rename = "static", default, skip_serializing_if = "String::is_empty")]
    pub static_text: String,
    /// Text regions drawn only while this case is active.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slots: Vec<String>,
    /// Runtime sprite regions drawn only while this case is active.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sprite_slots: Vec<String>,
    /// Inventory regions claimed only while this case is active.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<String>,
    /// Switches active only while this case is active.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub switches: Vec<String>,
    /// Item regions filled only while this case is active.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
    /// Collections filled, and clicked, only while this case is active.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collections: Vec<String>,
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
    /// The binding name when this slot is one case's copy of a binding shared across a switch's cases; the slot
    /// is then keyed `{binding}.{case}`. Absent when the key is the binding name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<String>,
    /// How content wider than [`Self::width`] is shortened; absent leaves it untouched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow: Option<TextOverflow>,
    /// Present when content wraps onto more than one line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<SlotLinesEntry>,
    /// The authored `debug_name` this slot comes from, for diagnostics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl SlotEntry {
    /// Height of the slot's box: 8px per line, consecutive lines `line_height` apart.
    pub fn height(&self) -> u32 {
        self.lines.as_ref().map_or(8, |lines| (lines.count - 1) * lines.line_height + 8)
    }

    /// Every font this slot draws with, paired with the offset of the line it draws below [`Self::y`].
    pub fn fonts(&self) -> Vec<(&str, i32)> {
        match &self.lines {
            Some(lines) => (lines.fonts.iter().enumerate())
                .map(|(s, font)| (font.as_str(), s as i32 * lines.line_height as i32 / 2))
                .collect(),
            None => vec![(self.font.as_str(), 0)],
        }
    }
}

/// How a text slot shortens content wider than its width.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextOverflow {
    /// Truncate and end with an ellipsis.
    Ellipsis,
}

/// The lines a multi-line text slot wraps onto, vertically centered in its box.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotLinesEntry {
    /// Maximum number of lines, at least 2.
    pub count: u32,
    /// Distance between the tops of consecutive lines, in pixels.
    pub line_height: u32,
    /// Shifted fonts by half-line step: `fonts[s]` draws a line whose top is
    /// `y + floor(s * line_height / 2)`. With `k` lines used, line `j` uses `s = count - k + 2j`, so the
    /// table has `2 * count - 1` entries and `fonts[0]` equals the slot's font.
    pub fonts: Vec<String>,
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
    /// The binding name when this sprite slot is one case's copy of a binding shared across a switch's cases;
    /// the slot is then keyed `{binding}.{case}`. Absent when the key is the binding name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<String>,
    /// The authored `debug_name` this sprite slot comes from, for diagnostics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// An inventory region: the slots it claims and fills with its hitbox item, and the action its clicks name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionEntry {
    /// Left edge (GUI space).
    pub x: i32,
    /// Top edge (GUI space).
    pub y: i32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Backing inventory slots whose clicks route to this region.
    pub slots: Vec<SlotRefEntry>,
    /// Slots this region fills with its hitbox item. Absent means every slot in [`Self::slots`]; present means a
    /// strict subset, because a repeater cell yielded the remaining slots to item controls that own the real stack
    /// (and therefore the native hover tooltip) in those slots, or the slot holds the anvil input's seed item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_slots: Option<Vec<SlotRefEntry>>,
    /// The action id a click names: the handler bound to it runs, else [`Self::default_action`]. Ids in the
    /// `window:` namespace are runtime actions with no generated Kotlin member. Absent for hover-only and
    /// claim-only regions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// The runtime action, such as `window:close`, run when no handler is bound to [`Self::action`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_action: Option<String>,
    /// The hitbox item filling [`Self::filled_slots`]; absent leaves them empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hitbox: Option<Hitbox>,
    /// The authored element this region comes from, such as ``button `buy` ``, for diagnostics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl RegionEntry {
    /// The slots this region paints with its hitbox item: [`Self::fill_slots`] when present, otherwise every click
    /// slot.
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
    /// The selected-cell sprite placed over each cell's 18x18 box, indexed like
    /// `slots`; empty when the collection has no selected sprite.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selection: Vec<SpriteSlotEntry>,
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
    /// The action of each cell's region, in index order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<String>,
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

/// The switch cases an entry sits in, outermost first, as `(switch key, case value)`.
pub type CasePath = Vec<(String, String)>;

/// The case paths of the regions, items, and collections inside switch cases, each by name. Entries outside any
/// case are absent.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CasePaths {
    pub regions: BTreeMap<String, CasePath>,
    pub items: BTreeMap<String, CasePath>,
    pub collections: BTreeMap<String, CasePath>,
}

/// The case path of every region, item, and collection inside a switch case.
pub fn case_paths(switches: &BTreeMap<String, SwitchEntry>) -> CasePaths {
    let mut parents: BTreeMap<&str, (&str, &str)> = BTreeMap::new();
    for (key, switch) in switches {
        for case in &switch.cases {
            for child in &case.switches {
                parents.insert(child, (key, &case.value));
            }
        }
    }
    let mut paths = CasePaths::default();
    for (key, switch) in switches {
        let mut outer = Vec::new();
        let mut child = key.as_str();
        while let Some(&(parent, value)) = parents.get(child) {
            outer.push((parent.to_string(), value.to_string()));
            child = parent;
        }
        outer.reverse();
        for case in &switch.cases {
            let mut path = outer.clone();
            path.push((key.clone(), case.value.clone()));
            let entries = [
                (&mut paths.regions, &case.regions),
                (&mut paths.items, &case.items),
                (&mut paths.collections, &case.collections),
            ];
            for (paths, names) in entries {
                paths.extend(names.iter().map(|name| (name.clone(), path.clone())));
            }
        }
    }
    paths
}

/// Whether two case paths can never be active together: they pick different cases of one switch.
pub fn exclusive_cases(a: &[(String, String)], b: &[(String, String)]) -> bool {
    for (x, y) in a.iter().zip(b) {
        if x.0 != y.0 {
            return false;
        }
        if x.1 != y.1 {
            return true;
        }
    }
    false
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
