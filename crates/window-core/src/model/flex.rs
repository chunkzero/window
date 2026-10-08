//! Taffy-solved layout containers authored through the JSX front end.

use crate::geometry::{Insets, Point, Size};
use crate::inventory::{InventorySlotSection, SlotRectClaim};
use crate::ir::Tooltip;

use super::Element;

/// A flexbox or grid box. Its children are sized and positioned by taffy.
///
/// An auto-sized box placed directly in a known content box (the window, a HUD, a panel, a button, a
/// repeater cell, or a section cell) fills that box; elsewhere it takes its max-content size.
#[derive(Clone, Debug)]
pub struct FlexBox {
    /// Explicit position relative to the parent content origin; inside another box it positions absolutely.
    pub pos: Option<Point>,
    /// Frame drawn over the box's border rect before its children.
    pub frame: Option<String>,
    /// Container style, including the box's own size.
    pub style: taffy::Style,
    pub children: Vec<LayoutChild>,
    /// Authored name shown in errors and diagnostics.
    pub debug_name: Option<String>,
}

/// A slot section laid out as a taffy grid with one track per inventory slot.
///
/// Children auto-flow through the grid. A control's slot pattern is relative to its grid area, and an
/// unpatterned control covers the whole area.
#[derive(Clone, Debug)]
pub struct SlotSection {
    pub section: InventorySlotSection,
    /// Frame drawn around the section's 18x18 slot boxes, grown by `outset`.
    pub frame: Option<String>,
    pub outset: Insets,
    /// Claim applied to the section's slots that no child owns; `None` leaves them to vanilla.
    pub claim: SlotRectClaim,
    pub flow: taffy::GridAutoFlow,
    pub children: Vec<LayoutChild>,
    /// Authored name shown in errors and diagnostics.
    pub debug_name: Option<String>,
}

/// Visual cases stacked in one box that takes the largest case's size; the runtime draws only the case
/// named by the `name` binding.
#[derive(Clone, Debug)]
pub struct Switch {
    /// Binding name of the active case value.
    pub name: String,
    /// Explicit position relative to the parent content origin; inside a box it positions absolutely.
    pub pos: Option<Point>,
    pub cases: Vec<SwitchCase>,
    /// Authored name shown in errors and as the switch's diagnostic source.
    pub debug_name: Option<String>,
}

/// An inventory region: the slots its rect covers take its clicks and show its hitbox item.
///
/// Without a size it fills its parent box; in a section it covers its grid area.
#[derive(Clone, Debug)]
pub struct Region {
    /// The click entry it routes to, or `None` for a hover-only region, which layout names `region_N`.
    pub name: Option<String>,
    /// The runtime action run when no handler is bound to the click entry, such as `window:close`.
    pub default_action: Option<String>,
    /// Fixed size; `None` fills the parent box.
    pub size: Option<Size>,
    pub tooltip: Option<Tooltip>,
    /// Item model of the hitbox item filling its slots.
    pub item_model: Option<String>,
    /// Authored name shown in errors and as the region's diagnostic source.
    pub debug_name: Option<String>,
}

/// One case of a [`Switch`]: a box that fills the switch.
#[derive(Clone, Debug)]
pub struct SwitchCase {
    pub value: String,
    pub body: FlexBox,
}

/// A child of a [`FlexBox`] or [`SlotSection`] with its item-level layout.
#[derive(Clone, Debug)]
pub struct LayoutChild {
    pub layout: ItemLayout,
    pub element: Element,
}

/// Item-level layout properties. Unset fields keep taffy's defaults.
#[derive(Clone, Debug, Default)]
pub struct ItemLayout {
    pub grow: Option<f32>,
    pub shrink: Option<f32>,
    pub basis: Option<taffy::Dimension>,
    pub align_self: Option<taffy::AlignSelf>,
    pub justify_self: Option<taffy::AlignSelf>,
    pub margin: Option<taffy::Rect<taffy::LengthPercentageAuto>>,
    pub absolute: bool,
    pub inset: Option<taffy::Rect<taffy::LengthPercentageAuto>>,
    pub min_size: Option<taffy::Size<taffy::LengthPercentageAuto>>,
    pub max_size: Option<taffy::Size<taffy::LengthPercentageAuto>>,
    pub column: Option<taffy::Line<taffy::GridPlacement>>,
    pub row: Option<taffy::Line<taffy::GridPlacement>>,
    /// Pixel offset applied after layout to the element and its subtree.
    pub translate: Point,
}

impl ItemLayout {
    /// Copy the set properties onto `style`.
    pub fn apply(&self, style: &mut taffy::Style) {
        if let Some(grow) = self.grow {
            style.flex_grow = grow;
        }
        if let Some(shrink) = self.shrink {
            style.flex_shrink = shrink;
        }
        if let Some(basis) = self.basis {
            style.flex_basis = basis;
        }
        if self.align_self.is_some() {
            style.align_self = self.align_self;
        }
        if self.justify_self.is_some() {
            style.justify_self = self.justify_self;
        }
        if let Some(margin) = self.margin {
            style.margin = margin;
        }
        if self.absolute {
            style.position = taffy::Position::Absolute;
        }
        if let Some(inset) = self.inset {
            style.inset = inset;
        }
        if let Some(min) = self.min_size {
            style.min_size = min;
        }
        if let Some(max) = self.max_size {
            style.max_size = max;
        }
        if let Some(column) = self.column.clone() {
            style.grid_column = column;
        }
        if let Some(row) = self.row.clone() {
            style.grid_row = row;
        }
    }
}
