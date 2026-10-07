use std::collections::{BTreeMap, BTreeSet};

use crate::Result;
use crate::codegen::naming;
use crate::ir::{ButtonDefault, IndexedBinding, IndexedKind};
use crate::manifest::{CollectionEntry, RepeatGroupEntry, SwitchEntry, WindowEntry};

/// A runtime-rendered value bound by name: a text slot, sprite id, or inventory item.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ValueKind {
    Slot,
    Sprite,
    Item,
}

impl ValueKind {
    fn member(self, name: &str) -> String {
        let base = naming::slot_member(name);
        match self {
            Self::Slot => base,
            Self::Sprite => format!("{base}Sprite"),
            Self::Item => format!("{base}Item"),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Slot => "slot",
            Self::Sprite => "sprite slot",
            Self::Item => "item",
        }
    }

    pub(super) fn noun(self) -> &'static str {
        match self {
            Self::Slot => "slot",
            Self::Sprite => "runtime sprite id",
            Self::Item => "inventory item",
        }
    }

    /// The Kotlin return type of this kind's member, given the view's item type.
    pub(super) fn return_type(self, item: &str) -> String {
        match self {
            Self::Slot => "Component".into(),
            Self::Sprite => "String?".into(),
            Self::Item => format!("{item}?"),
        }
    }

    /// The `WindowScope`/`HudScope` binder function for this kind.
    pub(super) fn binder(self) -> &'static str {
        match self {
            Self::Slot => "slot",
            Self::Sprite => "sprite",
            Self::Item => "item",
        }
    }
}

/// One generated view member and the manifest names it binds.
pub(super) enum Member {
    /// One value, or with a non-empty `shape` an indexed family whose entries are named
    /// [`IndexedBinding::entry_name`].
    Value {
        kind: ValueKind,
        source: String,
        member: String,
        shape: Vec<u32>,
    },
    GroupValue {
        kind: ValueKind,
        group: String,
        field: String,
        sources: Vec<String>,
        member: String,
    },
    GroupButton {
        group: String,
        sources: Vec<String>,
        member: String,
    },
    Button {
        source: String,
        member: String,
        is_close: bool,
    },
    Collection {
        source: String,
        item_member: String,
        handler: Option<String>,
        selection: Option<String>,
    },
    AnvilInput {
        source: String,
        member: String,
    },
    /// A switch bound to a `Boolean` when its cases are exactly `true` and `false`, otherwise to a generated
    /// enum whose constants are the upper-cased case values. A non-empty `shape` makes it an indexed family.
    Switch {
        source: String,
        member: String,
        cases: Option<SwitchEnum>,
        shape: Vec<u32>,
    },
}

/// A generated enum naming the cases of one switch.
pub(super) struct SwitchEnum {
    pub(super) name: String,
    /// `(constant, case value)` pairs in authoring order.
    pub(super) constants: Vec<(String, String)>,
}

/// Collects members in declaration order, rejecting invalid, reserved, or colliding names.
pub(super) struct Members<'a> {
    class_name: &'a str,
    reserved: &'a [&'a str],
    taken: BTreeMap<String, String>,
    members: Vec<Member>,
    grouped: BTreeSet<(ValueKind, String)>,
    grouped_buttons: BTreeSet<String>,
    indexed_switches: BTreeSet<String>,
}

impl<'a> Members<'a> {
    pub(super) fn new(class_name: &'a str, reserved: &'a [&'a str]) -> Self {
        Self {
            class_name,
            reserved,
            taken: BTreeMap::new(),
            members: Vec::new(),
            grouped: BTreeSet::new(),
            grouped_buttons: BTreeSet::new(),
            indexed_switches: BTreeSet::new(),
        }
    }

    /// Window members: repeater groups and indexed families first, then ungrouped slots, sprites, buttons,
    /// items, collections, and anvil inputs.
    pub(super) fn of_window(class_name: &'a str, window: &WindowEntry) -> Result<Vec<Member>> {
        let mut members = Self::new(class_name, &[]);
        for (name, group) in &window.groups {
            members.group(name, group)?;
        }
        members.indexed(&window.indexed, &window.switches)?;
        for (slot, entry) in &window.slots {
            if entry.text.is_none() {
                members.value(ValueKind::Slot, slot)?;
            }
        }
        for (slot, entry) in &window.sprite_slots {
            if entry.sprite.is_none() {
                members.value(ValueKind::Sprite, slot)?;
            }
        }
        members.switches(&window.switches)?;
        for (button, entry) in &window.buttons {
            if entry.action && !members.grouped_buttons.contains(button) {
                members.button(button, entry.default == Some(ButtonDefault::Close))?;
            }
        }
        for item in window.items.keys() {
            members.value(ValueKind::Item, item)?;
        }
        for (collection, entry) in &window.collections {
            members.collection(collection, entry)?;
        }
        for input in window.inputs.keys() {
            let member = format!("{}Changed", naming::button_member(input));
            members.claim(&member, format!("anvil input `{input}`"))?;
            members.members.push(Member::AnvilInput { source: input.clone(), member });
        }
        Ok(members.finish())
    }

    pub(super) fn finish(self) -> Vec<Member> {
        self.members
    }

    /// A value member, unless `source` is an entry of an indexed family or repeater group.
    pub(super) fn value(&mut self, kind: ValueKind, source: &str) -> Result<()> {
        if self.is_grouped(kind, source) {
            return Ok(());
        }
        let member = kind.member(source);
        self.claim(&member, format!("{} `{source}`", kind.label()))?;
        self.members.push(Member::Value { kind, source: source.into(), member, shape: Vec::new() });
        Ok(())
    }

    /// One member per indexed family, taking the family's index as parameters.
    pub(super) fn indexed(
        &mut self,
        families: &BTreeMap<String, IndexedBinding>,
        switches: &BTreeMap<String, SwitchEntry>,
    ) -> Result<()> {
        for (family, binding) in families {
            let names = entry_names(family, &binding.shape);
            let kind = match binding.kind {
                IndexedKind::Slot => ValueKind::Slot,
                IndexedKind::SpriteSlot => ValueKind::Sprite,
                IndexedKind::Switch => {
                    let first = names.first().and_then(|name| switches.get(name));
                    let first = first.ok_or_else(|| {
                        crate::Error::Validation(format!("indexed switch `{family}` has no switch entries"))
                    })?;
                    self.switch(family, first, binding.shape.clone())?;
                    self.indexed_switches.extend(names);
                    continue;
                }
            };
            let member = kind.member(family);
            self.claim(&member, format!("indexed {} `{family}`", kind.label()))?;
            self.grouped.extend(names.into_iter().map(|name| (kind, name)));
            self.members.push(Member::Value { kind, source: family.clone(), member, shape: binding.shape.clone() });
        }
        Ok(())
    }

    pub(super) fn switches(&mut self, switches: &BTreeMap<String, SwitchEntry>) -> Result<()> {
        for (source, switch) in switches {
            if !self.indexed_switches.contains(source) {
                self.switch(source, switch, Vec::new())?;
            }
        }
        Ok(())
    }

    fn switch(&mut self, source: &str, switch: &SwitchEntry, shape: Vec<u32>) -> Result<()> {
        let member = naming::slot_member(source);
        self.claim(&member, format!("switch `{source}`"))?;
        let mut values: Vec<&str> = switch.cases.iter().map(|case| case.value.as_str()).collect();
        values.sort_unstable();
        let cases = if values == ["false", "true"] {
            None
        } else {
            let name = naming::type_name(source);
            self.claim(&name, format!("switch `{source}` enum"))?;
            let constants = switch.cases.iter().map(|case| (case.value.to_uppercase(), case.value.clone())).collect();
            Some(SwitchEnum { name, constants })
        };
        self.members.push(Member::Switch { source: source.into(), member, cases, shape });
        Ok(())
    }

    fn group(&mut self, name: &str, group: &RepeatGroupEntry) -> Result<()> {
        let fields = [
            (ValueKind::Slot, &group.slots),
            (ValueKind::Sprite, &group.sprite_slots),
            (ValueKind::Item, &group.items),
        ];
        for (kind, fields) in fields {
            for (field, sources) in fields {
                self.group_value(kind, name, field, sources)?;
            }
        }
        let sources = non_empty(&group.buttons);
        if sources.is_empty() {
            return Ok(());
        }
        let member = naming::button_member(name);
        self.claim(&member, format!("repeater `{name}` button"))?;
        self.grouped_buttons.extend(sources.iter().cloned());
        self.members.push(Member::GroupButton { group: name.into(), sources, member });
        Ok(())
    }

    fn group_value(&mut self, kind: ValueKind, group: &str, field: &str, sources: &[String]) -> Result<()> {
        let sources = non_empty(sources);
        if sources.is_empty() {
            return Ok(());
        }
        let member = kind.member(&format!("{group}_{field}"));
        self.claim(&member, format!("repeater `{group}` {} `{field}`", kind.label()))?;
        self.grouped.extend(sources.iter().map(|source| (kind, source.clone())));
        self.members.push(Member::GroupValue { kind, group: group.into(), field: field.into(), sources, member });
        Ok(())
    }

    fn button(&mut self, source: &str, is_close: bool) -> Result<()> {
        let member = naming::button_member(source);
        self.claim(&member, format!("button `{source}`"))?;
        self.members.push(Member::Button { source: source.into(), member, is_close });
        Ok(())
    }

    fn collection(&mut self, source: &str, entry: &CollectionEntry) -> Result<()> {
        let item_member = ValueKind::Item.member(source);
        self.claim(&item_member, format!("collection `{source}` item"))?;
        let handler = if entry.action {
            let member = naming::button_member(source);
            self.claim(&member, format!("collection `{source}`"))?;
            Some(member)
        } else {
            None
        };
        let selection = if entry.selection.is_empty() {
            None
        } else {
            let member = format!("{}Selected", naming::slot_member(source));
            self.claim(&member, format!("collection `{source}` selection"))?;
            Some(member)
        };
        self.members.push(Member::Collection { source: source.into(), item_member, handler, selection });
        Ok(())
    }

    fn is_grouped(&self, kind: ValueKind, source: &str) -> bool {
        self.grouped.contains(&(kind, source.to_string()))
    }

    fn claim(&mut self, member: &str, source: String) -> Result<()> {
        naming::check_member(member, &source, self.class_name, self.reserved, &mut self.taken)
    }
}

/// The flattened entry names of an indexed family, row-major.
fn entry_names(family: &str, shape: &[u32]) -> Vec<String> {
    match shape {
        [len] => (0..*len).map(|i| IndexedBinding::entry_name(family, &[i])).collect(),
        [rows, columns] => {
            (0..*rows).flat_map(|i| (0..*columns).map(move |j| IndexedBinding::entry_name(family, &[i, j]))).collect()
        }
        _ => Vec::new(),
    }
}

fn non_empty(sources: &[String]) -> Vec<String> {
    sources.iter().filter(|source| !source.is_empty()).cloned().collect()
}
