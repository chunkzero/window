//! Layout intermediate representation — the pinned seam between the
//! frontend (authoring + layout) and the backend (compose + font + bake +
//! manifest).
//!
//! Everything here is fully resolved: absolute GUI-space rects, concrete
//! texture paths, no remaining theme references.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::geometry::{Insets, Rect};
use crate::inventory::InventorySlotRef;
use crate::model::{GeneratedStyle, TextFit};
use crate::surface::Surface;

/// Horizontal text alignment within a slot's reserved width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    /// Anchor to the left edge.
    #[default]
    Left,
    /// Center within the reserved width.
    Center,
    /// Anchor to the right edge.
    Right,
}

/// Server/client channel a HUD is sent through before any optional shader
/// relocation. These are deliberately the vanilla surfaces that work without
/// core shaders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HudChannel {
    /// `Player#sendActionBar` / action-bar packets.
    #[default]
    ActionBar,
    /// Adventure bossbar name text.
    BossBar,
    /// Scoreboard sidebar title/lines.
    Sidebar,
}

impl HudChannel {
    /// Stable id used in authoring and the manifest.
    pub const fn id(self) -> &'static str {
        match self {
            HudChannel::ActionBar => "actionbar",
            HudChannel::BossBar => "bossbar",
            HudChannel::Sidebar => "sidebar",
        }
    }
}

/// Optional core-shader relocation settings for a HUD.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HudShader {
    /// Source surface top y-offset from the bottom of the GUI, in GUI pixels.
    /// The generated shader moves text vertices in this source surface band.
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

/// An sRGB color.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rgb {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

impl Rgb {
    /// The vanilla inventory-title gray (`#404040`), the default text color.
    pub const DEFAULT_TEXT: Rgb = Rgb { r: 0x40, g: 0x40, b: 0x40 };

    /// Construct an RGB color.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse `#rrggbb` (case-insensitive).
    pub fn parse_hex(s: &str) -> Option<Self> {
        let hex = s.strip_prefix('#')?;
        if hex.len() != 6 {
            return None;
        }
        let v = u32::from_str_radix(hex, 16).ok()?;
        Some(Rgb { r: (v >> 16) as u8, g: (v >> 8) as u8, b: v as u8 })
    }

    /// Format as lowercase `#rrggbb`.
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// The runtime action closing the window. Action ids in the `window:` namespace are resolved by the runtime.
pub const CLOSE_ACTION: &str = "window:close";

/// Tooltip content for a region's hitbox item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ButtonTooltip {
    /// Tooltip title/name.
    pub title: String,
    /// Additional lore lines.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<String>,
}

/// The invisible hitbox item a region fills its slots with.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hitbox {
    /// Item model id, e.g. `example:gui/shop_button_active`; absent uses the pack's hitbox model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_model: Option<String>,
    /// Hover tooltip.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<ButtonTooltip>,
}

/// A texture reference: the pack-source-relative path of a PNG.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TextureKey(pub String);

/// One static drawing operation, in paint order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Draw {
    /// Stretch a 9-slice frame over `dest`.
    NineSlice {
        /// Source texture.
        texture: TextureKey,
        /// Fixed border widths within the source texture.
        insets: Insets,
        /// Destination rect in GUI space.
        dest: Rect,
    },
    /// Blit a whole texture at its intrinsic size.
    Sprite {
        /// Source texture.
        texture: TextureKey,
        /// Destination rect in GUI space (size == intrinsic size).
        dest: Rect,
    },
    /// Render a generated theme style directly at `dest`.
    Generated {
        /// Procedural style to rasterize.
        style: GeneratedStyle,
        /// Destination rect in GUI space.
        dest: Rect,
    },
}

impl Draw {
    /// The destination rect of this draw.
    pub fn dest(&self) -> &Rect {
        match self {
            Draw::NineSlice { dest, .. } | Draw::Sprite { dest, .. } | Draw::Generated { dest, .. } => dest,
        }
    }

    /// The source texture of this draw, when it is bitmap-backed.
    pub fn texture(&self) -> Option<&TextureKey> {
        match self {
            Draw::NineSlice { texture, .. } | Draw::Sprite { texture, .. } => Some(texture),
            Draw::Generated { .. } => None,
        }
    }
}

/// A positioned text region: a dynamic slot, or a static label
/// (`text == Some(_)`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotIr {
    /// Unique name within the window. Static labels get generated names
    /// (`label_0`, `label_1`, … in document order).
    pub name: String,
    /// `Some` for static labels; `None` for dynamic slots.
    pub text: Option<String>,
    /// Reserved text rect; 8px tall (one vanilla text line) unless `fit` wraps onto more lines.
    pub rect: Rect,
    /// Horizontal alignment within `rect`.
    pub align: Align,
    /// Text color.
    pub color: Rgb,
    /// Whether the text renders with a shadow.
    pub shadow: bool,
    /// Whether the text renders bold.
    pub bold: bool,
    /// Whether the text renders italic.
    pub italic: bool,
    /// Whether the text renders underlined.
    pub underlined: bool,
    /// Whether the text renders with strikethrough.
    pub strikethrough: bool,
    /// Whether the text renders obfuscated.
    pub obfuscated: bool,
    /// The text font to draw with, or `None` for vanilla glyphs.
    pub font: Option<String>,
    /// Repeater metadata for grouped codegen, if this slot was emitted by a
    /// repeated template.
    pub repeat: Option<RepeatBindingIr>,
    /// The binding this slot shares with its copies in the other cases of one switch.
    pub binding: Option<String>,
    /// Runtime fitting of overflowing content; the default for static labels.
    pub fit: TextFit,
}

/// The kind of binding an [`IndexedBinding`] family flattens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexedKind {
    /// Dynamic text slots.
    Slot,
    /// Runtime sprite slots.
    SpriteSlot,
    /// Switches.
    Switch,
}

/// A binding authored with an `index`, flattened to one entry named `{name}_{i}` (one dimension) or
/// `{name}_{i}_{j}` (two dimensions) per index. Every index within `shape` is present.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexedBinding {
    /// What each flattened entry is.
    pub kind: IndexedKind,
    /// Extent of each index dimension: one or two values.
    pub shape: Vec<u32>,
}

impl IndexedBinding {
    /// The flattened entry name of `family` at `index`, such as `power[3]` or `strokes[2][4]`. Authored names
    /// cannot contain brackets, so entry names never collide with them.
    pub fn entry_name(family: &str, index: &[u32]) -> String {
        index.iter().fold(family.to_string(), |name, i| format!("{name}[{i}]"))
    }
}

/// What a typed handle declares: a value Kotlin computes, runtime state the UI owns, or an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandleKind {
    /// A Boolean computed by Kotlin.
    Flag,
    /// A Boolean the UI owns as runtime state.
    Toggle,
    /// One of `values`, computed by Kotlin.
    Value,
    /// One of `values`, owned by the UI as runtime state.
    Selection,
    /// Dynamic text.
    Text,
    /// A runtime sprite.
    Sprite,
    /// An inventory item.
    Items,
    /// A collection of inventory items, one per cell.
    Collection,
    /// A click handled by Kotlin.
    Action,
    /// A native anvil text input.
    Input,
    /// A runtime action such as `window:close`; it has no Kotlin member.
    Builtin,
}

impl HandleKind {
    /// The authored name of this kind.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Flag => "flag",
            Self::Toggle => "toggle",
            Self::Value => "value",
            Self::Selection => "selection",
            Self::Text => "text",
            Self::Sprite => "sprite",
            Self::Items => "items",
            Self::Collection => "collection",
            Self::Action => "action",
            Self::Input => "input",
            Self::Builtin => "builtin",
        }
    }
}

/// How an element uses a handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandleRole {
    /// Renders a dynamic text slot.
    Slot,
    /// Renders a runtime sprite slot.
    SpriteSlot,
    /// Renders an inventory item region.
    Item,
    /// Supplies a collection's cells.
    Collection,
    /// Receives an anvil input's value.
    Input,
    /// Selects a switch case.
    Switch,
    /// Handles a button's clicks.
    Click,
    /// Enables a button.
    Enabled,
    /// Selects a button's named state.
    State,
}

/// One element's use of a handle, bound to the manifest entry `entry`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HandleUse {
    /// How the entry uses the handle.
    pub role: HandleRole,
    /// The manifest entry key: a slot, sprite slot, item, collection, input, switch, or button.
    pub entry: String,
    /// The index read from an indexed handle; empty otherwise.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub at: Vec<u32>,
    /// The value a condition compares with (`is`), or a click assigns (`set`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// A typed handle a surface uses, with every use the surface makes of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Handle {
    /// What the handle declares.
    pub kind: HandleKind,
    /// The values of a value or selection handle, in declaration order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    /// Extent of each index dimension of an indexed handle: empty, one, or two values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shape: Vec<u32>,
    /// The initial value of UI-owned state: `true`/`false` for a toggle, a value for a selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial: Option<String>,
    /// Whether a collection marks a selected cell.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub selectable: bool,
    /// Every use, in authoring order.
    pub uses: Vec<HandleUse>,
}

impl Handle {
    /// Whether `other` declares the same handle, ignoring uses.
    pub fn same_declaration(&self, other: &Handle) -> bool {
        self.kind == other.kind
            && self.values == other.values
            && self.shape == other.shape
            && self.initial == other.initial
            && self.selectable == other.selectable
    }
}

/// A runtime-positioned sprite region.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteSlotIr {
    /// Unique name within the window.
    pub name: String,
    /// Reserved sprite rect in GUI space.
    pub rect: Rect,
    /// Horizontal alignment within `rect`.
    pub align: Align,
    /// Fixed sprite id, or `None` when the runtime must bind this slot.
    pub sprite: Option<String>,
    /// Repeater metadata for grouped codegen, if this sprite slot was emitted
    /// by a repeated template.
    pub repeat: Option<RepeatBindingIr>,
    /// The binding this sprite slot shares with its copies in the other cases of one switch.
    pub binding: Option<String>,
}

/// An inventory region: slots it claims and fills with its hitbox item, and the action its clicks name.
///
/// A region distinguishes two slot sets. Its **click slots** are the slots whose clicks route to it; its **fill
/// slots** are the slots it paints with its hitbox item. Fill slots are the click slots minus
/// [`Self::yielded_slots`], which a repeater cell uses to hand one of its slots to a child item control so the
/// item's real stack (and its native tooltip) survives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionIr {
    /// Unique region key within the window.
    pub name: String,
    /// Region rect in GUI space.
    pub rect: Rect,
    /// Explicit backing inventory slots, if authored. `None` means derive from the rect.
    pub slots: Option<Vec<InventorySlotRef>>,
    /// Click slots this region must not fill because another control owns their stacks. Always a subset of the
    /// resolved click slots.
    pub yielded_slots: Vec<InventorySlotRef>,
    /// Whether the region claims only the slots no other control owns, after every other claim.
    pub unowned: bool,
    /// The action id a click names; `None` for a hover-only or claim-only region.
    pub action: Option<String>,
    /// The runtime action run when no handler is bound to [`Self::action`].
    pub default_action: Option<String>,
    /// The hitbox item filling the region's slots; `None` leaves them empty.
    pub hitbox: Option<Hitbox>,
    /// Repeater metadata for grouped codegen, if this region was emitted by a repeated template.
    pub repeat: Option<RepeatBindingIr>,
    /// The authored element this region comes from, such as ``button `buy` ``.
    pub source: String,
}

/// A dynamic inventory item region.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemIr {
    /// Unique name within the window.
    pub name: String,
    /// Backing inventory slots populated by the runtime.
    pub slots: Vec<InventorySlotRef>,
    /// Repeater metadata for grouped codegen, if this item was emitted by a
    /// repeated template.
    pub repeat: Option<RepeatBindingIr>,
}

/// A dynamic repeated inventory item region.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionIr {
    /// Unique name within the window.
    pub name: String,
    /// Backing inventory slots, one cell per slot in authoring order.
    pub slots: Vec<InventorySlotRef>,
    /// Theme sprite drawn over the selected cell's 18x18 box, if any.
    pub selected_sprite: Option<String>,
    /// Whether codegen/runtime should expect a click handler.
    pub action: bool,
    /// Repeater metadata for grouped codegen, if this collection was emitted by
    /// a repeated template.
    pub repeat: Option<RepeatBindingIr>,
}

/// A native anvil text-input binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnvilInputIr {
    /// Unique input name within the window.
    pub name: String,
    /// Initial text placed into the vanilla rename field.
    pub initial: String,
    /// Optional item model used by the input-slot seed item.
    pub item_model: Option<String>,
}

/// Metadata attached to flattened controls emitted from a repeater template.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepeatBindingIr {
    /// Repeater group name.
    pub group: String,
    /// Repeated child field name, if this is a named child. `None` means the
    /// cell/root control itself.
    pub field: Option<String>,
    /// Zero-based cell index in authoring order.
    pub index: u32,
}

/// A runtime-selected group of cases, of which at most one is active.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwitchIr {
    /// Unique switch key: its binding name, or `{binding}.{case path}` for a copy inside switch cases.
    pub name: String,
    /// The binding a copy shares with the other copies; `None` when it equals [`Self::name`].
    pub binding: Option<String>,
    /// Whether this switch selects the named states of the control [`Self::name`] instead of a binding.
    pub states: bool,
    /// The case active while the switch is unbound; `None` means no case.
    pub initial: Option<String>,
    /// The authored element this switch comes from, for derived switches such as a button's states.
    pub source: Option<String>,
    /// Cases in authoring order.
    pub cases: Vec<SwitchCaseIr>,
}

/// One case of a [`SwitchIr`]: everything drawn or claimed only while it is active.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SwitchCaseIr {
    /// Case value.
    pub value: String,
    /// Static draws of this case in paint order, kept out of the surface's static layer.
    pub draws: Vec<Draw>,
    /// Names of the text regions (labels and dynamic slots) directly inside this case.
    pub slots: Vec<String>,
    /// Names of the runtime sprite regions directly inside this case.
    pub sprite_slots: Vec<String>,
    /// Names of the inventory regions directly inside this case.
    pub regions: Vec<String>,
    /// Names of the switches directly inside this case.
    pub switches: Vec<String>,
}

/// One runtime-drawn layer; a surface's layers compose in authored tree order above its static chrome.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "name", rename_all = "snake_case")]
pub enum Layer {
    /// A text slot or static label.
    Slot(String),
    /// A runtime sprite slot.
    SpriteSlot(String),
    /// The baked art of a switch's active case.
    Switch(String),
    /// A collection's selected-cell sprite.
    Collection(String),
}

/// A fully laid-out window, ready for the backend.
#[derive(Clone, Debug)]
pub struct LaidOutWindow {
    /// Window name (manifest key).
    pub name: String,
    /// Target surface.
    pub surface: Surface,
    /// Static draws in paint order (back to front).
    pub draws: Vec<Draw>,
    /// Text regions (dynamic slots and static labels), document order.
    pub slots: Vec<SlotIr>,
    /// Runtime sprite regions, document order.
    pub sprite_slots: Vec<SpriteSlotIr>,
    /// Inventory regions, document order.
    pub regions: Vec<RegionIr>,
    /// Dynamic inventory item regions, document order.
    pub items: Vec<ItemIr>,
    /// Dynamic repeated inventory item regions, document order.
    pub collections: Vec<CollectionIr>,
    /// Native text inputs, document order. Anvil surfaces support one.
    pub inputs: Vec<AnvilInputIr>,
    /// Runtime-selected cases, document order.
    pub switches: Vec<SwitchIr>,
    /// Runtime layers in authored tree order.
    pub layers: Vec<Layer>,
    /// Indexed binding families by name.
    pub indexed: BTreeMap<String, IndexedBinding>,
    /// Typed handles by id.
    pub handles: BTreeMap<String, Handle>,
    /// Non-fatal findings to surface to the user (e.g. overlay overflow).
    pub warnings: Vec<String>,
}

/// A fully laid-out HUD, ready for the backend.
#[derive(Clone, Debug)]
pub struct LaidOutHud {
    /// HUD name (manifest key).
    pub name: String,
    /// Vanilla transport channel.
    pub channel: HudChannel,
    /// Fixed line/canvas width in GUI pixels. Runtime-composed HUD components
    /// always claim this width so actionbar/bossbar/sidebar centering remains
    /// stable across updates.
    pub width: u32,
    /// Informational canvas height in GUI pixels.
    pub height: u32,
    /// Optional shader relocation settings.
    pub shader: Option<HudShader>,
    /// Static draws in paint order (back to front).
    pub draws: Vec<Draw>,
    /// Text regions (dynamic slots and static labels), document order.
    pub slots: Vec<SlotIr>,
    /// Runtime-selected cases, document order.
    pub switches: Vec<SwitchIr>,
    /// Runtime layers in authored tree order.
    pub layers: Vec<Layer>,
    /// Indexed binding families by name.
    pub indexed: BTreeMap<String, IndexedBinding>,
    /// Typed handles by id.
    pub handles: BTreeMap<String, Handle>,
    /// Non-fatal findings to surface to the user.
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_hex_round_trip() {
        let c = Rgb::parse_hex("#FFd700").unwrap();
        assert_eq!(c, Rgb { r: 0xff, g: 0xd7, b: 0x00 });
        assert_eq!(c.to_hex(), "#ffd700");
        assert!(Rgb::parse_hex("ffd700").is_none());
        assert!(Rgb::parse_hex("#ffd70").is_none());
    }
}
