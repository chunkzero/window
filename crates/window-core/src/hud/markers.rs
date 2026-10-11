use std::collections::{BTreeMap, BTreeSet};

use xxhash_rust::xxh3::xxh3_64;

use crate::ir::{LaidOutHud, Rgb, tint_palette};

const MARKER_SPAN: u16 = u16::MAX;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SegmentMarkers {
    markers: BTreeMap<String, Rgb>,
}

impl SegmentMarkers {
    pub fn static_marker(&self, hud: &str) -> Rgb {
        self.markers[&static_key(hud)]
    }

    pub fn slot_marker(&self, hud: &str, slot: &str) -> Rgb {
        self.markers[&slot_key(hud, slot)]
    }

    /// The marker `segment` of `hud` is sent in while its tint is at palette entry `value`.
    pub fn tint_marker(&self, hud: &str, segment: &TintedSegment<'_>, value: &str) -> Rgb {
        self.markers[&tint_key(hud, segment, value)]
    }
}

/// A HUD segment colored by a tint.
#[derive(Clone, Copy, Debug)]
pub enum TintedSegment<'a> {
    /// The art layer with this name.
    Art(&'a str),
    /// The text slot with this name.
    Slot(&'a str),
}

/// Every tinted segment of `hud` with the palette of its tint.
pub fn tinted_segments(hud: &LaidOutHud) -> Vec<(TintedSegment<'_>, Vec<(String, Rgb)>)> {
    let art = hud.art.iter().map(|art| (TintedSegment::Art(&art.name), &art.tint));
    let slots = hud.slots.iter().map(|slot| (TintedSegment::Slot(&slot.name), &slot.tint));
    art.chain(slots)
        .filter_map(|(segment, tint)| Some((segment, tint_palette(&hud.handles, tint.as_deref()?))))
        .collect()
}

pub fn segment_markers(huds: &[&LaidOutHud]) -> SegmentMarkers {
    let mut keys = BTreeSet::new();
    for hud in huds {
        if hud.shader.is_none() {
            continue;
        }
        keys.insert(static_key(&hud.name));
        for slot in &hud.slots {
            keys.insert(slot_key(&hud.name, &slot.name));
        }
        for (segment, palette) in tinted_segments(hud) {
            for (value, _) in palette {
                keys.insert(tint_key(&hud.name, &segment, &value));
            }
        }
    }
    SegmentMarkers { markers: allocate_marker_colors(&keys) }
}

fn static_key(hud: &str) -> String {
    format!("hud/{hud}/static")
}

fn slot_key(hud: &str, slot: &str) -> String {
    format!("hud/{hud}/slot/{slot}")
}

fn tint_key(hud: &str, segment: &TintedSegment<'_>, value: &str) -> String {
    match segment {
        TintedSegment::Art(art) => format!("hud/{hud}/{art}/tint/{value}"),
        TintedSegment::Slot(slot) => format!("{}/tint/{value}", slot_key(hud, slot)),
    }
}

fn allocate_marker_colors(keys: &BTreeSet<String>) -> BTreeMap<String, Rgb> {
    let mut used = BTreeSet::new();
    let mut out = BTreeMap::new();
    for key in keys {
        let mut id = marker_start_id(key);
        while used.contains(&id) {
            id = if id == u16::MAX { 1 } else { id + 1 };
        }
        used.insert(id);
        out.insert(key.clone(), marker_color(id));
    }
    out
}

fn marker_start_id(key: &str) -> u16 {
    (xxh3_64(key.as_bytes()) % MARKER_SPAN as u64) as u16 + 1
}

pub(super) fn marker_color(id: u16) -> Rgb {
    Rgb { r: (id >> 8) as u8, g: 0, b: id as u8 }
}
