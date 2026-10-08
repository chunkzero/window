//! Render surfaces a window can target.
//!
//! Window supports generic container screens (`generic_9x1` … `generic_9x6`)
//! and the anvil screen used for native text entry. Modern clients lay generic
//! screens out programmatically:
//! `gui_height = 114 + rows * 18`, title cursor at `(8, 6)`, container slot
//! grid at `(8, 18)` with an 18px stride, 9 columns, player inventory at
//! `(8, 31 + rows * 18)`, and hotbar 58px below it.

use crate::geometry::{Point, Rect, Size};
use crate::inventory::{InventorySlotArea, InventorySlotRef, InventorySlotSection};

/// Where a window is rendered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Surface {
    /// A container (inventory) screen; the UI rides in the window title.
    Container(ContainerKind),
}

impl Surface {
    /// The title cursor origin in GUI space.
    pub fn title_origin(&self) -> Point {
        match self {
            Surface::Container(kind) => kind.title_origin(),
        }
    }

    /// The GUI texture size in pixels.
    pub fn gui_size(&self) -> Size {
        match self {
            Surface::Container(kind) => kind.gui_size(),
        }
    }
}

/// Supported inventory screen kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ContainerKind {
    /// `generic_9x1` (one container row).
    Generic9x1,
    /// `generic_9x2`.
    Generic9x2,
    /// `generic_9x3` (single chest).
    Generic9x3,
    /// `generic_9x4`.
    Generic9x4,
    /// `generic_9x5`.
    Generic9x5,
    /// `generic_9x6` (double chest).
    Generic9x6,
    /// Vanilla anvil screen with one native rename text field.
    Anvil,
}

impl ContainerKind {
    /// All kinds, ascending by row count.
    pub const ALL: [ContainerKind; 7] = [
        ContainerKind::Generic9x1,
        ContainerKind::Generic9x2,
        ContainerKind::Generic9x3,
        ContainerKind::Generic9x4,
        ContainerKind::Generic9x5,
        ContainerKind::Generic9x6,
        ContainerKind::Anvil,
    ];

    /// Parse an id like `generic_9x6`.
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|k| k.id() == s)
    }

    /// The stable id used in authoring and the manifest.
    pub const fn id(&self) -> &'static str {
        match self {
            ContainerKind::Generic9x1 => "generic_9x1",
            ContainerKind::Generic9x2 => "generic_9x2",
            ContainerKind::Generic9x3 => "generic_9x3",
            ContainerKind::Generic9x4 => "generic_9x4",
            ContainerKind::Generic9x5 => "generic_9x5",
            ContainerKind::Generic9x6 => "generic_9x6",
            ContainerKind::Anvil => "anvil",
        }
    }

    /// Number of container rows.
    pub const fn rows(&self) -> u32 {
        match self {
            ContainerKind::Generic9x1 => 1,
            ContainerKind::Generic9x2 => 2,
            ContainerKind::Generic9x3 => 3,
            ContainerKind::Generic9x4 => 4,
            ContainerKind::Generic9x5 => 5,
            ContainerKind::Generic9x6 => 6,
            ContainerKind::Anvil => 1,
        }
    }

    /// Number of logical container columns.
    pub const fn columns(&self) -> u32 {
        match self {
            ContainerKind::Anvil => 3,
            _ => 9,
        }
    }

    /// Container slot count (excludes the player inventory).
    pub const fn slot_count(&self) -> u32 {
        match self {
            ContainerKind::Anvil => 3,
            _ => self.rows() * self.columns(),
        }
    }

    /// GUI texture size: `176 × (114 + rows * 18)`.
    pub const fn gui_size(&self) -> Size {
        match self {
            ContainerKind::Anvil => Size::new(176, 166),
            _ => Size::new(176, 114 + self.rows() * 18),
        }
    }

    /// GUI rect of vanilla's anvil text-field sprite; the rename box draws inside it.
    pub(crate) const ANVIL_TEXT_FIELD: Rect = Rect::new(59, 20, 110, 16);

    /// Where the title cursor starts, in GUI space.
    pub const fn title_origin(&self) -> Point {
        match self {
            ContainerKind::Anvil => Point::new(60, 6),
            _ => Point::new(8, 6),
        }
    }

    /// Top-left of the container slot grid, in GUI space.
    pub const fn slot_grid_origin(&self) -> Point {
        match self {
            ContainerKind::Anvil => Point::new(27, 47),
            _ => Point::new(8, 18),
        }
    }

    /// The GUI-space rect of a container slot's 16×16 interior
    /// (index is row-major from the top-left).
    pub fn slot_rect(&self, index: u32) -> Option<Rect> {
        if index >= self.slot_count() {
            return None;
        }
        if *self == ContainerKind::Anvil {
            let x = match index {
                0 => 27,
                1 => 76,
                2 => 134,
                _ => return None,
            };
            return Some(Rect::new(x, 47, 16, 16));
        }
        let origin = self.slot_grid_origin();
        let col = (index % self.columns()) as i32;
        let row = (index / self.columns()) as i32;
        Some(Rect::new(origin.x + col * 18, origin.y + row * 18, 16, 16))
    }

    /// Container slot indices whose interiors overlap `rect`, ascending.
    pub fn slots_overlapping(&self, rect: &Rect) -> Vec<u32> {
        (0..self.slot_count()).filter(|&i| self.slot_rect(i).is_some_and(|slot| slot.intersects(rect))).collect()
    }

    /// Top-left of the player's 3x9 inventory grid in this screen.
    pub const fn player_inventory_origin(&self) -> Point {
        match self {
            ContainerKind::Anvil => Point::new(8, 84),
            _ => Point::new(8, 18 + self.rows() as i32 * 18 + 13),
        }
    }

    /// Top-left of the player's hotbar in this screen.
    pub const fn player_hotbar_origin(&self) -> Point {
        match self {
            ContainerKind::Anvil => Point::new(8, 142),
            _ => {
                let main = self.player_inventory_origin();
                Point::new(main.x, main.y + 58)
            }
        }
    }

    /// The GUI-space rect of a visible player inventory slot.
    ///
    /// Uses Minestom player-inventory indices: `0..=8` hotbar and `9..=35`
    /// main inventory. Armor, crafting, and offhand slots are not visible in a
    /// generic chest screen, so they return `None`.
    pub fn player_slot_rect(&self, index: u32) -> Option<Rect> {
        if index < 9 {
            let origin = self.player_hotbar_origin();
            return Some(Rect::new(origin.x + index as i32 * 18, origin.y, 16, 16));
        }
        if (9..36).contains(&index) {
            let origin = self.player_inventory_origin();
            let local = index - 9;
            let col = (local % 9) as i32;
            let row = (local / 9) as i32;
            return Some(Rect::new(origin.x + col * 18, origin.y + row * 18, 16, 16));
        }
        None
    }

    /// Section dimensions in zero-based authoring coordinates.
    pub const fn section_size(&self, section: InventorySlotSection) -> Size {
        match section {
            InventorySlotSection::Container => Size::new(self.columns(), self.rows()),
            InventorySlotSection::Player => Size::new(9, 3),
            InventorySlotSection::Hotbar => Size::new(9, 1),
        }
    }

    /// Bounds of every visible slot interior in one inventory section.
    pub fn section_bounds(&self, section: InventorySlotSection) -> Rect {
        let size = self.section_size(section);
        let mut bounds = self.section_slot_rect(section, 0).expect("visible inventory sections are non-empty");
        for index in 1..size.width * size.height {
            let rect = self.section_slot_rect(section, index).expect("section dimensions only include visible slots");
            bounds = bounds.union(&rect);
        }
        bounds
    }

    /// Resolve a zero-based local section slot index to its backing inventory slot.
    pub fn section_slot_ref(&self, section: InventorySlotSection, local_index: u32) -> Option<InventorySlotRef> {
        let size = self.section_size(section);
        if local_index >= size.width * size.height {
            return None;
        }
        match section {
            InventorySlotSection::Container => Some(InventorySlotRef::container(local_index)),
            InventorySlotSection::Player => Some(InventorySlotRef::player(local_index + 9)),
            InventorySlotSection::Hotbar => Some(InventorySlotRef::player(local_index)),
        }
    }

    /// GUI-space rect of a zero-based local section slot.
    pub fn section_slot_rect(&self, section: InventorySlotSection, local_index: u32) -> Option<Rect> {
        self.section_slot_ref(section, local_index).and_then(|slot| self.slot_ref_rect(slot))
    }

    /// GUI-space rect for a resolved backing slot.
    pub fn slot_ref_rect(&self, slot: InventorySlotRef) -> Option<Rect> {
        match slot.area {
            InventorySlotArea::Container => self.slot_rect(slot.index),
            InventorySlotArea::Player => self.player_slot_rect(slot.index),
        }
    }

    /// Resolve a zero-based slot-space rectangle into backing slots, row-major.
    pub fn section_rect_slots(
        &self,
        section: InventorySlotSection,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Option<Vec<InventorySlotRef>> {
        let size = self.section_size(section);
        if width == 0
            || height == 0
            || x.checked_add(width).is_none_or(|right| right > size.width)
            || y.checked_add(height).is_none_or(|bottom| bottom > size.height)
        {
            return None;
        }
        let mut out = Vec::with_capacity((width * height) as usize);
        for row in y..y + height {
            for col in x..x + width {
                let index = row * size.width + col;
                out.push(self.section_slot_ref(section, index)?);
            }
        }
        Some(out)
    }

    /// GUI-space union of the slot interiors in a zero-based slot-space rect.
    pub fn section_rect_bounds(
        &self,
        section: InventorySlotSection,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Option<Rect> {
        let slots = self.section_rect_slots(section, x, y, width, height)?;
        self.slot_ref_bounds(&slots)
    }

    /// GUI-space union of resolved backing slot interiors.
    pub fn slot_ref_bounds(&self, slots: &[InventorySlotRef]) -> Option<Rect> {
        let mut iter = slots.iter();
        let first = self.slot_ref_rect(*iter.next()?)?;
        let mut rect = first;
        for slot in iter {
            rect = rect.union(&self.slot_ref_rect(*slot)?);
        }
        Some(rect)
    }

    /// Inventory slot references whose interiors overlap `rect`, ascending in
    /// screen order: container slots, main inventory, then hotbar.
    pub fn slot_refs_overlapping(&self, rect: &Rect) -> Vec<InventorySlotRef> {
        let mut out: Vec<InventorySlotRef> =
            self.slots_overlapping(rect).into_iter().map(InventorySlotRef::container).collect();
        for i in 9..36 {
            if self.player_slot_rect(i).is_some_and(|slot| slot.intersects(rect)) {
                out.push(InventorySlotRef::player(i));
            }
        }
        for i in 0..9 {
            if self.player_slot_rect(i).is_some_and(|slot| slot.intersects(rect)) {
                out.push(InventorySlotRef::player(i));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests;
