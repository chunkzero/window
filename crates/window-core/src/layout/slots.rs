use super::target::LayoutTarget;
use super::{ActiveCase, Solver};
use crate::Result;
use crate::error::Error;
use crate::geometry::{Point, Rect, Size};
use crate::inventory::{InventorySlotRef, InventorySlotSection, SlotGridPattern, SlotPattern};
use crate::ir::RepeatBindingIr;
use crate::surface::ContainerKind;

pub(super) struct ResolvedControl {
    pub(super) rect: Rect,
    pub(super) slots: Option<Vec<InventorySlotRef>>,
}

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
    /// Resolves a control's rect (explicit size, else pattern bounds) and backing
    /// slots (explicit slots, else pattern slots).
    pub(super) fn resolve_control_rect_and_slots(
        &self,
        name: &str,
        origin: Point,
        size: Option<Size>,
        slots: Option<&Vec<InventorySlotRef>>,
        pattern: Option<&SlotPattern>,
    ) -> Result<ResolvedControl> {
        let pattern_slots = pattern.map(|pattern| self.resolve_pattern_slots(name, pattern)).transpose()?;
        let rect = match (size, pattern) {
            (Some(size), _) => Rect::from_parts(origin, size),
            (None, Some(pattern)) => self.pattern_bounds(name, pattern)?,
            (None, None) => {
                return Err(self.target.layout_err(format!(
                    "control `{name}` requires `width` and `height` unless it uses `pattern` or `transform`"
                )));
            }
        };
        Ok(ResolvedControl { rect, slots: slots.cloned().or(pattern_slots) })
    }

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

    /// Resolves a one-based `cell_slot` against the enclosing repeater cell and
    /// records it as yielded, so the cell button stops filling that slot.
    pub(super) fn claim_cell_slot(&mut self, name: &str, cell_slot: u32) -> Result<InventorySlotRef> {
        let Some(repeat) = self.active_repeat.as_mut() else {
            return Err(self.target.layout_err(format!(
                "item `{name}` uses `cell_slot` outside a repeater; use `slots`, `pattern`, or \
                 `transform` for an absolute slot"
            )));
        };
        let cell_size = repeat.slots.len();
        let Some(slot) = cell_slot.checked_sub(1).and_then(|zero_based| repeat.slots.get(zero_based as usize)).copied()
        else {
            let group = repeat.group.clone();
            return Err(self.target.layout_err(format!(
                "item `{name}` sets `cell_slot` {cell_slot}, but repeater `{group}` has \
                 {cell_size} slot(s) per cell (valid range 1..={cell_size})"
            )));
        };
        if repeat.yielded.contains(&slot) {
            let group = repeat.group.clone();
            return Err(self.target.layout_err(format!(
                "item `{name}` sets `cell_slot` {cell_slot}, which another item already claims in \
                 repeater `{group}`"
            )));
        }
        repeat.yielded.push(slot);
        Ok(slot)
    }

    /// The emitted name for `name`: prefixed by the active repeater group and suffixed by its cell index, or
    /// `{name}.{case}` for a binding shared across the cases of the active switch.
    pub(super) fn scoped_name(&self, name: &str) -> String {
        match (self.active_repeat.as_ref().filter(|repeat| repeat.scoped), self.case_binding(name)) {
            (Some(repeat), _) => format!("{}_{}_{}", repeat.group, name, repeat.index),
            (None, Some(case)) => format!("{name}.{}", case.value),
            (None, None) => name.to_string(),
        }
    }

    /// The active switch case when `name` is a binding shared across that switch's cases.
    pub(super) fn case_binding(&self, name: &str) -> Option<&ActiveCase> {
        self.active_case.as_ref().filter(|case| case.shared.contains(name))
    }

    pub(super) fn repeat_binding(&self, field: &str) -> Option<RepeatBindingIr> {
        self.active_repeat.as_ref().filter(|repeat| repeat.scoped).map(|repeat| RepeatBindingIr {
            group: repeat.group.clone(),
            field: Some(field.to_string()),
            index: repeat.index,
        })
    }
}
