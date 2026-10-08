use std::collections::BTreeMap;

use super::slots::{PatternCell, ResolvedControl, cells_bounds};
use super::target::LayoutTarget;
use super::{ActiveRepeat, Solver};
use crate::Result;
use crate::geometry::{Insets, Point, Rect, Size};
use crate::inventory::{InventorySlotRef, SlotPattern, SlotRectClaim};
use crate::ir::{
    AnvilInputIr, CollectionIr, Draw, Hitbox, ItemIr, Layer, RegionIr, RepeatBindingIr, SpriteSlotIr, SwitchIr, Tooltip,
};
use crate::model::{ControlState, Element, Region, RepeaterCells};
use crate::surface::ContainerKind;

/// Where a button or hotspot sits: its authored kind, name, and slot placement.
pub(super) struct ControlSpec<'e> {
    /// The authored widget kind, such as `button`, `tab`, or `hotspot`.
    pub(super) kind: &'e str,
    pub(super) name: &'e str,
    pub(super) size: Option<Size>,
    pub(super) slots: Option<&'e Vec<InventorySlotRef>>,
    pub(super) pattern: Option<&'e SlotPattern>,
}

/// What a button or hotspot shows and does: its hitbox tooltip, named states, and default runtime action.
pub(super) struct ControlBehavior<'e> {
    pub(super) tooltip: Option<&'e Tooltip>,
    pub(super) states: &'e BTreeMap<String, ControlState>,
    pub(super) default_action: Option<&'e str>,
}

/// A control's art outside its content: its frame, and whether its states draw their own frames.
struct ControlArt<'e> {
    frame: Option<&'e str>,
    framed_states: bool,
    /// Static draws of the control's content, copied into each state when states draw frames.
    content: Vec<Draw>,
    /// Index in the surface layers where the control's state layers go, before its content.
    layer: usize,
}

/// The state case a control shows while its state switch is unbound, unless it declares one.
const DEFAULT_STATE: &str = "default";

impl<T: LayoutTarget> Solver<'_, T> {
    pub(super) fn place_button(
        &mut self,
        control: ControlSpec<'_>,
        behavior: ControlBehavior<'_>,
        frame: Option<&str>,
        padding: u32,
        children: &[Element],
        origin: Point,
    ) -> Result<Size> {
        let name = control.name;
        self.require_interaction(name)?;
        let resolved =
            self.resolve_control_rect_and_slots(name, origin, control.size, control.slots, control.pattern)?;
        let framed_states = behavior.states.values().any(|state| state.frame.is_some());
        if let Some(frame) = frame.filter(|_| !framed_states) {
            self.emit_frame(frame, resolved.rect, &format!("{} `{name}` frame `{frame}`", control.kind))?;
        }
        let layer = self.layers.len();
        let draws = self.draws.len();
        self.layout_button_children(name, children, resolved.rect, padding)?;
        let content = if framed_states { self.draws.split_off(draws) } else { Vec::new() };
        let actual_name = self.register_control(&control, &resolved)?;
        let action = Some(actual_name.clone());
        let art = ControlArt { frame, framed_states, content, layer };
        let size = resolved.rect.size();
        self.push_control(&control, &behavior, actual_name, resolved, action, art)?;
        Ok(size)
    }

    pub(super) fn place_hotspot(
        &mut self,
        control: ControlSpec<'_>,
        behavior: ControlBehavior<'_>,
        origin: Point,
    ) -> Result<Size> {
        let name = control.name;
        self.require_interaction(name)?;
        let resolved =
            self.resolve_control_rect_and_slots(name, origin, control.size, control.slots, control.pattern)?;
        let actual_name = self.register_control(&control, &resolved)?;
        if behavior.tooltip.is_none() && behavior.states.is_empty() {
            return Err(self.target.layout_err(format!("hotspot `{name}` must define `tooltip` or `states`")));
        }
        let art = ControlArt { frame: None, framed_states: false, content: Vec::new(), layer: self.layers.len() };
        let size = resolved.rect.size();
        self.push_control(&control, &behavior, actual_name, resolved, None, art)?;
        Ok(size)
    }

    /// Registers a resolved button or hotspot, checks its bounds, and records
    /// placement warnings. Returns the scoped name.
    fn register_control(&mut self, control: &ControlSpec<'_>, resolved: &ResolvedControl) -> Result<String> {
        let (kind, name, rect) = (control.kind, control.name, resolved.rect);
        let actual_name = self.scoped_name(name);
        self.register_name(&actual_name)?;
        self.check_inside_bounds(rect, &format!("{kind} `{name}`"))?;
        self.warn_if_reserved_gutter(rect, &format!("{kind} `{name}`"));
        self.warn_if_control_slot_bounds_mismatch(name, rect, resolved.slots.as_deref());
        self.warn_if_sparse_anvil_control(name, rect, resolved.slots.as_deref());
        Ok(actual_name)
    }

    /// Emits a control's region, or with states a state switch whose cases each hold the state's region, frame,
    /// and sprite.
    fn push_control(
        &mut self,
        control: &ControlSpec<'_>,
        behavior: &ControlBehavior<'_>,
        actual_name: String,
        resolved: ResolvedControl,
        action: Option<String>,
        art: ControlArt<'_>,
    ) -> Result<()> {
        let source = format!("{} `{actual_name}`", control.kind);
        let region = |name: String, hitbox: Option<Hitbox>, repeat: Option<RepeatBindingIr>| RegionIr {
            name,
            rect: resolved.rect,
            slots: resolved.slots.clone(),
            yielded_slots: Vec::new(),
            unowned: false,
            action: action.clone(),
            default_action: behavior.default_action.map(str::to_string),
            hitbox,
            repeat,
            source: source.clone(),
        };
        let base_hitbox = behavior.tooltip.map(|tooltip| Hitbox { item_model: None, tooltip: Some(tooltip.clone()) });
        let repeat = self.repeat_binding(control.name);
        if behavior.states.is_empty() {
            self.regions.push(region(actual_name, base_hitbox, repeat));
            return Ok(());
        }

        let mut states: BTreeMap<&str, Option<&ControlState>> =
            behavior.states.iter().map(|(value, state)| (value.as_str(), Some(state))).collect();
        states.entry(DEFAULT_STATE).or_insert(None);
        let mut cases = Vec::with_capacity(states.len());
        let mut layers = vec![Layer::Switch(actual_name.clone())];
        for (value, state) in states {
            let key = format!("{actual_name}.{value}");
            let hitbox = match state {
                Some(state) => Some(Hitbox {
                    item_model: state.item_model.clone(),
                    tooltip: state.tooltip.clone().or_else(|| behavior.tooltip.cloned()),
                }),
                None => base_hitbox.clone(),
            };
            let frame = state.and_then(|state| state.frame.as_deref()).or(art.frame);
            let sprite = state.and_then(|state| state.sprite.as_deref());
            let case = self.emit_case(value, |s| {
                if let (true, Some(frame)) = (art.framed_states, frame) {
                    let subject = format!("{source} state `{value}` frame `{frame}`");
                    let draw = s.frame_draw(frame, resolved.rect, &subject)?;
                    s.draws.push(draw);
                }
                if art.framed_states {
                    s.draws.extend(art.content.iter().cloned());
                }
                if let Some(sprite) = sprite {
                    s.push_state_sprite(&key, sprite, resolved.rect, &format!("{source} state `{value}`"))?;
                    layers.push(Layer::SpriteSlot(key.clone()));
                }
                let mut region = region(key.clone(), hitbox, repeat.clone());
                // A runtime action has no handler to wrap, so its disabled state routes no clicks.
                if value == "disabled" && action.as_deref().is_some_and(|id| id.starts_with("window:")) {
                    region.action = None;
                    region.default_action = None;
                }
                s.regions.push(region);
                Ok(())
            })?;
            cases.push(case);
        }
        self.layers.splice(art.layer..art.layer, layers);
        self.switches.push(SwitchIr {
            name: actual_name,
            binding: None,
            states: true,
            initial: Some(DEFAULT_STATE.to_string()),
            source: Some(source),
            cases,
        });
        Ok(())
    }

    /// A state's sprite: a fixed sprite slot at the control's top-left corner.
    fn push_state_sprite(&mut self, key: &str, sprite: &str, rect: Rect, subject: &str) -> Result<()> {
        self.require_sprite_slots(key)?;
        let size = self.sprite_size(sprite, self.sprite_def(sprite)?)?;
        if size.width > rect.width || size.height > rect.height {
            return Err(self.target.layout_err(format!(
                "{subject} sprite `{sprite}` does not fit its {}x{} rect ({}x{})",
                rect.width, rect.height, size.width, size.height
            )));
        }
        self.sprite_slots.push(SpriteSlotIr {
            name: key.to_string(),
            rect,
            align: crate::ir::Align::Left,
            sprite: Some(sprite.to_string()),
            repeat: None,
            binding: None,
            source: None,
        });
        Ok(())
    }

    /// Emits item `name` over `slots`, resolved from its slot source or its laid-out rect.
    pub(super) fn place_item(&mut self, name: &str, slots: Vec<InventorySlotRef>) -> Result<Size> {
        self.require_interaction(name)?;
        let actual_name = self.scoped_name(name);
        self.register_name(&actual_name)?;
        if slots.is_empty() {
            return Err(self.target.layout_err(format!("item `{name}` must define at least one slot")));
        }
        self.items.push(ItemIr { name: actual_name, slots, repeat: self.repeat_binding(name) });
        Ok(Size::new(0, 0))
    }

    /// Emits collection `name` with one cell per slot of `slots`, resolved from its slot source or its laid-out
    /// rect.
    pub(super) fn place_collection(
        &mut self,
        name: &str,
        slots: Vec<InventorySlotRef>,
        frame: Option<&str>,
        selected_sprite: Option<String>,
        action: bool,
    ) -> Result<Size> {
        self.require_interaction(name)?;
        let actual_name = self.scoped_name(name);
        self.register_name(&actual_name)?;
        if slots.is_empty() {
            return Err(self.target.layout_err(format!("collection `{name}` must define at least one slot")));
        }
        self.emit_slot_frames("collection", name, frame, &slots)?;
        if selected_sprite.is_some() {
            self.layers.push(Layer::Collection(actual_name.clone()));
        }
        self.collections.push(CollectionIr {
            name: actual_name,
            slots,
            selected_sprite,
            action,
            repeat: self.repeat_binding(name),
        });
        Ok(Size::new(0, 0))
    }

    /// Emits `region` over `rect`. Its slots are `slots`, or else the inventory slots `rect` overlaps.
    pub(super) fn place_region(
        &mut self,
        region: &Region,
        rect: Rect,
        slots: Option<Vec<InventorySlotRef>>,
    ) -> Result<()> {
        let name = match &region.name {
            Some(name) => name.clone(),
            None => {
                let name = format!("region_{}", self.next_region);
                self.next_region += 1;
                name
            }
        };
        self.require_interaction(&name)?;
        let actual_name = self.scoped_name(&name);
        self.register_name(&actual_name)?;
        let source = match &region.debug_name {
            Some(debug_name) => debug_name.clone(),
            None => format!("region `{actual_name}`"),
        };
        self.check_inside_bounds(rect, &source)?;
        let hitbox = (region.tooltip.is_some() || region.item_model.is_some())
            .then(|| Hitbox { item_model: region.item_model.clone(), tooltip: region.tooltip.clone() });
        self.regions.push(RegionIr {
            action: region.name.is_some().then(|| actual_name.clone()),
            repeat: self.repeat_binding(&name),
            name: actual_name,
            rect,
            slots,
            yielded_slots: Vec::new(),
            unowned: false,
            default_action: region.default_action.clone(),
            hitbox,
            source,
        });
        Ok(())
    }

    /// The inventory slots `rect` overlaps, for a slot-bound element laid out by a box.
    pub(super) fn slots_under(&self, name: &str, rect: Rect) -> Result<Vec<InventorySlotRef>> {
        let kind = self.target.container_kind().ok_or_else(|| self.target.interaction_err(name))?;
        Ok(kind.slot_refs_overlapping(&rect))
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

    /// Draws slot frames and, unless `claim` is `None`, emits a claim-only region over the slots.
    pub(super) fn place_slot_rects(
        &mut self,
        name: &str,
        source: &str,
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
        if claim != SlotRectClaim::None {
            let rect = self.target.container_kind().and_then(|kind| kind.slot_ref_bounds(&slots)).unwrap_or_default();
            self.regions.push(RegionIr {
                name: actual_name,
                rect,
                slots: Some(slots),
                yielded_slots: Vec::new(),
                unowned: claim == SlotRectClaim::Unowned,
                action: None,
                default_action: None,
                hitbox: None,
                repeat: None,
                source: source.to_string(),
            });
        }
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
        // The cell region keeps every cell slot as a click route. It is pushed
        // before its children so document order is stable; any slot a child
        // item claims is subtracted afterwards.
        let region_index = self.regions.len();
        let repeat = content.scoped.then(|| RepeatBindingIr { group: group.to_string(), field: None, index });
        self.regions.push(RegionIr {
            action: content.action.then(|| content.button.clone()),
            name: content.button,
            rect: cell.rect,
            slots: Some(cell.slots.clone()),
            yielded_slots: Vec::new(),
            unowned: false,
            default_action: None,
            hitbox: None,
            repeat,
            source: format!("repeater `{group}` cell {index}"),
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
            self.regions[region_index].yielded_slots = finished.yielded;
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
