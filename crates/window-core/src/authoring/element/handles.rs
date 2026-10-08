use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::ElementDto;
use crate::authoring::parse::validate_name;
use crate::ir::{Handle, HandleKind, HandleRole, HandleUse, IndexedBinding};
use crate::{Error, Result};

/// The runtime action `builtin("window:close")`.
const CLOSE: &str = "window:close";

/// A typed handle as an element references it: the declaration plus how this element reads it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::authoring) struct HandleRefDto {
    kind: String,
    id: String,
    #[serde(default)]
    values: Vec<String>,
    #[serde(default)]
    shape: Vec<u32>,
    initial: Option<serde_json::Value>,
    #[serde(default)]
    selectable: bool,
    #[serde(default)]
    only: Vec<String>,
    #[serde(default)]
    at: Vec<u32>,
    is: Option<String>,
    set: Option<String>,
}

/// Collects the handles `children` use, naming each using element after the handle it reads.
///
/// An element reading handle `id` is named `id`, followed by `[i]` per index, `=value` for a click that sets a
/// selection, or `?value` for a condition; later uses of the same name append `~2`, `~3`, and so on.
pub(in crate::authoring) fn collect_handles(
    children: &mut [ElementDto],
    owner: &str,
    hud: bool,
) -> Result<BTreeMap<String, Handle>> {
    let mut collector = Collector { owner, hud, handles: BTreeMap::new(), names: BTreeSet::new() };
    for child in children {
        collector.visit(child, &Scope::default())?;
    }
    Ok(collector.handles)
}

/// Whether an element sits in a template repeater, or in a cell of a repeater with `cells`.
#[derive(Clone, Default)]
struct Scope {
    template: Option<String>,
    cells: Option<u32>,
}

struct Collector<'a> {
    owner: &'a str,
    hud: bool,
    handles: BTreeMap<String, Handle>,
    names: BTreeSet<String>,
}

impl Collector<'_> {
    fn visit(&mut self, dto: &mut ElementDto, scope: &Scope) -> Result<()> {
        let refs = [&dto.handle, &dto.on_click, &dto.enabled, &dto.state];
        if refs.iter().any(|r| r.is_some()) {
            if let Some(repeater) = &scope.template {
                return Err(self.err(format!(
                    "handles cannot be used inside repeater `{repeater}`; render its cells with a function"
                )));
            }
            if dto.index.is_some() {
                return Err(self.err(format!("{} element sets both a handle and `index`; use `.at()`", dto.kind)));
            }
        }
        let role = match dto.kind.as_str() {
            "slot" => Some(HandleRole::Slot),
            "sprite_slot" => Some(HandleRole::SpriteSlot),
            "item" => Some(HandleRole::Item),
            "collection" => Some(HandleRole::Collection),
            "anvil_input" => Some(HandleRole::Input),
            "switch" => Some(HandleRole::Switch),
            _ => None,
        };
        if let (Some(role), Some(handle)) = (role, dto.handle.clone()) {
            self.primary(dto, role, &handle, scope)?;
        }
        if dto.kind == "button" || dto.kind == "region" {
            self.button(dto, scope)?;
        }
        let mut inner = scope.clone();
        if dto.kind == "repeater" {
            match dto.cells.is_some() {
                true => inner.cells = Some(self.repeater(dto)?),
                false => inner.template = inner.template.or_else(|| dto.name.clone()),
            }
        }
        for child in &mut dto.children {
            self.visit(child, &inner)?;
        }
        if let Some(cells) = &mut dto.cells {
            for cell in cells {
                for child in cell {
                    self.visit(child, &inner)?;
                }
            }
        }
        Ok(())
    }

    /// Names an element that binds one handle: a text, sprite, or item slot, a collection, an input, or a switch.
    fn primary(&mut self, dto: &mut ElementDto, role: HandleRole, handle: &HandleRefDto, scope: &Scope) -> Result<()> {
        if dto.name.is_some() {
            return Err(self.err(format!("{} element sets both `name` and a handle", dto.kind)));
        }
        if role == HandleRole::SpriteSlot && dto.sprite.is_some() {
            return Err(
                self.err(format!("sprite slot bound to `{}` sets a fixed `sprite`, which needs no binding", handle.id))
            );
        }
        let kind = self.declare(handle, role, scope)?;
        if role == HandleRole::Collection && dto.selected_sprite.is_some() && !handle.selectable {
            return Err(self
                .err(format!("collection `{}` draws a selected cell, but its handle is not `selectable`", handle.id)));
        }
        if role == HandleRole::Switch {
            let mut cases: Vec<&str> = dto.children.iter().filter_map(|case| case.value.as_deref()).collect();
            cases.sort_unstable();
            let mut expected: Vec<&str> = match (kind, &handle.is) {
                (HandleKind::Value | HandleKind::Selection, None) => handle.values.iter().map(String::as_str).collect(),
                _ => vec!["false", "true"],
            };
            expected.sort_unstable();
            if cases != expected {
                return Err(self.err(format!(
                    "switch on `{}` must have exactly the cases {}",
                    handle.id,
                    expected.join(", ")
                )));
            }
        }
        let suffix = handle.is.as_ref().map(|value| format!("?{value}"));
        let entry = self.entry(&handle.id, &handle.at, suffix.as_deref());
        dto.name = Some(entry.clone());
        self.record(handle, role, entry, handle.is.clone());
        Ok(())
    }

    /// Names a button or region after its click handle and records its click, enabled, and state uses.
    fn button(&mut self, dto: &mut ElementDto, scope: &Scope) -> Result<()> {
        let Some(click) = dto.on_click.clone() else {
            if dto.enabled.is_some() || dto.state.is_some() {
                return Err(self.err("a button with `enabled` or `state` requires a click handle".into()));
            }
            return Ok(());
        };
        if dto.name.is_some() {
            return Err(self.err("button element sets both `name` and `on_click`".into()));
        }
        let kind = self.declare(&click, HandleRole::Click, scope)?;
        let suffix = click.set.as_ref().map(|value| format!("={value}"));
        let entry = self.entry(&click.id, &click.at, suffix.as_deref());
        if kind == HandleKind::Builtin {
            dto.default = Some("close".into());
        }
        if let (Some(_), Some(_)) = (&dto.enabled, &dto.state) {
            return Err(self.err(format!("button `{entry}` sets both `enabled` and `state`")));
        }
        if let Some(enabled) = dto.enabled.clone() {
            self.declare(&enabled, HandleRole::Enabled, scope)?;
            self.record(&enabled, HandleRole::Enabled, entry.clone(), enabled.is.clone());
        }
        if let Some(state) = dto.state.clone() {
            self.declare(&state, HandleRole::State, scope)?;
            let keys: Vec<&String> = dto.states.keys().collect();
            let mut values: Vec<&String> = state.values.iter().collect();
            values.sort_unstable();
            if keys != values {
                return Err(self.err(format!(
                    "button `{entry}` must declare one state per value of `{}`: {}",
                    state.id,
                    state.values.join(", ")
                )));
            }
            self.record(&state, HandleRole::State, entry.clone(), None);
        }
        dto.name = Some(entry.clone());
        self.record(&click, HandleRole::Click, entry, click.set.clone());
        Ok(())
    }

    /// Names the cells of a repeater rendered per cell and returns its cell count. Cells click into the indexed
    /// `on_click` action, one index per cell.
    fn repeater(&mut self, dto: &mut ElementDto) -> Result<u32> {
        if !dto.children.is_empty() {
            return Err(self.err("repeater element sets both `children` and `cells`".into()));
        }
        let count = dto.cells.as_ref().map_or(0, Vec::len) as u32;
        if count == 0 {
            return Err(self.err("repeater `cells` must render at least one cell".into()));
        }
        let Some(click) = dto.on_click.clone() else {
            let name = dto.name.clone().ok_or_else(|| self.err("repeater requires `name` or `on_click`".into()))?;
            validate_name(&name, "repeater")?;
            dto.cell_buttons = (0..count).map(|i| self.entry(&name, &[i], None)).collect();
            return Ok(count);
        };
        if dto.name.is_some() {
            return Err(self.err("repeater element sets both `name` and `on_click`".into()));
        }
        if click.kind != "action" || !click.at.is_empty() || click.shape != [count] {
            return Err(self.err(format!(
                "repeater `on_click` must be an action indexed by its {count} cells (`shape: [{count}]`), not `{}`",
                click.id
            )));
        }
        self.declare(&HandleRefDto { at: vec![0], ..click.clone() }, HandleRole::Click, &Scope::default())?;
        dto.name = Some(self.entry(&click.id, &[], None));
        dto.cell_buttons = Vec::with_capacity(count as usize);
        for i in 0..count {
            let entry = self.entry(&click.id, &[i], None);
            dto.cell_buttons.push(entry.clone());
            self.record(&HandleRefDto { at: vec![i], ..click.clone() }, HandleRole::Click, entry, None);
        }
        dto.cell_action = true;
        Ok(count)
    }

    /// Validates `handle` for `role` and merges its declaration into this surface's handles.
    fn declare(&mut self, handle: &HandleRefDto, role: HandleRole, scope: &Scope) -> Result<HandleKind> {
        let id = &handle.id;
        let kind = parse_kind(&handle.kind)
            .ok_or_else(|| self.err(format!("handle `{id}` has unknown kind `{}`", handle.kind)))?;
        if kind == HandleKind::Builtin {
            if id != CLOSE {
                return Err(self.err(format!("unknown runtime action `{id}`; known actions: {CLOSE}")));
            }
        } else {
            validate_name(id, "handle")?;
        }
        if !role_accepts(role, kind) {
            return Err(self.err(format!("a {} handle cannot be used as {}", kind.id(), role_noun(role))));
        }
        if self.hud && matches!(kind, HandleKind::Toggle | HandleKind::Selection) {
            return Err(self.err(format!("{} `{id}` is window state; HUDs read flags and values", kind.id())));
        }
        let enumerated = matches!(kind, HandleKind::Value | HandleKind::Selection);
        if enumerated {
            if handle.values.is_empty() {
                return Err(self.err(format!("{} `{id}` requires at least one value", kind.id())));
            }
            for (i, value) in handle.values.iter().enumerate() {
                validate_name(value, &format!("{} `{id}` value", kind.id()))?;
                if handle.values[..i].contains(value) {
                    return Err(self.err(format!("{} `{id}` repeats value `{value}`", kind.id())));
                }
            }
        } else if !handle.values.is_empty() {
            return Err(self.err(format!("{} `{id}` does not take values", kind.id())));
        }
        let indexable = matches!(
            kind,
            HandleKind::Flag
                | HandleKind::Value
                | HandleKind::Text
                | HandleKind::Sprite
                | HandleKind::Items
                | HandleKind::Action
        );
        if !handle.shape.is_empty() && (!indexable || handle.shape.len() > 2 || handle.shape.contains(&0)) {
            return Err(self.err(format!(
                "{} `{id}` has an invalid shape; flags, values, text, sprites, items, and actions take `[n]` or \
                 `[rows, columns]` of positive sizes",
                kind.id()
            )));
        }
        if handle.at.len() != handle.shape.len() {
            return Err(self.err(match handle.shape.len() {
                0 => format!("{} `{id}` is not indexed", kind.id()),
                rank => format!("{} `{id}` is indexed; read it with `.at()` and {rank} index values", kind.id()),
            }));
        }
        if let Some((i, len)) = handle.at.iter().zip(&handle.shape).find(|(i, len)| i >= len) {
            return Err(self.err(format!("{} `{id}` index {i} is outside its size {len}", kind.id())));
        }
        if let (Some(cells), false) = (scope.cells, handle.at.is_empty()) {
            let size = handle.shape.iter().product::<u32>();
            if size != cells {
                return Err(self.err(format!(
                    "{} `{id}` has {size} entries, but the repeater rendering it has {cells} cells",
                    kind.id()
                )));
            }
        }
        let initial = match (kind, &handle.initial) {
            (HandleKind::Toggle, None) => Some("false".to_string()),
            (HandleKind::Toggle, Some(serde_json::Value::Bool(value))) => Some(value.to_string()),
            (HandleKind::Selection, None) => handle.values.first().cloned(),
            (HandleKind::Selection, Some(serde_json::Value::String(value))) if handle.values.contains(value) => {
                Some(value.clone())
            }
            (_, None) => None,
            (_, Some(value)) => return Err(self.err(format!("{} `{id}` has invalid `initial` {value}", kind.id()))),
        };
        if handle.selectable && kind != HandleKind::Collection {
            return Err(self.err(format!("{} `{id}` cannot be `selectable`", kind.id())));
        }
        if !handle.only.is_empty() && kind != HandleKind::Sprite {
            return Err(self.err(format!("{} `{id}` cannot take `only`; only sprite handles narrow", kind.id())));
        }
        for (i, sprite) in handle.only.iter().enumerate() {
            validate_name(sprite, &format!("sprite `{id}` `only` entry"))?;
            if handle.only[..i].contains(sprite) {
                return Err(self.err(format!("sprite `{id}` lists `{sprite}` twice in `only`")));
            }
        }
        let check_value = |value: &Option<String>, field: &str| -> Result<()> {
            match value {
                Some(value) if !handle.values.contains(value) => {
                    Err(self.err(format!("`{id}.{field}(\"{value}\")` is not one of its values")))
                }
                _ => Ok(()),
            }
        };
        check_value(&handle.is, "is")?;
        check_value(&handle.set, "set")?;
        let condition = matches!(role, HandleRole::Switch | HandleRole::Enabled);
        if handle.is.is_some() && !(condition && enumerated) {
            return Err(self.err(format!("`{id}.is()` is a condition for `when` or `enabled`")));
        }
        if role == HandleRole::Enabled && enumerated && handle.is.is_none() {
            return Err(self.err(format!("`enabled` needs a condition such as `{id}.is(...)`")));
        }
        if (handle.set.is_some()) != (role == HandleRole::Click && kind == HandleKind::Selection) {
            return Err(self.err(format!("selection `{id}` is set on click with `{id}.set(...)`")));
        }
        let declared = Handle {
            kind,
            values: handle.values.clone(),
            shape: handle.shape.clone(),
            initial,
            selectable: handle.selectable,
            only: handle.only.clone(),
            uses: Vec::new(),
        };
        match self.handles.get(id) {
            Some(existing) if !existing.same_declaration(&declared) => {
                return Err(
                    self.err(format!("handle `{id}` is declared twice with different kinds, values, or shapes"))
                );
            }
            Some(_) => {}
            None => {
                self.handles.insert(id.clone(), declared);
            }
        }
        Ok(kind)
    }

    fn record(&mut self, handle: &HandleRefDto, role: HandleRole, entry: String, value: Option<String>) {
        let uses = &mut self.handles.get_mut(&handle.id).expect("declared before use").uses;
        uses.push(HandleUse { role, entry, at: handle.at.clone(), value });
    }

    /// A unique entry name for a use of `id` at `at`.
    fn entry(&mut self, id: &str, at: &[u32], suffix: Option<&str>) -> String {
        let base = format!("{}{}", IndexedBinding::entry_name(id, at), suffix.unwrap_or_default());
        let mut name = base.clone();
        let mut copy = 1;
        while !self.names.insert(name.clone()) {
            copy += 1;
            name = format!("{base}~{copy}");
        }
        name
    }

    fn err(&self, message: String) -> Error {
        Error::Validation(format!("{}: {message}", self.owner))
    }
}

/// Rejects handles declared differently by two surfaces.
pub(in crate::authoring) fn check_shared<'a>(
    surfaces: impl IntoIterator<Item = (&'a str, &'a BTreeMap<String, Handle>)>,
) -> Result<()> {
    let mut seen: BTreeMap<&str, (&str, &Handle)> = BTreeMap::new();
    for (owner, handles) in surfaces {
        for (id, handle) in handles {
            match seen.get(id.as_str()) {
                Some((first, existing)) if !existing.same_declaration(handle) => {
                    return Err(Error::Validation(format!(
                        "handle `{id}` is declared differently by {first} and {owner}; handles sharing an id must \
                         agree on kind, values, and shape"
                    )));
                }
                Some(_) => {}
                None => {
                    seen.insert(id, (owner, handle));
                }
            }
        }
    }
    Ok(())
}

fn parse_kind(kind: &str) -> Option<HandleKind> {
    Some(match kind {
        "flag" => HandleKind::Flag,
        "toggle" => HandleKind::Toggle,
        "value" => HandleKind::Value,
        "selection" => HandleKind::Selection,
        "text" => HandleKind::Text,
        "sprite" => HandleKind::Sprite,
        "items" => HandleKind::Items,
        "collection" => HandleKind::Collection,
        "action" => HandleKind::Action,
        "input" => HandleKind::Input,
        "builtin" => HandleKind::Builtin,
        _ => return None,
    })
}

fn role_accepts(role: HandleRole, kind: HandleKind) -> bool {
    use HandleKind as K;
    match role {
        HandleRole::Slot => kind == K::Text,
        HandleRole::SpriteSlot => kind == K::Sprite,
        HandleRole::Item => kind == K::Items,
        HandleRole::Collection => kind == K::Collection,
        HandleRole::Input => kind == K::Input,
        HandleRole::Switch | HandleRole::Enabled => matches!(kind, K::Flag | K::Toggle | K::Value | K::Selection),
        HandleRole::Click => matches!(kind, K::Action | K::Builtin | K::Selection | K::Toggle),
        HandleRole::State => matches!(kind, K::Value | K::Selection),
    }
}

fn role_noun(role: HandleRole) -> &'static str {
    match role {
        HandleRole::Slot => "text",
        HandleRole::SpriteSlot => "an icon",
        HandleRole::Item => "an item",
        HandleRole::Collection => "a collection",
        HandleRole::Input => "an anvil input",
        HandleRole::Switch => "a switch or show condition",
        HandleRole::Click => "a click",
        HandleRole::Enabled => "a button's `enabled`",
        HandleRole::State => "a button's `state`",
    }
}
