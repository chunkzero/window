//! Inventory-backed controls: slot ownership, button routing, and repeat groups.

use std::collections::BTreeMap;

use crate::inventory::{InventorySlotArea, InventorySlotRef};
use crate::ir::{Align, CollectionIr, LaidOutWindow, RegionIr, RepeatBindingIr};
use crate::manifest::{
    AnvilInputEntry, CasePath, CollectionEntry, ItemEntry, RegionEntry, RepeatGroupEntry, SlotRefEntry,
    SpriteSlotEntry, SwitchEntry, case_paths, exclusive_cases,
};
use crate::surface::{ContainerKind, Surface};
use crate::{Error, Result};

use super::CompileContext;
use super::sprites::check_sprite_fits;
use super::window::sprite_slot_entry;

#[derive(Default)]
pub(super) struct InventoryEntries {
    pub(super) regions: BTreeMap<String, RegionEntry>,
    pub(super) items: BTreeMap<String, ItemEntry>,
    pub(super) collections: BTreeMap<String, CollectionEntry>,
    pub(super) inputs: BTreeMap<String, AnvilInputEntry>,
}

/// Claim slots for every control: regions, items, collections, inputs, then regions claiming only unowned slots.
/// Controls in mutually exclusive switch cases may claim the same slots.
pub(super) fn compile_inventory(
    ctx: &mut CompileContext<'_>,
    w: &LaidOutWindow,
    switches: &BTreeMap<String, SwitchEntry>,
    title_y: i32,
) -> Result<InventoryEntries> {
    let Surface::Container(kind) = w.surface;
    let paths = case_paths(switches);
    let mut claims = SlotClaims { window: &w.name, kind, owners: BTreeMap::new() };
    let mut entries = InventoryEntries::default();
    let input_slot = (!w.inputs.is_empty()).then(|| InventorySlotRef::container(0));
    for region in w.regions.iter().filter(|region| !region.unowned) {
        let path = paths.regions.get(&region.name).cloned().unwrap_or_default();
        entries.regions.insert(region.name.clone(), region_entry(&mut claims, region, path, input_slot)?);
    }
    for item in &w.items {
        let path = paths.items.get(&item.name).map(Vec::as_slice).unwrap_or_default();
        let slots = claims.claim(&format!("item `{}`", item.name), path, &item.slots)?;
        entries.items.insert(item.name.clone(), ItemEntry { slots });
    }
    for collection in &w.collections {
        let path = paths.collections.get(&collection.name).map(Vec::as_slice).unwrap_or_default();
        let slots = claims.claim(&format!("collection `{}`", collection.name), path, &collection.slots)?;
        let selection = collection_selection(ctx, &claims, collection, title_y)?;
        entries
            .collections
            .insert(collection.name.clone(), CollectionEntry { slots, action: collection.action, selection });
    }
    for input in &w.inputs {
        let slot = claims
            .claim(&format!("anvil input `{}`", input.name), &[], &[InventorySlotRef::container(0)])?
            .into_iter()
            .next()
            .expect("anvil input slot is valid");
        entries.inputs.insert(
            input.name.clone(),
            AnvilInputEntry { slot, initial: input.initial.clone(), item_model: input.item_model.clone() },
        );
    }
    for region in w.regions.iter().filter(|region| region.unowned) {
        let path = paths.regions.get(&region.name).map(Vec::as_slice).unwrap_or_default();
        let slots = claims.claim_unowned(&region.source, path, region.slots.as_deref().unwrap_or_default())?;
        if !slots.is_empty() {
            entries.regions.insert(region.name.clone(), RegionEntry { slots, ..base_entry(region) });
        }
    }
    Ok(entries)
}

/// Inventory slot ownership within one window.
struct SlotClaims<'a> {
    window: &'a str,
    kind: ContainerKind,
    /// Each owned slot's owners, with the case path each claimed it in.
    owners: BTreeMap<InventorySlotRef, Vec<(String, CasePath)>>,
}

impl SlotClaims<'_> {
    /// Claim every slot for `owner` in the switch cases `path`; a slot owned outside a mutually exclusive case is
    /// an error.
    fn claim(
        &mut self,
        owner: &str,
        path: &[(String, String)],
        slots: &[InventorySlotRef],
    ) -> Result<Vec<SlotRefEntry>> {
        let mut out = Vec::with_capacity(slots.len());
        for slot in slots {
            self.validate(owner, slot)?;
            let owners = self.owners.entry(*slot).or_default();
            if let Some((existing, _)) = owners.iter().find(|(_, other)| !exclusive_cases(path, other)) {
                return Err(Error::Validation(format!(
                    "window `{}`: {owner} and {existing} both own {} slot {}",
                    self.window,
                    slot_area_name(slot.area),
                    slot.index
                )));
            }
            owners.push((owner.to_string(), path.to_vec()));
            out.push((*slot).into());
        }
        Ok(out)
    }

    /// Claim only the slots no earlier control owns in a case that can be active with `path`.
    fn claim_unowned(
        &mut self,
        owner: &str,
        path: &[(String, String)],
        slots: &[InventorySlotRef],
    ) -> Result<Vec<SlotRefEntry>> {
        let mut out = Vec::new();
        for slot in slots {
            self.validate(owner, slot)?;
            let owners = self.owners.entry(*slot).or_default();
            if owners.iter().any(|(_, other)| !exclusive_cases(path, other)) {
                continue;
            }
            owners.push((owner.to_string(), path.to_vec()));
            out.push((*slot).into());
        }
        Ok(out)
    }

    fn validate(&self, owner: &str, slot: &InventorySlotRef) -> Result<()> {
        let (window, kind) = (self.window, self.kind);
        match slot.area {
            InventorySlotArea::Container if slot.index >= kind.slot_count() => Err(Error::Validation(format!(
                "window `{window}`: {owner} references container slot {}, but `{}` has only {} slots",
                slot.index,
                kind.id(),
                kind.slot_count()
            ))),
            InventorySlotArea::Player if slot.index >= 36 => Err(Error::Validation(format!(
                "window `{window}`: {owner} references player slot {}, but generic container screens expose only player slots 0..35",
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

/// A region's entry before its slots are claimed.
fn base_entry(region: &RegionIr) -> RegionEntry {
    RegionEntry {
        x: region.rect.x,
        y: region.rect.y,
        width: region.rect.width,
        height: region.rect.height,
        slots: Vec::new(),
        fill_slots: None,
        action: region.action.clone(),
        default_action: region.default_action.clone(),
        hitbox: region.hitbox.clone(),
        source: Some(region.source.clone()),
    }
}

/// Click routes cover every backing slot; the region only fills (and owns) the slots it has not yielded to a
/// repeater-cell item control or the anvil input, which keeps its seed item in `input_slot`. Fill slots are `None`
/// when they equal the routed slots.
fn region_entry(
    claims: &mut SlotClaims<'_>,
    region: &RegionIr,
    path: CasePath,
    input_slot: Option<InventorySlotRef>,
) -> Result<RegionEntry> {
    let (window, owner) = (claims.window, &region.source);
    let slot_refs = region.slots.clone().unwrap_or_else(|| claims.kind.slot_refs_overlapping(&region.rect));
    if slot_refs.is_empty() {
        return Err(Error::Validation(format!("window `{window}`: {owner} overlaps no inventory slot")));
    }
    let routes_input = input_slot.is_some_and(|input| slot_refs.contains(&input));
    let fill_refs: Vec<InventorySlotRef> = slot_refs
        .iter()
        .copied()
        .filter(|slot| !region.yielded_slots.contains(slot) && Some(*slot) != input_slot)
        .collect();
    if fill_refs.is_empty() && !routes_input {
        return Err(Error::Validation(format!(
            "window `{window}`: {owner} yielded every backing slot; leave at least one slot for its own hitbox item"
        )));
    }
    let fill_slots = claims.claim(owner, &path, &fill_refs)?;
    let mut slots = Vec::with_capacity(slot_refs.len());
    for slot in &slot_refs {
        claims.validate(owner, slot)?;
        slots.push((*slot).into());
    }
    let fill_slots = if fill_slots == slots { None } else { Some(fill_slots) };
    Ok(RegionEntry { slots, fill_slots, ..base_entry(region) })
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
    let mut boxes = Vec::with_capacity(collection.slots.len());
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
        boxes.push(item.slot_box());
    }
    if let Some(first) = boxes.first() {
        let subject = format!("collection `{}` selected cell", collection.name);
        check_sprite_fits(ctx.runtime_sprites, claims.window, &subject, sprite, first)?;
    }
    Ok(boxes
        .iter()
        .map(|rect| {
            let font = ctx.sprite_font(rect.y - title_y);
            sprite_slot_entry(font, rect, Align::Left, Some(sprite.clone()))
        })
        .collect())
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
    for region in &window.regions {
        let Some(action) = &region.action else {
            continue;
        };
        if let Some(repeat) = &region.repeat
            && repeat.field.is_none()
        {
            let group = repeat_group(&mut groups, repeat);
            ensure_len(&mut group.actions, repeat.index);
            group.actions[repeat.index as usize] = action.clone();
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
