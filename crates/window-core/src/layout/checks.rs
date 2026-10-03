use std::collections::BTreeSet;

use super::Solver;
use super::target::LayoutTarget;
use crate::Result;
use crate::error::Error;
use crate::geometry::Rect;
use crate::inventory::{InventorySlotArea, InventorySlotRef};
use crate::surface::ContainerKind;
use crate::text_font::TextFont;
use crate::vanilla;

impl<T: LayoutTarget> Solver<'_, T> {
    pub(super) fn require_interaction(&self, name: &str) -> Result<()> {
        if self.target.allows_interaction() { Ok(()) } else { Err(self.target.interaction_err(name)) }
    }

    pub(super) fn require_slot_patterns(&self, name: &str) -> Result<()> {
        if self.target.allows_interaction() { Ok(()) } else { Err(self.target.slot_pattern_err(name)) }
    }

    pub(super) fn require_sprite_slots(&self, name: &str) -> Result<()> {
        if self.target.allows_sprite_slots() { Ok(()) } else { Err(self.target.sprite_slot_err(name)) }
    }

    pub(super) fn warn_if_overflow(&mut self, rect: Rect, what: &str) {
        if !self.target.visual_bounds().contains(&rect) {
            self.warnings.push(self.target.overflow_warning(rect, what));
        }
    }

    pub(super) fn warn_if_static_text_overflow(
        &mut self,
        text: &str,
        visible_width: u32,
        reserved_width: u32,
        name: &str,
    ) {
        if visible_width > reserved_width {
            self.warnings.push(format!(
                "{} `{}`: label `{name}` text `{text}` is {visible_width}px wide but reserves {reserved_width}px",
                self.target.kind(),
                self.target.name(),
            ));
        }
    }

    pub(super) fn warn_if_unsupported_static_text(&mut self, text: &str, font: Option<&TextFont>, name: &str) {
        let unsupported: Vec<String> = text
            .chars()
            .filter(|&character| {
                font.map_or_else(|| vanilla::advance(character), |font| font.advance(character)).is_none()
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|character| format!("U+{:04X}", character as u32))
            .collect();
        if !unsupported.is_empty() {
            self.warnings.push(format!(
                "{} `{}`: label `{name}` uses unsupported shifted-font glyph(s) {}; use printable ASCII, window.text.small_caps, Window UI markers, or a sprite",
                self.target.kind(),
                self.target.name(),
                unsupported.join(", "),
            ));
        }
    }

    pub(super) fn warn_if_reserved_gutter(&mut self, rect: Rect, what: &str) {
        let Some(kind) = self.target.container_kind() else {
            return;
        };
        for gutter in kind.reserved_gutters() {
            if rect.intersects(&gutter.rect) {
                self.warnings.push(format!(
                    "window `{}`: {what} rect {rect:?} overlaps reserved {} gutter {:?}; use a section-backed pattern to preserve native spacing",
                    self.target.name(),
                    gutter.name,
                    gutter.rect,
                ));
            }
        }
    }

    pub(super) fn warn_if_control_slot_bounds_mismatch(
        &mut self,
        name: &str,
        rect: Rect,
        slots: Option<&[InventorySlotRef]>,
    ) {
        let (Some(kind), Some(slots)) = (self.target.container_kind(), slots) else {
            return;
        };
        let Some(slot_bounds) = kind.slot_ref_bounds(slots) else {
            return;
        };
        if rect != slot_bounds {
            self.warnings.push(format!(
                "window `{}`: control `{name}` draws at {rect:?} but its backing slots are bounded by {slot_bounds:?}; clicks follow the backing slots, so omit x/y/width/height to use native bounds or align the artwork exactly",
                self.target.name(),
            ));
        }
    }

    pub(super) fn warn_if_sparse_anvil_control(&mut self, name: &str, rect: Rect, slots: Option<&[InventorySlotRef]>) {
        if self.target.container_kind() != Some(ContainerKind::Anvil) {
            return;
        }
        let container_slots =
            slots.unwrap_or_default().iter().filter(|slot| slot.area == InventorySlotArea::Container).count();
        if container_slots > 1 && rect.width > 16 {
            self.warnings.push(format!(
                "window `{}`: control `{name}` spans {container_slots} nonuniform anvil container slots in rect {rect:?}; prefer separate single-slot controls",
                self.target.name(),
            ));
        }
    }

    pub(super) fn check_inside_bounds(&self, rect: Rect, what: &str) -> Result<()> {
        if !self.target.bounds().contains(&rect) {
            return Err(self.target.outside_err(rect, what));
        }
        Ok(())
    }

    pub(super) fn check_text_constraints(&self, rect: Rect, name: &str, is_slot: bool) -> Result<()> {
        let kind = if is_slot { "slot" } else { "label" };
        self.check_inside_bounds(rect, &format!("{kind} `{name}`"))?;
        if rect.y < self.target.min_text_y() {
            return Err(self.target.text_limit_err(kind, name, rect.y));
        }
        Ok(())
    }

    pub(super) fn next_label_name(&mut self) -> String {
        let name = format!("label_{}", self.next_label);
        self.next_label += 1;
        name
    }

    pub(super) fn register_name(&mut self, name: &str) -> Result<()> {
        if !self.names.insert(name.to_string()) {
            return Err(Error::Validation(format!(
                "duplicate name `{name}` in {} `{}` ({})",
                self.target.kind(),
                self.target.name(),
                self.target.duplicate_namespace()
            )));
        }
        Ok(())
    }
}
