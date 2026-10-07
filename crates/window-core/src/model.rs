//! The authored UI model, after authoring data has been decoded and before
//! layout. See `docs/AUTHORING.md` for the source format.

use std::collections::BTreeMap;

use crate::geometry::{Insets, Point, Size};
use crate::inventory::{InventorySlotRef, SlotPattern, SlotRectClaim};
use crate::ir::{Align, ButtonDefault, ButtonState, ButtonTooltip, HudChannel, HudShader, IndexedBinding, Rgb};
use crate::surface::ContainerKind;

mod flex;
mod generated;

pub use flex::{FlexBox, ItemLayout, LayoutChild, SlotSection, Switch, SwitchCase};
pub use generated::{GeneratedKind, GeneratedStyle};

/// All theme definitions across every theme document (names are global).
#[derive(Clone, Debug, Default)]
pub struct Theme {
    /// Frame styles by name.
    pub frames: BTreeMap<String, Frame>,
    /// Sprite styles by name.
    pub sprites: BTreeMap<String, SpriteDef>,
    /// Bitmap text fonts by name.
    pub fonts: BTreeMap<String, FontDef>,
    /// Palette colors by name, emitted for runtime code.
    pub colors: BTreeMap<String, Rgb>,
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
    /// Render the frame from theme parameters during compilation.
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
    /// Render the sprite at a fixed size from theme parameters.
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
    /// The root element (the `window` node's single child in practice, but
    /// any number of children is allowed — they are laid out like a panel's).
    pub children: Vec<Element>,
    /// Indexed binding families by name; their elements carry the flattened names.
    pub indexed: BTreeMap<String, IndexedBinding>,
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
    /// button and hotspot elements are rejected during layout.
    pub children: Vec<Element>,
    /// Indexed binding families by name; their elements carry the flattened names.
    pub indexed: BTreeMap<String, IndexedBinding>,
}

/// Cross-axis alignment for `row`/`column` children.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CrossAlign {
    /// Align to the start of the cross axis.
    #[default]
    Start,
    /// Center on the cross axis.
    Center,
    /// Align to the end of the cross axis.
    End,
}

/// Common text styling for labels and slots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextStyle {
    /// Horizontal alignment within the reserved width. Omitted text inherits
    /// the surrounding layout default (left normally, centered in buttons).
    pub align: Option<Align>,
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
    /// The theme or bundled text font to draw with, or `None` for vanilla glyphs.
    pub font: Option<String>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            align: None,
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
    /// A framed container with explicit size.
    Panel {
        /// Theme frame name.
        frame: String,
        /// Explicit position (overrides flow when inside a row/column).
        pos: Option<Point>,
        /// Explicit size.
        size: Size,
        /// Inner padding.
        padding: u32,
        /// Children, placed inside the padding box.
        children: Vec<Element>,
    },
    /// A horizontal flow container.
    Row {
        /// Explicit position (overrides flow).
        pos: Option<Point>,
        /// Gap between children on the main axis.
        gap: u32,
        /// Inner padding.
        padding: u32,
        /// Cross-axis alignment of children.
        align: CrossAlign,
        /// Children.
        children: Vec<Element>,
    },
    /// A vertical flow container.
    Column {
        /// Explicit position (overrides flow).
        pos: Option<Point>,
        /// Gap between children on the main axis.
        gap: u32,
        /// Inner padding.
        padding: u32,
        /// Cross-axis alignment of children.
        align: CrossAlign,
        /// Children.
        children: Vec<Element>,
    },
    /// A theme sprite at intrinsic size.
    Sprite {
        /// Theme sprite name.
        name: String,
        /// Explicit position (overrides flow).
        pos: Option<Point>,
    },
    /// A runtime-selected sprite region. The compiler emits the sprite catalog
    /// into y-shifted fonts and the runtime binds this region by name.
    SpriteSlot {
        /// Runtime sprite slot name.
        name: String,
        /// Reserved region for horizontal alignment and bounds checks.
        size: Size,
        /// Explicit position (overrides flow).
        pos: Option<Point>,
        /// Horizontal alignment of the chosen sprite within `size`.
        align: Align,
        /// Fixed sprite id, or `None` for a runtime-selected sprite.
        sprite: Option<String>,
    },
    /// A clickable region with optional frame and visual children.
    Button {
        /// Button name (becomes a manifest button).
        name: String,
        /// Optional theme frame drawn behind the children.
        frame: Option<String>,
        /// Explicit position (overrides flow).
        pos: Option<Point>,
        /// Explicit size. Pattern-backed buttons may omit it.
        size: Option<Size>,
        /// Explicit backing inventory slots. Prefer `pattern`; this remains an
        /// escape hatch and for compatibility with low-level layouts.
        slots: Option<Vec<InventorySlotRef>>,
        /// Slot pattern that supplies both backing slots and, when `x`/`y` or
        /// `width`/`height` are omitted, the button rect.
        pattern: Option<SlotPattern>,
        /// Inner padding.
        padding: u32,
        /// Built-in default behavior, if any.
        default: Option<ButtonDefault>,
        /// Default tooltip shown when hovering the button.
        tooltip: Option<ButtonTooltip>,
        /// Named inventory item states for dynamic visual/tooltip toggles.
        states: BTreeMap<String, ButtonState>,
        /// Visual children (centered content lives here).
        children: Vec<Element>,
    },
    /// A non-rendered hover/click region, useful for tooltip-only areas.
    Hotspot {
        /// Hotspot name (becomes a manifest button without a generated handler).
        name: String,
        /// Explicit position (overrides flow).
        pos: Option<Point>,
        /// Explicit size. Pattern-backed hotspots may omit it.
        size: Option<Size>,
        /// Explicit backing inventory slots. Prefer `pattern`; this remains an
        /// escape hatch and for compatibility with low-level layouts.
        slots: Option<Vec<InventorySlotRef>>,
        /// Slot pattern that supplies both backing slots and, when `x`/`y` or
        /// `width`/`height` are omitted, the hotspot rect.
        pattern: Option<SlotPattern>,
        /// Default tooltip shown when hovering the hotspot.
        tooltip: Option<ButtonTooltip>,
        /// Named inventory item states for dynamic visual/tooltip toggles.
        states: BTreeMap<String, ButtonState>,
    },
    /// A dynamic inventory item region with no click handler.
    Item {
        /// Item control name.
        name: String,
        /// Explicit backing inventory slots populated by the runtime.
        slots: Option<Vec<InventorySlotRef>>,
        /// Slot pattern populated by the runtime.
        pattern: Option<SlotPattern>,
        /// One-based index into the enclosing repeater cell's own slots. Only
        /// valid inside a [`Element::Repeater`], and mutually exclusive with
        /// `slots`/`pattern`.
        cell_slot: Option<u32>,
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
        /// Optional theme sprite the runtime draws over the selected cell's 18x18 box.
        selected_sprite: Option<String>,
        /// Whether codegen/runtime should expect a click handler.
        action: bool,
    },
    /// Native anvil rename-field binding. Only valid on an `anvil` surface.
    AnvilInput {
        /// Input name used by generated/runtime bindings.
        name: String,
        /// Initial text placed into the vanilla rename field.
        initial: String,
        /// Optional item model used by the input-slot seed item.
        item_model: Option<String>,
    },
    /// Draw one framed rectangle per slot in a pattern, optionally claiming
    /// backing slots without a Kotlin binding.
    SlotRects {
        /// Primitive name (manifest/debug only; no generated binding).
        name: String,
        /// Optional theme frame name. Omit for claim-only slot rects.
        frame: Option<String>,
        /// Slot pattern to draw and/or claim.
        pattern: SlotPattern,
        /// Claim/clear behavior.
        claim: SlotRectClaim,
    },
    /// Repeat a button/card template over a slot pattern grid, producing
    /// flattened controls plus grouped codegen metadata.
    Repeater {
        /// Repeater group name.
        name: String,
        /// Slot pattern whose resolved cells become repeated cards.
        pattern: SlotPattern,
        /// Optional theme frame drawn for each cell.
        frame: Option<String>,
        /// Inner padding for repeated children.
        padding: u32,
        /// Visual children placed relative to each repeated cell.
        children: Vec<Element>,
    },
    /// Static text, baked into the compiled definition as a constant slot.
    Label {
        /// The text.
        text: String,
        /// Reserved width; defaults to the measured text width.
        width: Option<u32>,
        /// Explicit position (overrides flow).
        pos: Option<Point>,
        /// Styling.
        style: TextStyle,
    },
    /// A dynamic text region (becomes an abstract member in codegen).
    Slot {
        /// Slot name (becomes a manifest slot).
        name: String,
        /// Reserved width. A direct, unpositioned button child may omit it and
        /// fill the button's padded content width.
        width: Option<u32>,
        /// Explicit position (overrides flow).
        pos: Option<Point>,
        /// Styling.
        style: TextStyle,
        /// Runtime fitting of overflowing content.
        fit: TextFit,
    },
    /// A taffy flexbox or grid box.
    Flex(Box<FlexBox>),
    /// A slot section laid out as a taffy grid of inventory slots.
    Section(Box<SlotSection>),
    /// Runtime-selected visual cases stacked in one box.
    Switch(Box<Switch>),
}
