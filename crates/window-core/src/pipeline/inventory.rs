//! Inventory-backed controls: slot contents and click routing.

use std::collections::BTreeMap;

use crate::inventory::{InventorySlotArea, InventorySlotRef};
use crate::ir::{Align, CollectionIr, LaidOutWindow, RegionIr};
use crate::manifest::{
    AnvilInputEntry, CasePath, CollectionEntry, ItemEntry, RegionEntry, SlotRefEntry, SpriteSlotEntry, SwitchEntry,
    case_paths, exclusive_cases,
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

/// Claim slots for every control. Items, collections, and inputs own their slots' contents; regions route clicks on
/// every slot they cover and fill their hitbox item only into slots no content owner holds. Regions claiming only
/// unowned slots come last. Controls in mutually exclusive switch cases may claim the same slots.
pub(super) fn compile_inventory(
    ctx: &mut CompileContext<'_>,
    w: &LaidOutWindow,
    switches: &BTreeMap<String, SwitchEntry>,
    title_y: i32,
) -> Result<InventoryEntries> {
    let Surface::Container(kind) = w.surface;
    let paths = case_paths(switches);
    let mut claims = SlotClaims { window: &w.name, kind, contents: BTreeMap::new(), routes: BTreeMap::new() };
    let mut entries = InventoryEntries::default();
    for item in &w.items {
        let path = paths.items.get(&item.name).map(Vec::as_slice).unwrap_or_default();
        let slots = claims.own(&format!("item `{}`", item.name), path, &item.slots)?;
        entries.items.insert(item.name.clone(), ItemEntry { slots });
    }
    for collection in &w.collections {
        let path = paths.collections.get(&collection.name).map(Vec::as_slice).unwrap_or_default();
        let slots = claims.own(&format!("collection `{}`", collection.name), path, &collection.slots)?;
        let selection = collection_selection(ctx, &claims, collection, title_y)?;
        entries
            .collections
            .insert(collection.name.clone(), CollectionEntry { slots, action: collection.action, selection });
    }
    for input in &w.inputs {
        let slot = claims
            .own(&format!("anvil input `{}`", input.name), &[], &[InventorySlotRef::container(0)])?
            .into_iter()
            .next()
            .expect("anvil input slot is valid");
        entries.inputs.insert(
            input.name.clone(),
            AnvilInputEntry { slot, initial: input.initial.clone(), item_model: input.item_model.clone() },
        );
    }
    for region in w.regions.iter().filter(|region| !region.unowned) {
        let path = paths.regions.get(&region.name).map(Vec::as_slice).unwrap_or_default();
        entries.regions.insert(region.name.clone(), region_entry(&mut claims, region, path)?);
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

/// Each slot's claimants with the case path each claimed it in.
type Claimants = BTreeMap<InventorySlotRef, Vec<(String, CasePath)>>;

/// Inventory slot claims within one window: who owns each slot's contents and which region routes its clicks.
struct SlotClaims<'a> {
    window: &'a str,
    kind: ContainerKind,
    contents: Claimants,
    routes: Claimants,
}

impl SlotClaims<'_> {
    /// Own the contents of every slot for `owner` in the switch cases `path`.
    fn own(&mut self, owner: &str, path: &[(String, String)], slots: &[InventorySlotRef]) -> Result<Vec<SlotRefEntry>> {
        slots.iter().map(|slot| self.claim(Claim::Contents, owner, path, *slot)).collect()
    }

    /// Claim `slot` for `owner`; a slot claimed outside a mutually exclusive case is an error.
    fn claim(
        &mut self,
        claim: Claim,
        owner: &str,
        path: &[(String, String)],
        slot: InventorySlotRef,
    ) -> Result<SlotRefEntry> {
        self.validate(owner, &slot)?;
        let (window, verb) = (self.window, if claim == Claim::Contents { "own" } else { "route" });
        let claimants = self.claimants(claim).entry(slot).or_default();
        if let Some((existing, _)) = claimants.iter().find(|(_, other)| !exclusive_cases(path, other)) {
            return Err(Error::Validation(format!(
                "window `{window}`: {owner} and {existing} both {verb} {} slot {}",
                slot_area_name(slot.area),
                slot.index
            )));
        }
        claimants.push((owner.to_string(), path.to_vec()));
        Ok(slot.into())
    }

    fn claimants(&mut self, claim: Claim) -> &mut Claimants {
        match claim {
            Claim::Contents => &mut self.contents,
            Claim::Route => &mut self.routes,
        }
    }

    /// Whether a content owner holds `slot` in a case that can be active with `path`.
    fn owned(&self, path: &[(String, String)], slot: &InventorySlotRef) -> bool {
        self.contents.get(slot).is_some_and(|owners| owners.iter().any(|(_, other)| !exclusive_cases(path, other)))
    }

    /// Whether a content owner holds `slot` in every state where a control at `path` is active.
    fn owned_always(&self, path: &[(String, String)], slot: &InventorySlotRef) -> bool {
        self.contents.get(slot).is_some_and(|owners| owners.iter().any(|(_, other)| path.starts_with(other)))
    }

    /// Route and fill only the slots no earlier control routes or owns in a case that can be active with `path`.
    fn claim_unowned(
        &mut self,
        owner: &str,
        path: &[(String, String)],
        slots: &[InventorySlotRef],
    ) -> Result<Vec<SlotRefEntry>> {
        let mut out = Vec::new();
        for slot in slots {
            self.validate(owner, slot)?;
            let routed =
                self.routes.get(slot).is_some_and(|r| r.iter().any(|(_, other)| !exclusive_cases(path, other)));
            if routed || self.owned(path, slot) {
                continue;
            }
            self.claim(Claim::Route, owner, path, *slot)?;
            out.push(self.claim(Claim::Contents, owner, path, *slot)?);
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Claim {
    Contents,
    Route,
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

/// Click routes cover every backing slot; the region fills its hitbox item into the slots no item, collection, or
/// input owns whenever the region is active. Slots a conditional owner holds stay fill slots: the owner's contents
/// take precedence while its case is active. Fill slots are `None` when they equal the routed slots.
fn region_entry(claims: &mut SlotClaims<'_>, region: &RegionIr, path: &[(String, String)]) -> Result<RegionEntry> {
    let (window, owner) = (claims.window, &region.source);
    let slot_refs = region.slots.clone().unwrap_or_else(|| claims.kind.slot_refs_overlapping(&region.rect));
    if slot_refs.is_empty() {
        return Err(Error::Validation(format!("window `{window}`: {owner} overlaps no inventory slot")));
    }
    let mut slots = Vec::with_capacity(slot_refs.len());
    let mut fill_slots = Vec::with_capacity(slot_refs.len());
    for slot in slot_refs {
        slots.push(claims.claim(Claim::Route, owner, path, slot)?);
        if claims.owned_always(path, &slot) {
            continue;
        }
        if claims.owned(path, &slot) {
            fill_slots.push(slot.into());
        } else {
            fill_slots.push(claims.claim(Claim::Contents, owner, path, slot)?);
        }
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
