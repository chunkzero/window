//! The authored UI model, after authoring data has been decoded and before
//! layout. See `docs/AUTHORING.md` for the source format.

use std::collections::{BTreeMap, BTreeSet};

use crate::geometry::{Insets, Point, Size};
use crate::inventory::{InventorySlotRef, SlotPattern};
use crate::ir::{Align, Handle, HudChannel, HudShader, Rgb};
use crate::surface::ContainerKind;

mod flex;
mod generated;

pub use flex::{FlexBox, ItemLayout, LayoutChild, Region, SlotSection, Switch, SwitchCase};
pub use generated::{GeneratedKind, GeneratedStyle};

/// The art a project draws, by interned name: inline art values and the runtime sprite catalog.
#[derive(Clone, Debug, Default)]
pub struct Art {
    /// Frames stretched over laid-out rects.
    pub frames: BTreeMap<String, Frame>,
    /// Images and runtime sprites drawn at their own size.
    pub sprites: BTreeMap<String, SpriteDef>,
    /// The sprites the runtime draws: the catalog and collection selections. Other sprites are only baked into
    /// static art.
    pub runtime: BTreeSet<String>,
}

/// A frame definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Frame {
    /// Stretch a pack-source PNG as a nine-slice texture.
    Texture {
        /// Pack-source-relative texture path.
        texture: String,
        /// Fixed border widths within the texture.
        insets: Insets,
    },
    /// Render the frame from generated style parameters during compilation.
    Generated(GeneratedStyle),
}

/// A sprite definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpriteDef {
    /// Blit a bitmap sprite.
    Texture {
        /// Pack-source-relative texture path, or a Minecraft font-provider
        /// texture id such as `example:tool/wooden_pickaxe.png`.
        texture: String,
        /// Explicit rendered size. Required for external texture ids because
        /// Window cannot decode them during compilation.
        size: Option<Size>,
    },
    /// Render the sprite at a fixed size from generated style parameters.
    Generated {
        /// Intrinsic sprite size.
        size: Size,
        /// Generated sprite style.
        style: GeneratedStyle,
    },
}

/// A bitmap text font: a glyph sheet whose cells the `chars` rows map to characters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontDef {
    /// Pack-source-relative texture path.
    pub texture: String,
    /// One string per sheet row; `\0` marks an empty cell.
    pub chars: Vec<String>,
}

/// One authored window.
#[derive(Clone, Debug)]
pub struct Window {
    /// Window name (manifest key); from the `window` node's argument.
    pub name: String,
    /// Target container kind.
    pub container: ContainerKind,
    /// Extra visual overflow allowed outside the container GUI.
    pub bleed: Insets,
    /// Frame drawn first, over the GUI rect grown by `bleed`.
    pub frame: Option<String>,
    /// The root elements, laid out in the GUI rect.
    pub children: Vec<Element>,
    /// Typed handles by id; their elements carry the entry names of their uses.
    pub handles: BTreeMap<String, Handle>,
    /// Authored name shown in errors and diagnostics.
    pub debug_name: Option<String>,
}

/// One authored HUD.
#[derive(Clone, Debug)]
pub struct Hud {
    /// HUD name (manifest key).
    pub name: String,
    /// Vanilla transport channel used by the runtime.
    pub channel: HudChannel,
    /// Fixed canvas/line size; `None` sizes the HUD to the extent of its children.
    pub size: Option<Size>,
    /// Extra visual overflow allowed outside the HUD canvas.
    pub bleed: Insets,
    /// Frame drawn first, over the HUD rect grown by `bleed`.
    pub frame: Option<String>,
    /// Optional core-shader relocation settings.
    pub shader: Option<HudShader>,
    /// The root elements. HUDs share the same visual primitives as windows, but
    /// slot-bound elements are rejected during layout.
    pub children: Vec<Element>,
    /// Typed handles by id; their elements carry the entry names of their uses.
    pub handles: BTreeMap<String, Handle>,
    /// Authored name shown in errors and diagnostics.
    pub debug_name: Option<String>,
}

/// Common text styling for labels and slots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextStyle {
    /// Horizontal alignment within the reserved width.
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
    /// The project or bundled text font to draw with, or `None` for vanilla glyphs.
    pub font: Option<String>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            align: Align::Left,
            color: Rgb::DEFAULT_TEXT,
            shadow: false,
            bold: false,
            italic: false,
            underlined: false,
            strikethrough: false,
            obfuscated: false,
            font: None,
        }
    }
}

/// How a dynamic text slot fits runtime content wider than its width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextFit {
    /// Whether overflowing content ends in an ellipsis. Always set when `lines > 1`.
    pub ellipsis: bool,
    /// Maximum number of lines the content wraps onto, at least 1.
    pub lines: u32,
    /// Distance between the tops of consecutive lines, in pixels.
    pub line_height: u32,
}

impl TextFit {
    /// Vanilla line spacing.
    pub const DEFAULT_LINE_HEIGHT: u32 = 9;

    /// The height of the slot's box: `lines` lines of 8px text spaced `line_height` apart.
    pub fn height(self) -> u32 {
        (self.lines - 1) * self.line_height + 8
    }
}

impl Default for TextFit {
    fn default() -> Self {
        Self { ellipsis: false, lines: 1, line_height: Self::DEFAULT_LINE_HEIGHT }
    }
}

/// An element of the authored tree.
#[derive(Clone, Debug)]
pub enum Element {
    /// Static art drawn at its own size.
    Sprite {
        /// Interned art name.
        name: String,
        /// Explicit position relative to the parent content origin.
        pos: Option<Point>,
        /// Authored name shown in errors and diagnostics.
        debug_name: Option<String>,
    },
    /// A runtime-selected sprite region. The compiler emits the sprite catalog
    /// into y-shifted fonts and the runtime binds this region by name.
    SpriteSlot {
        /// Runtime sprite slot name.
        name: String,
        /// Reserved region for horizontal alignment and bounds checks.
        size: Size,
        /// Explicit position relative to the parent content origin.
        pos: Option<Point>,
        /// Horizontal alignment of the chosen sprite within `size`.
        align: Align,
        /// Authored name shown in errors and diagnostics.
        debug_name: Option<String>,
    },
    /// A dynamic inventory item region with no click handler.
    Item {
        /// Item control name.
        name: String,
        /// Explicit backing inventory slots populated by the runtime.
        slots: Option<Vec<InventorySlotRef>>,
        /// Slot pattern populated by the runtime.
        pattern: Option<SlotPattern>,
        /// Authored name shown in errors and diagnostics.
        debug_name: Option<String>,
    },
    /// A repeated dynamic item region with optional click handling.
    Collection {
        /// Collection control name.
        name: String,
        /// Explicit backing inventory slots, one cell per slot in authoring order.
        slots: Option<Vec<InventorySlotRef>>,
        /// Slot pattern, flattened into cells in authoring order.
        pattern: Option<SlotPattern>,
        /// Optional frame drawn around every resolved inventory cell.
        frame: Option<String>,
        /// Optional sprite the runtime draws over the selected cell's 18x18 box.
        selected_sprite: Option<String>,
        /// Whether codegen/runtime should expect a click handler.
        action: bool,
        /// Authored name shown in errors and diagnostics.
        debug_name: Option<String>,
    },
    /// Native anvil rename-field binding. Only valid on an `anvil` surface.
    AnvilInput {
        /// Input name used by generated/runtime bindings.
        name: String,
        /// Initial text placed into the vanilla rename field.
        initial: String,
        /// Optional item model used by the input-slot seed item.
        item_model: Option<String>,
        /// Authored name shown in errors and diagnostics.
        debug_name: Option<String>,
    },
    /// Static text, baked into the compiled definition as a constant slot.
    Label {
        /// The text.
        text: String,
        /// Reserved width; defaults to the measured text width.
        width: Option<u32>,
        /// Explicit position relative to the parent content origin.
        pos: Option<Point>,
        /// Styling.
        style: TextStyle,
        /// Authored name shown in errors and diagnostics.
        debug_name: Option<String>,
    },
    /// A dynamic text region.
    Slot {
        /// Slot name (becomes a manifest slot).
        name: String,
        /// Reserved width. In a box, a slot without one fills the box's width.
        width: Option<u32>,
        /// Explicit position relative to the parent content origin.
        pos: Option<Point>,
        /// Styling.
        style: TextStyle,
        /// Runtime fitting of overflowing content.
        fit: TextFit,
        /// Authored name shown in errors and diagnostics.
        debug_name: Option<String>,
    },
    /// An inventory region: the slots it covers take its clicks and show its hitbox item.
    Region(Box<Region>),
    /// A taffy flexbox or grid box.
    Flex(Box<FlexBox>),
    /// A slot section laid out as a taffy grid of inventory slots.
    Section(Box<SlotSection>),
    /// Runtime-selected visual cases stacked in one box.
    Switch(Box<Switch>),
}
