use super::Solver;
use super::target::LayoutTarget;
use crate::Result;
use crate::geometry::{Rect, Size};
use crate::inventory::{InventorySlotRef, SlotPattern, SlotRectClaim};
use crate::ir::{AnvilInputIr, CollectionIr, Hitbox, ItemIr, Layer, RegionIr};
use crate::model::Region;
use crate::surface::ContainerKind;

impl<T: LayoutTarget> Solver<'_, T> {
    /// Emits item `name` over `slots`, resolved from its slot source or its laid-out rect.
    pub(super) fn place_item(&mut self, name: &str, slots: Vec<InventorySlotRef>) -> Result<Size> {
        self.require_interaction(name)?;
        self.register_name(name)?;
        if slots.is_empty() {
            return Err(self.target.layout_err(format!("item `{name}` must define at least one slot")));
        }
        self.items.push(ItemIr { name: name.to_string(), slots });
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
        self.register_name(name)?;
        if slots.is_empty() {
            return Err(self.target.layout_err(format!("collection `{name}` must define at least one slot")));
        }
        if let Some(frame) = frame {
            for slot in &slots {
                let rect = self.slot_rect(name, *slot)?.slot_box();
                self.emit_frame(frame, rect, &format!("collection `{name}` frame"))?;
            }
        }
        if selected_sprite.is_some() {
            self.layers.push(Layer::Collection(name.to_string()));
        }
        self.collections.push(CollectionIr { name: name.to_string(), slots, selected_sprite, action });
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
                let name = format!("region~{}", self.next_region);
                self.next_region += 1;
                name
            }
        };
        self.require_interaction(&name)?;
        self.register_name(&name)?;
        let source = match &region.debug_name {
            Some(debug_name) => debug_name.clone(),
            None => format!("region `{name}`"),
        };
        self.check_inside_bounds(rect, &source)?;
        let hitbox = (region.tooltip.is_some() || region.item_model.is_some())
            .then(|| Hitbox { item_model: region.item_model.clone(), tooltip: region.tooltip.clone() });
        self.regions.push(RegionIr {
            action: region.name.clone(),
            name,
            rect,
            slots,
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
        self.register_name(name)?;
        self.inputs.push(AnvilInputIr {
            name: name.to_string(),
            initial: initial.to_string(),
            item_model: item_model.map(str::to_string),
        });
        Ok(Size::new(0, 0))
    }

    /// Emits a claim-only region named `name` over the slots of `pattern`; `Unowned` claims only the slots no other
    /// control owns.
    pub(super) fn place_claim(
        &mut self,
        name: &str,
        source: &str,
        pattern: &SlotPattern,
        claim: SlotRectClaim,
    ) -> Result<()> {
        self.require_slot_patterns(name)?;
        self.register_name(name)?;
        let slots = self.resolve_pattern_slots(name, pattern)?;
        if slots.is_empty() {
            return Err(self.target.layout_err(format!("{source} resolved to no slots")));
        }
        let rect = self.target.container_kind().and_then(|kind| kind.slot_ref_bounds(&slots)).unwrap_or_default();
        self.regions.push(RegionIr {
            name: name.to_string(),
            rect,
            slots: Some(slots),
            unowned: claim == SlotRectClaim::Unowned,
            action: None,
            default_action: None,
            hitbox: None,
            source: source.to_string(),
        });
        Ok(())
    }
}
