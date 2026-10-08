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
                    let file = format!("{file_base}_switch/{}/{}", path_segment(&switch.name), case.value);
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
                    regions: case.regions.clone(),
                    switches: case.switches.clone(),
                    items: case.items.clone(),
                    collections: case.collections.clone(),
                });
            }
            let entry = SwitchEntry {
                binding: switch.binding.clone(),
                states: switch.states,
                initial: switch.initial.clone(),
                source: switch.source.clone(),
                cases: out,
            };
            entries.insert(switch.name.clone(), entry);
        }
        Ok(entries)
    }
}

/// `name` as a resource path segment that no authored name or other entry name collides with: an indexed entry
/// such as `lamp[2]` becomes `lamp-2`, a handle condition such as `mode?gear~2` becomes `mode-is-gear--2`, a
/// selection click such as `mode=gear` becomes `mode-set-gear`, and a runtime action such as `window:close` becomes
/// `window-close`.
fn path_segment(name: &str) -> String {
    name.replace('[', "-")
        .replace(']', "")
        .replace('?', "-is-")
        .replace('=', "-set-")
        .replace('~', "--")
        .replace(':', "-")
}
