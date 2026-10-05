//! Switch cases: each case's art baked as its own net-zero static segment.

use std::collections::BTreeMap;

use crate::Result;
use crate::bake;
use crate::compose::Composite;
use crate::geometry::Point;
use crate::ir::SwitchIr;
use crate::manifest::{SwitchCaseEntry, SwitchEntry};

use super::CompileContext;
use super::glyphs::case_key;

impl CompileContext<'_> {
    /// Bakes every case of `switches` net-zero from `origin`, where `cases` holds the composites indexed
    /// `[switch][case]`. `key_prefix` and `file_base` name the owning surface's glyph keys and textures.
    pub(super) fn switch_entries(
        &mut self,
        owner: &str,
        key_prefix: &str,
        file_base: &str,
        switches: &[SwitchIr],
        cases: &[Vec<Composite>],
        origin: Point,
    ) -> Result<BTreeMap<String, SwitchEntry>> {
        let mut entries = BTreeMap::new();
        for (switch, comps) in switches.iter().zip(cases) {
            let mut out = Vec::with_capacity(switch.cases.len());
            for (case, comp) in switch.cases.iter().zip(comps) {
                let static_text = if comp.has_content {
                    let ascent = origin.y + 7 - comp.bounds.y;
                    let file = format!("{file_base}_switch/{}/{}", switch.name, case.value);
                    let key = case_key(key_prefix, &switch.name, &case.value);
                    let glyph = self.emit_static_glyph(comp, ascent, owner, &file, key)?;
                    bake::bake_static_with_advance(glyph.glyph, &comp.bounds, origin, glyph.advance)
                } else {
                    bake::bake_empty()
                };
                out.push(SwitchCaseEntry {
                    value: case.value.clone(),
                    static_text,
                    slots: case.slots.clone(),
                    sprite_slots: case.sprite_slots.clone(),
                });
            }
            entries.insert(switch.name.clone(), SwitchEntry { cases: out });
        }
        Ok(entries)
    }
}
