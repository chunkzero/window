//! Inventory topology shared by authoring, layout, validation, and runtime codegen.
//!
//! This module deliberately contains no visual layout behavior. It identifies
//! backing inventories and describes reusable patterns in slot space; mapping
//! those slots to GUI pixels belongs to [`crate::surface`].

use serde::{Deserialize, Serialize};

/// Which backing inventory owns a slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InventorySlotArea {
    /// Slot in the opened container inventory.
    Container,
    /// Slot in the viewing player's own inventory, using Minestom indices.
    Player,
}

/// A visible inventory section in a generic container screen.
///
/// Section coordinates are zero-based. [`Self::Player`] is the visible 3x9
/// main inventory grid and [`Self::Hotbar`] is the visible 1x9 hotbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InventorySlotSection {
    /// The opened container grid.
    Container,
    /// The viewing player's visible main inventory grid.
    Player,
    /// The viewing player's visible hotbar.
    Hotbar,
}

/// A typed slot in either the open container or the player's inventory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InventorySlotRef {
    /// Backing inventory area.
    pub area: InventorySlotArea,
    /// Slot index in that area.
    pub index: u32,
}

impl InventorySlotRef {
    /// Construct a container slot reference.
    pub const fn container(index: u32) -> Self {
        Self { area: InventorySlotArea::Container, index }
    }

    /// Construct a player-inventory slot reference.
    pub const fn player(index: u32) -> Self {
        Self { area: InventorySlotArea::Player, index }
    }
}

/// A reusable slot-space pattern resolved against a concrete container.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlotPattern {
    /// A single rectangular cell group in a visible slot section.
    Rect(SlotRectPattern),
    /// Explicit local slot indices in one visible section.
    Slots {
        /// Visible slot section.
        section: InventorySlotSection,
        /// Local indices in that section.
        slots: Vec<u32>,
    },
    /// A repeated grid of rectangular cell groups.
    Grid(SlotGridPattern),
}

/// A zero-based slot-space rectangle in a visible section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotRectPattern {
    /// Visible slot section.
    pub section: InventorySlotSection,
    /// Left grid coordinate, zero-based.
    pub x: u32,
    /// Top grid coordinate, zero-based.
    pub y: u32,
    /// Width in slots.
    pub width: u32,
    /// Height in slots.
    pub height: u32,
}

/// A zero-based repeated grid of rectangular slot cell groups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotGridPattern {
    /// Visible slot section.
    pub section: InventorySlotSection,
    /// Left grid coordinate of the first cell group, zero-based.
    pub x: u32,
    /// Top grid coordinate of the first cell group, zero-based.
    pub y: u32,
    /// Number of cell groups per row.
    pub columns: u32,
    /// Number of cell-group rows.
    pub rows: u32,
    /// Width of each cell group in slots.
    pub cell_width: u32,
    /// Height of each cell group in slots.
    pub cell_height: u32,
}

/// Claim behavior for a drawn slot-rectangle primitive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SlotRectClaim {
    /// Draw only; do not claim or clear backing slots.
    #[default]
    None,
    /// Claim every slot, failing if another control owns one.
    All,
    /// Claim only slots not already owned by another control.
    Unowned,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_slot_areas_are_not_interchangeable() {
        assert_ne!(InventorySlotRef::container(4), InventorySlotRef::player(4));
    }

    #[test]
    fn slot_refs_sort_by_area_then_index() {
        let mut slots =
            vec![InventorySlotRef::player(0), InventorySlotRef::container(2), InventorySlotRef::container(1)];
        slots.sort();
        assert_eq!(
            slots,
            vec![InventorySlotRef::container(1), InventorySlotRef::container(2), InventorySlotRef::player(0),]
        );
    }
}
