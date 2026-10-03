//! Inventory-backed controls: slot ownership, button routing, and repeat groups.

use std::collections::BTreeMap;

use crate::font::shifted_suffix;
use crate::geometry::Rect;
use crate::inventory::{InventorySlotArea, InventorySlotRef, SlotRectClaim};
use crate::ir::{Align, ButtonIr, CollectionIr, LaidOutWindow, RepeatBindingIr};
use crate::manifest::{
    AnvilInputEntry, ButtonEntry, CollectionEntry, ItemEntry, RepeatGroupEntry, SlotRectEntry, SlotRefEntry,
    SpriteSlotEntry,
};
use crate::surface::{ContainerKind, Surface};
use crate::{Error, Result};

use super::CompileContext;
use super::sprites::check_sprite_fits;

#[derive(Default)]
pub(super) struct InventoryEntries {
    pub(super) buttons: BTreeMap<String, ButtonEntry>,
    pub(super) items: BTreeMap<String, ItemEntry>,
    pub(super) collections: BTreeMap<String, CollectionEntry>,
    pub(super) inputs: BTreeMap<String, AnvilInputEntry>,
    pub(super) slot_rects: BTreeMap<String, SlotRectEntry>,
}

/// Claim slots for every control in kind order: buttons, items, collections,
/// inputs, then slot rects.
pub(super) fn compile_inventory(
    ctx: &mut CompileContext<'_>,
    w: &LaidOutWindow,
    title_y: i32,
) -> Result<InventoryEntries> {
    let Surface::Container(kind) = w.surface;
    let mut claims = SlotClaims { window: &w.name, kind, owners: BTreeMap::new() };
    let mut entries = InventoryEntries::default();
    for button in &w.buttons {
        entries.buttons.insert(button.name.clone(), button_entry(ctx, &mut claims, button, title_y)?);
    }
    for item in &w.items {
        let slots = claims.claim(&item.name, &item.slots)?;
        entries.items.insert(item.name.clone(), ItemEntry { slots });
    }
    for collection in &w.collections {
        let slots = claims.claim(&collection.name, &collection.slots)?;
        let selection = collection_selection(ctx, &claims, collection, title_y)?;
        entries
            .collections
            .insert(collection.name.clone(), CollectionEntry { slots, action: collection.action, selection });
    }
    for input in &w.inputs {
        let slot = claims
            .claim(&input.name, &[InventorySlotRef::container(0)])?
            .into_iter()
            .next()
            .expect("anvil input slot is valid");
        entries.inputs.insert(
            input.name.clone(),
            AnvilInputEntry { slot, initial: input.initial.clone(), item_model: input.item_model.clone() },
        );
    }
    for slot_rect in &w.slot_rects {
        let slots = match slot_rect.claim {
            SlotRectClaim::None => Vec::new(),
            SlotRectClaim::All => claims.claim(&slot_rect.name, &slot_rect.slots)?,
            SlotRectClaim::Unowned => claims.claim_unowned(&slot_rect.name, &slot_rect.slots)?,
        };
        if !slots.is_empty() {
            entries.slot_rects.insert(slot_rect.name.clone(), SlotRectEntry { slots });
        }
    }
    Ok(entries)
}

/// Inventory slot ownership within one window.
struct SlotClaims<'a> {
    window: &'a str,
    kind: ContainerKind,
    owners: BTreeMap<InventorySlotRef, String>,
}

impl SlotClaims<'_> {
    /// Claim every slot for `owner`; a slot already owned is an error.
    fn claim(&mut self, owner: &str, slots: &[InventorySlotRef]) -> Result<Vec<SlotRefEntry>> {
        let mut out = Vec::with_capacity(slots.len());
        for slot in slots {
            self.validate(owner, slot)?;
            if let Some(existing) = self.owners.insert(*slot, owner.to_string()) {
                return Err(Error::Validation(format!(
                    "window `{}`: control `{owner}` and control `{existing}` both own {} slot {}",
                    self.window,
                    slot_area_name(slot.area),
                    slot.index
                )));
            }
            out.push((*slot).into());
        }
        Ok(out)
    }

    /// Claim only the slots no earlier control owns.
    fn claim_unowned(&mut self, owner: &str, slots: &[InventorySlotRef]) -> Result<Vec<SlotRefEntry>> {
        let mut out = Vec::new();
        for slot in slots {
            self.validate(owner, slot)?;
            if self.owners.contains_key(slot) {
                continue;
            }
            self.owners.insert(*slot, owner.to_string());
            out.push((*slot).into());
        }
        Ok(out)
    }

    fn validate(&self, owner: &str, slot: &InventorySlotRef) -> Result<()> {
        let (window, kind) = (self.window, self.kind);
        match slot.area {
            InventorySlotArea::Container if slot.index >= kind.slot_count() => Err(Error::Validation(format!(
                "window `{window}`: control `{owner}` references container slot {}, but `{}` has only {} slots",
                slot.index,
                kind.id(),
                kind.slot_count()
            ))),
            InventorySlotArea::Player if slot.index >= 36 => Err(Error::Validation(format!(
                "window `{window}`: control `{owner}` references player slot {}, but generic container screens expose only player slots 0..35",
                slot.index
            ))),
            _ => Ok(()),
        }
    }
}

fn slot_area_name(area: InventorySlotArea) -> &'static str {
    match area {
        InventorySlotArea::Container => "container",
        InventorySlotArea::Player => "player",
    }
}

fn button_entry(
    ctx: &mut CompileContext<'_>,
    claims: &mut SlotClaims<'_>,
    button: &ButtonIr,
    title_y: i32,
) -> Result<ButtonEntry> {
    let (slots, fill_slots) = button_slots(claims, button)?;
    let sprite_font = button_sprite_font(ctx, claims.window, button, title_y)?;
    Ok(ButtonEntry {
        x: button.rect.x,
        y: button.rect.y,
        width: button.rect.width,
        height: button.rect.height,
        slots,
        fill_slots,
        default: button.default,
        action: button.action,
        tooltip: button.tooltip.clone(),
        states: button.states.clone(),
        sprite_font,
    })
}

/// Click routes cover every backing slot; the button only fills (and owns) the
/// slots it has not yielded to a repeater-cell item control. Fill slots are
/// `None` when they equal the routed slots.
fn button_slots(
    claims: &mut SlotClaims<'_>,
    button: &ButtonIr,
) -> Result<(Vec<SlotRefEntry>, Option<Vec<SlotRefEntry>>)> {
    let window = claims.window;
    let slot_refs = button.slots.clone().unwrap_or_else(|| claims.kind.slot_refs_overlapping(&button.rect));
    if slot_refs.is_empty() {
        return Err(Error::Validation(format!(
            "window `{window}`: button `{}` overlaps no inventory slot",
            button.name
        )));
    }
    let fill_refs: Vec<InventorySlotRef> =
        slot_refs.iter().copied().filter(|slot| !button.yielded_slots.contains(slot)).collect();
    if fill_refs.is_empty() {
        return Err(Error::Validation(format!(
            "window `{window}`: button `{}` yielded every backing slot; leave at least one slot \
             for the button's own hitbox item",
            button.name
        )));
    }
    let fill_slots = claims.claim(&button.name, &fill_refs)?;
    let mut slots = Vec::with_capacity(slot_refs.len());
    for slot in &slot_refs {
        claims.validate(&button.name, slot)?;
        slots.push((*slot).into());
    }
    let fill_slots = if fill_slots == slots { None } else { Some(fill_slots) };
    Ok((slots, fill_slots))
}

/// The sprite font for a button with sprite states, after checking each state
/// sprite exists and fits the button.
fn button_sprite_font(
    ctx: &mut CompileContext<'_>,
    window: &str,
    button: &ButtonIr,
    title_y: i32,
) -> Result<Option<String>> {
    if !button.states.values().any(|state| state.sprite.is_some()) {
        return Ok(None);
    }
    let k = button.rect.y - title_y;
    ctx.sprite_offsets.insert(k);
    for (state_name, state) in &button.states {
        if let Some(sprite) = &state.sprite {
            let subject = format!("button `{}` state `{state_name}`", button.name);
            check_sprite_fits(ctx.runtime_sprites, window, &subject, sprite, &button.rect)?;
        }
    }
    Ok(Some(format!("{}:sprite_{}", ctx.namespace, shifted_suffix(k))))
}

/// One placement of the collection's selected sprite per cell, covering the
/// cell's 18x18 slot box.
fn collection_selection(
    ctx: &mut CompileContext<'_>,
    claims: &SlotClaims<'_>,
    collection: &CollectionIr,
    title_y: i32,
) -> Result<Vec<SpriteSlotEntry>> {
    let Some(sprite) = &collection.selected_sprite else {
        return Ok(Vec::new());
    };
    let mut cells = Vec::with_capacity(collection.slots.len());
    for slot in &collection.slots {
        let item = claims.kind.slot_ref_rect(*slot).ok_or_else(|| {
            Error::Validation(format!(
                "window `{}`: collection `{}` selects hidden {} slot {}",
                claims.window,
                collection.name,
                slot_area_name(slot.area),
                slot.index
            ))
        })?;
        let rect = Rect::new(item.x - 1, item.y - 1, item.width + 2, item.height + 2);
        let subject = format!("collection `{}` selected cell", collection.name);
        check_sprite_fits(ctx.runtime_sprites, claims.window, &subject, sprite, &rect)?;
        let k = rect.y - title_y;
        ctx.sprite_offsets.insert(k);
        cells.push(SpriteSlotEntry {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            align: Align::Left,
            font: format!("{}:sprite_{}", ctx.namespace, shifted_suffix(k)),
            sprite: Some(sprite.clone()),
        });
    }
    Ok(cells)
}

/// Group repeated controls by repeater, indexed by cell.
pub(super) fn repeat_groups(window: &LaidOutWindow) -> BTreeMap<String, RepeatGroupEntry> {
    let mut groups: BTreeMap<String, RepeatGroupEntry> = BTreeMap::new();
    for slot in &window.slots {
        if slot.text.is_some() {
            continue;
        }
        if let Some(repeat) = &slot.repeat
            && let Some(field) = &repeat.field
        {
            let group = repeat_group(&mut groups, repeat);
            set_indexed_name(&mut group.slots, field, repeat.index, slot.name.clone());
        }
    }
    for sprite_slot in &window.sprite_slots {
        if sprite_slot.sprite.is_some() {
            continue;
        }
        if let Some(repeat) = &sprite_slot.repeat
            && let Some(field) = &repeat.field
        {
            let group = repeat_group(&mut groups, repeat);
            set_indexed_name(&mut group.sprite_slots, field, repeat.index, sprite_slot.name.clone());
        }
    }
    for item in &window.items {
        if let Some(repeat) = &item.repeat
            && let Some(field) = &repeat.field
        {
            let group = repeat_group(&mut groups, repeat);
            set_indexed_name(&mut group.items, field, repeat.index, item.name.clone());
        }
    }
    for button in &window.buttons {
        if !button.action {
            continue;
        }
        if let Some(repeat) = &button.repeat
            && repeat.field.is_none()
        {
            let group = repeat_group(&mut groups, repeat);
            ensure_len(&mut group.buttons, repeat.index);
            group.buttons[repeat.index as usize] = button.name.clone();
        }
    }
    groups
}

fn repeat_group<'a>(
    groups: &'a mut BTreeMap<String, RepeatGroupEntry>,
    repeat: &RepeatBindingIr,
) -> &'a mut RepeatGroupEntry {
    let group = groups.entry(repeat.group.clone()).or_default();
    group.count = group.count.max(repeat.index + 1);
    group
}

fn set_indexed_name(fields: &mut BTreeMap<String, Vec<String>>, field: &str, index: u32, name: String) {
    let names = fields.entry(field.to_string()).or_default();
    ensure_len(names, index);
    names[index as usize] = name;
}

fn ensure_len(values: &mut Vec<String>, index: u32) {
    let len = index as usize + 1;
    if values.len() < len {
        values.resize(len, String::new());
    }
}
