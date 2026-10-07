use std::collections::BTreeMap;

use super::slots::{PatternCell, ResolvedControl, cells_bounds};
use super::target::LayoutTarget;
use super::{ActiveRepeat, Solver};
use crate::Result;
use crate::geometry::{Insets, Point, Size};
use crate::inventory::{InventorySlotRef, SlotPattern, SlotRectClaim};
use crate::ir::{
    AnvilInputIr, ButtonDefault, ButtonIr, ButtonState, ButtonTooltip, CollectionIr, ItemIr, RepeatBindingIr,
    SlotRectIr,
};
use crate::model::{Element, RepeaterCells};
use crate::surface::ContainerKind;

/// The fields buttons and hotspots share.
pub(super) struct ControlSpec<'e> {
    name: &'e str,
    size: Option<Size>,
    slots: Option<&'e Vec<InventorySlotRef>>,
    pattern: Option<&'e SlotPattern>,
    tooltip: Option<&'e ButtonTooltip>,
    states: &'e BTreeMap<String, ButtonState>,
}

impl<'e> ControlSpec<'e> {
    pub(super) fn new(
        name: &'e str,
        size: Option<Size>,
        slots: Option<&'e Vec<InventorySlotRef>>,
        pattern: Option<&'e SlotPattern>,
        tooltip: &'e Option<ButtonTooltip>,
        states: &'e BTreeMap<String, ButtonState>,
    ) -> Self {
        Self { name, size, slots, pattern, tooltip: tooltip.as_ref(), states }
    }
}

impl<T: LayoutTarget> Solver<'_, T> {
    pub(super) fn place_button(
        &mut self,
        control: ControlSpec<'_>,
        frame: Option<&str>,
        padding: u32,
        default: Option<ButtonDefault>,
        children: &[Element],
        origin: Point,
    ) -> Result<Size> {
        let name = control.name;
        self.require_interaction(name)?;
        let resolved =
            self.resolve_control_rect_and_slots(name, origin, control.size, control.slots, control.pattern)?;
        if let Some(frame) = frame {
            self.emit_frame(frame, resolved.rect, &format!("button `{name}` frame `{frame}`"))?;
        }
        self.layout_button_children(name, children, resolved.rect, padding)?;
        let actual_name = self.register_control("button", name, &resolved)?;
        Ok(self.push_control(&control, actual_name, resolved, default, true))
    }

    pub(super) fn place_hotspot(&mut self, control: ControlSpec<'_>, origin: Point) -> Result<Size> {
        let name = control.name;
        self.require_interaction(name)?;
        let resolved =
            self.resolve_control_rect_and_slots(name, origin, control.size, control.slots, control.pattern)?;
        let actual_name = self.register_control("hotspot", name, &resolved)?;
        if control.tooltip.is_none() && control.states.is_empty() {
            return Err(self.target.layout_err(format!("hotspot `{name}` must define `tooltip` or `states`")));
        }
        Ok(self.push_control(&control, actual_name, resolved, None, false))
    }

    /// Registers a resolved button or hotspot, checks its bounds, and records
    /// placement warnings. Returns the scoped name.
    fn register_control(&mut self, kind: &str, name: &str, resolved: &ResolvedControl) -> Result<String> {
        let rect = resolved.rect;
        let actual_name = self.scoped_name(name);
        self.register_name(&actual_name)?;
        self.check_inside_bounds(rect, &format!("{kind} `{name}`"))?;
        self.warn_if_reserved_gutter(rect, &format!("{kind} `{name}`"));
        self.warn_if_control_slot_bounds_mismatch(name, rect, resolved.slots.as_deref());
        self.warn_if_sparse_anvil_control(name, rect, resolved.slots.as_deref());
        Ok(actual_name)
    }

    fn push_control(
        &mut self,
        control: &ControlSpec<'_>,
        actual_name: String,
        resolved: ResolvedControl,
        default: Option<ButtonDefault>,
        action: bool,
    ) -> Size {
        self.buttons.push(ButtonIr {
            name: actual_name,
            rect: resolved.rect,
            slots: resolved.slots,
            yielded_slots: Vec::new(),
            default,
            action,
            tooltip: control.tooltip.cloned(),
            states: control.states.clone(),
            repeat: self.repeat_binding(control.name),
        });
        resolved.rect.size()
    }

    pub(super) fn place_item(
        &mut self,
        name: &str,
        slots: Option<&Vec<InventorySlotRef>>,
        pattern: Option<&SlotPattern>,
        cell_slot: Option<u32>,
    ) -> Result<Size> {
        self.require_interaction(name)?;
        let actual_name = self.scoped_name(name);
        self.register_name(&actual_name)?;
        let slots = match cell_slot {
            Some(cell_slot) => vec![self.claim_cell_slot(name, cell_slot)?],
            None => self.resolve_required_slots(name, slots, pattern)?,
        };
        if slots.is_empty() {
            return Err(self.target.layout_err(format!("item `{name}` must define at least one slot")));
        }
        self.items.push(ItemIr { name: actual_name, slots, repeat: self.repeat_binding(name) });
        Ok(Size::new(0, 0))
    }

    pub(super) fn place_collection(
        &mut self,
        name: &str,
        slots: Option<&Vec<InventorySlotRef>>,
        pattern: Option<&SlotPattern>,
        frame: Option<&str>,
        selected_sprite: Option<String>,
        action: bool,
    ) -> Result<Size> {
        self.require_interaction(name)?;
        let actual_name = self.scoped_name(name);
        self.register_name(&actual_name)?;
        let slots = self.resolve_required_slots(name, slots, pattern)?;
        if slots.is_empty() {
            return Err(self.target.layout_err(format!("collection `{name}` must define at least one slot")));
        }
        self.emit_slot_frames("collection", name, frame, &slots)?;
        self.collections.push(CollectionIr {
            name: actual_name,
            slots,
            selected_sprite,
            action,
            repeat: self.repeat_binding(name),
        });
        Ok(Size::new(0, 0))
    }

    pub(super) fn place_anvil_input(&mut self, name: &str, initial: &str, item_model: Option<&str>) -> Result<Size> {
        if self.target.container_kind() != Some(ContainerKind::Anvil) {
            return Err(self.target.layout_err(format!("anvil_input `{name}` requires container `anvil`")));
        }
        if !self.inputs.is_empty() {
            return Err(self.target.layout_err("anvil surfaces support exactly one anvil_input"));
        }
        let actual_name = self.scoped_name(name);
        self.register_name(&actual_name)?;
        self.inputs.push(AnvilInputIr {
            name: actual_name,
            initial: initial.to_string(),
            item_model: item_model.map(str::to_string),
        });
        Ok(Size::new(0, 0))
    }

    pub(super) fn place_slot_rects(
        &mut self,
        name: &str,
        frame: Option<&str>,
        pattern: &SlotPattern,
        claim: SlotRectClaim,
    ) -> Result<Size> {
        self.require_slot_patterns(name)?;
        let actual_name = self.scoped_name(name);
        self.register_name(&actual_name)?;
        let slots = self.resolve_pattern_slots(name, pattern)?;
        if slots.is_empty() {
            return Err(self.target.layout_err(format!("slot_rects `{name}` resolved to no slots")));
        }
        self.emit_slot_frames("slot_rects", name, frame, &slots)?;
        self.slot_rects.push(SlotRectIr { name: actual_name, slots, claim });
        Ok(Size::new(0, 0))
    }

    /// Draws `frame`, when set, over the full 18x18 vanilla slot box of every backing slot in order.
    fn emit_slot_frames(
        &mut self,
        kind: &str,
        name: &str,
        frame: Option<&str>,
        slots: &[InventorySlotRef],
    ) -> Result<()> {
        let Some(frame) = frame else {
            return Ok(());
        };
        for slot in slots {
            let item = self.slot_rect(name, *slot)?;
            let rect = item.slot_box();
            self.emit_frame(frame, rect, &format!("{kind} `{name}` frame `{frame}`"))?;
        }
        Ok(())
    }

    pub(super) fn place_repeater(
        &mut self,
        name: &str,
        pattern: &SlotPattern,
        frame: Option<&str>,
        padding: u32,
        children: &[Element],
        cells: Option<&RepeaterCells>,
    ) -> Result<Size> {
        self.require_slot_patterns(name)?;
        self.register_name(name)?;
        let resolved = self.resolve_pattern_cells(name, pattern)?;
        if resolved.is_empty() {
            return Err(self.target.layout_err(format!("repeater `{name}` resolved to no cells")));
        }
        if let Some(cells) = cells
            && cells.children.len() != resolved.len()
        {
            return Err(self.target.layout_err(format!(
                "repeater `{name}` renders {} cells, but its pattern has {}",
                cells.children.len(),
                resolved.len()
            )));
        }
        for (index, cell) in resolved.iter().enumerate() {
            let content = match cells {
                Some(cells) => CellContent {
                    button: cells.buttons[index].clone(),
                    action: cells.action,
                    children: &cells.children[index],
                    scoped: false,
                },
                None => CellContent { button: format!("{name}_{index}"), action: true, children, scoped: true },
            };
            self.place_repeater_cell(name, index as u32, cell, frame, padding, content)?;
        }
        Ok(cells_bounds(&resolved).map_or(Size::new(0, 0), |rect| rect.size()))
    }

    fn place_repeater_cell(
        &mut self,
        group: &str,
        index: u32,
        cell: &PatternCell,
        frame: Option<&str>,
        padding: u32,
        content: CellContent<'_>,
    ) -> Result<()> {
        if let Some(frame) = frame {
            self.emit_frame(frame, cell.rect, &format!("repeater `{group}` frame `{frame}`"))?;
        }
        self.register_name(&content.button)?;
        self.check_inside_bounds(cell.rect, &format!("repeater `{group}` cell {index}"))?;
        // The cell button keeps every cell slot as a click route. It is pushed
        // before its children so document order is stable; any slot a child
        // item claims is subtracted afterwards.
        let button_index = self.buttons.len();
        let repeat = content.scoped.then(|| RepeatBindingIr { group: group.to_string(), field: None, index });
        self.buttons.push(ButtonIr {
            name: content.button,
            rect: cell.rect,
            slots: Some(cell.slots.clone()),
            yielded_slots: Vec::new(),
            default: None,
            action: content.action,
            tooltip: None,
            states: BTreeMap::new(),
            repeat,
        });
        let previous = self.active_repeat.replace(ActiveRepeat {
            group: group.to_string(),
            index,
            scoped: content.scoped,
            slots: cell.slots.clone(),
            yielded: Vec::new(),
        });
        let placed = self.layout_container_children(content.children, cell.rect, Insets::uniform(padding));
        let finished = std::mem::replace(&mut self.active_repeat, previous);
        placed?;
        if let Some(finished) = finished {
            self.buttons[button_index].yielded_slots = finished.yielded;
        }
        Ok(())
    }
}

/// What one repeater cell holds: its button and the children placed in it.
struct CellContent<'e> {
    button: String,
    action: bool,
    children: &'e [Element],
    /// Whether child names are scoped to the repeater group and cell.
    scoped: bool,
}
