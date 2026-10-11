use std::collections::BTreeSet;

use super::SegmentMarkers;
use super::markers::{TintedSegment, tinted_segments};
use crate::ir::{LaidOutHud, Rgb};

#[derive(Clone, Debug)]
pub(super) struct HudShaderRule {
    pub(super) source_bottom: i32,
    /// Rows the HUD's glyphs are baked below the vanilla source, which the text shader subtracts again.
    pub(super) row_offset: i32,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) origin_x: f32,
    pub(super) origin_y: f32,
    pub(super) anchor_x: f32,
    pub(super) anchor_y: f32,
    pub(super) offset_x: i32,
    pub(super) offset_y: i32,
    pub(super) segments: Vec<HudShaderSegment>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct HudShaderSegment {
    pub(super) marker: Rgb,
    pub(super) display: Rgb,
    pub(super) texture: HudShaderTexture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum HudShaderTexture {
    Intensity,
    Rgba,
}

/// Builds one rule per distinct placement; HUDs sharing a placement share a rule's segments. `row_offsets` holds
/// each HUD's baked row offset in the order of `huds`.
pub(super) fn rules(huds: &[&LaidOutHud], row_offsets: &[i32], markers: &SegmentMarkers) -> Vec<HudShaderRule> {
    let mut rules: Vec<HudShaderRule> = Vec::new();

    for (hud, &row_offset) in huds.iter().zip(row_offsets) {
        let Some(shader) = hud.shader else {
            continue;
        };
        let rule = HudShaderRule {
            source_bottom: shader.source_bottom,
            row_offset,
            width: hud.width,
            height: hud.height,
            origin_x: shader.origin_x,
            origin_y: shader.origin_y,
            anchor_x: shader.anchor_x,
            anchor_y: shader.anchor_y,
            offset_x: shader.offset_x,
            offset_y: shader.offset_y,
            segments: shader_segments(hud, markers),
        };

        if let Some(existing) = rules.iter_mut().find(|existing| same_placement(existing, &rule)) {
            existing.segments.extend(rule.segments);
            existing.segments.sort();
            existing.segments.dedup();
            continue;
        }

        rules.push(rule);
    }

    rules
}

fn same_placement(a: &HudShaderRule, b: &HudShaderRule) -> bool {
    a.source_bottom == b.source_bottom
        && a.row_offset == b.row_offset
        && a.width == b.width
        && a.height == b.height
        && a.origin_x == b.origin_x
        && a.origin_y == b.origin_y
        && a.anchor_x == b.anchor_x
        && a.anchor_y == b.anchor_y
        && a.offset_x == b.offset_x
        && a.offset_y == b.offset_y
}

fn shader_segments(hud: &LaidOutHud, markers: &SegmentMarkers) -> Vec<HudShaderSegment> {
    let mut segments = BTreeSet::new();
    segments.insert(HudShaderSegment {
        marker: markers.static_marker(&hud.name),
        display: Rgb { r: 0xff, g: 0xff, b: 0xff },
        texture: HudShaderTexture::Rgba,
    });
    for slot in &hud.slots {
        segments.insert(HudShaderSegment {
            marker: markers.slot_marker(&hud.name, &slot.name),
            display: slot.color,
            texture: HudShaderTexture::Intensity,
        });
    }
    for (segment, palette) in tinted_segments(hud) {
        let texture = match segment {
            TintedSegment::Art(_) => HudShaderTexture::Rgba,
            TintedSegment::Slot(_) => HudShaderTexture::Intensity,
        };
        for (value, display) in palette {
            let marker = markers.tint_marker(&hud.name, &segment, &value);
            segments.insert(HudShaderSegment { marker, display, texture });
        }
    }
    segments.into_iter().collect()
}
