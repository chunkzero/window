use std::collections::{BTreeMap, BTreeSet};

use xxhash_rust::xxh3::xxh3_64;

use crate::ir::{LaidOutHud, Rgb};

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
    }
    SegmentMarkers { markers: allocate_marker_colors(&keys) }
}

fn static_key(hud: &str) -> String {
    format!("hud/{hud}/static")
}

fn slot_key(hud: &str, slot: &str) -> String {
    format!("hud/{hud}/slot/{slot}")
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
