use super::Solver;
use super::target::LayoutTarget;
use crate::Result;
use crate::error::Error;
use crate::geometry::Rect;
use crate::inventory::{InventorySlotRef, InventorySlotSection, SlotGridPattern, SlotPattern};
use crate::surface::ContainerKind;

pub(super) struct PatternCell {
    pub(super) rect: Rect,
    pub(super) slots: Vec<InventorySlotRef>,
}

/// The union of every cell rect, or `None` for no cells.
pub(super) fn cells_bounds(cells: &[PatternCell]) -> Option<Rect> {
    cells.iter().map(|cell| cell.rect).reduce(|a, b| a.union(&b))
}

fn section_label(section: InventorySlotSection) -> &'static str {
    match section {
        InventorySlotSection::Container => "container",
        InventorySlotSection::Player => "player",
        InventorySlotSection::Hotbar => "hotbar",
    }
}

impl<T: LayoutTarget> Solver<'_, T> {
    pub(super) fn resolve_required_slots(
        &self,
        name: &str,
        slots: Option<&Vec<InventorySlotRef>>,
        pattern: Option<&SlotPattern>,
    ) -> Result<Vec<InventorySlotRef>> {
        match (slots, pattern) {
            (Some(slots), _) => Ok(slots.clone()),
            (None, Some(pattern)) => self.resolve_pattern_slots(name, pattern),
            (None, None) => {
                Err(self.target.layout_err(format!("control `{name}` requires `slots`, `pattern`, or `transform`")))
            }
        }
    }

    pub(super) fn pattern_bounds(&self, name: &str, pattern: &SlotPattern) -> Result<Rect> {
        let cells = self.resolve_pattern_cells(name, pattern)?;
        cells_bounds(&cells).ok_or_else(|| self.target.layout_err(format!("pattern for `{name}` resolved to no slots")))
    }

    pub(super) fn resolve_pattern_slots(&self, name: &str, pattern: &SlotPattern) -> Result<Vec<InventorySlotRef>> {
        let mut slots = Vec::new();
        for cell in self.resolve_pattern_cells(name, pattern)? {
            slots.extend(cell.slots);
        }
        Ok(slots)
    }

    pub(super) fn resolve_pattern_cells(&self, name: &str, pattern: &SlotPattern) -> Result<Vec<PatternCell>> {
        let kind = self.target.container_kind().ok_or_else(|| self.target.slot_pattern_err(name))?;
        match pattern {
            SlotPattern::Rect(rect) => {
                let slots = kind
                    .section_rect_slots(rect.section, rect.x, rect.y, rect.width, rect.height)
                    .ok_or_else(|| self.pattern_outside_err(name, section_label(rect.section)))?;
                let rect = kind
                    .slot_ref_bounds(&slots)
                    .ok_or_else(|| self.target.layout_err(format!("pattern for `{name}` resolved to no slots")))?;
                Ok(vec![PatternCell { rect, slots }])
            }
            SlotPattern::Slots { section, slots } => {
                let mut out = Vec::with_capacity(slots.len());
                for slot in slots {
                    let slot_ref = kind
                        .section_slot_ref(*section, *slot)
                        .ok_or_else(|| self.pattern_outside_err(name, section_label(*section)))?;
                    let rect = kind
                        .slot_ref_rect(slot_ref)
                        .ok_or_else(|| self.pattern_outside_err(name, section_label(*section)))?;
                    out.push(PatternCell { rect, slots: vec![slot_ref] });
                }
                Ok(out)
            }
            SlotPattern::Grid(grid) => self.resolve_grid_cells(name, kind, grid),
        }
    }

    fn resolve_grid_cells(&self, name: &str, kind: ContainerKind, grid: &SlotGridPattern) -> Result<Vec<PatternCell>> {
        let mut out = Vec::with_capacity((grid.columns * grid.rows) as usize);
        for row in 0..grid.rows {
            for col in 0..grid.columns {
                let x = grid.x + col * grid.cell_width;
                let y = grid.y + row * grid.cell_height;
                let slots = kind
                    .section_rect_slots(grid.section, x, y, grid.cell_width, grid.cell_height)
                    .ok_or_else(|| self.pattern_outside_err(name, section_label(grid.section)))?;
                let rect = kind
                    .slot_ref_bounds(&slots)
                    .ok_or_else(|| self.target.layout_err(format!("grid pattern for `{name}` resolved to no slots")))?;
                out.push(PatternCell { rect, slots });
            }
        }
        Ok(out)
    }

    fn pattern_outside_err(&self, name: &str, section: &'static str) -> Error {
        self.target.layout_err(format!("slot pattern for `{name}` extends outside the visible `{section}` section"))
    }

    pub(super) fn slot_rect(&self, name: &str, slot: InventorySlotRef) -> Result<Rect> {
        let kind = self.target.container_kind().ok_or_else(|| self.target.slot_pattern_err(name))?;
        kind.slot_ref_rect(slot)
            .ok_or_else(|| self.target.layout_err(format!("slot pattern for `{name}` references a hidden slot")))
    }
}
