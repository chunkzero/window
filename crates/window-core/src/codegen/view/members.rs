use std::collections::{BTreeMap, BTreeSet};

use crate::Result;
use crate::codegen::naming;
use crate::ir::ButtonDefault;
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
    Value {
        kind: ValueKind,
        source: String,
        member: String,
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
    /// enum whose constants are the upper-cased case values.
    Switch {
        source: String,
        member: String,
        cases: Option<SwitchEnum>,
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
    taken: BTreeMap<String, String>,
    members: Vec<Member>,
    grouped: BTreeSet<(ValueKind, String)>,
    grouped_buttons: BTreeSet<String>,
}

impl<'a> Members<'a> {
    pub(super) fn new(class_name: &'a str) -> Self {
        Self {
            class_name,
            taken: BTreeMap::new(),
            members: Vec::new(),
            grouped: BTreeSet::new(),
            grouped_buttons: BTreeSet::new(),
        }
    }

    /// Window members: repeater groups first, then ungrouped slots, sprites, buttons, items,
    /// collections, and anvil inputs.
    pub(super) fn of_window(class_name: &'a str, window: &WindowEntry) -> Result<Vec<Member>> {
        let mut members = Self::new(class_name);
        for (name, group) in &window.groups {
            members.group(name, group)?;
        }
        for (slot, entry) in &window.slots {
            if entry.text.is_none() && !members.is_grouped(ValueKind::Slot, slot) {
                members.value(ValueKind::Slot, slot)?;
            }
        }
        for (slot, entry) in &window.sprite_slots {
            if entry.sprite.is_none() && !members.is_grouped(ValueKind::Sprite, slot) {
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
            if !members.is_grouped(ValueKind::Item, item) {
                members.value(ValueKind::Item, item)?;
            }
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

    pub(super) fn value(&mut self, kind: ValueKind, source: &str) -> Result<()> {
        let member = kind.member(source);
        self.claim(&member, format!("{} `{source}`", kind.label()))?;
        self.members.push(Member::Value { kind, source: source.into(), member });
        Ok(())
    }

    pub(super) fn switches(&mut self, switches: &BTreeMap<String, SwitchEntry>) -> Result<()> {
        for (source, switch) in switches {
            let member = naming::slot_member(source);
            self.claim(&member, format!("switch `{source}`"))?;
            let mut values: Vec<&str> = switch.cases.iter().map(|case| case.value.as_str()).collect();
            values.sort_unstable();
            let cases = if values == ["false", "true"] {
                None
            } else {
                let name = naming::type_name(source);
                self.claim(&name, format!("switch `{source}` enum"))?;
                let constants =
                    switch.cases.iter().map(|case| (case.value.to_uppercase(), case.value.clone())).collect();
                Some(SwitchEnum { name, constants })
            };
            self.members.push(Member::Switch { source: source.clone(), member, cases });
        }
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
        naming::check_member(member, &source, self.class_name, &mut self.taken)
    }
}

fn non_empty(sources: &[String]) -> Vec<String> {
    sources.iter().filter(|source| !source.is_empty()).cloned().collect()
}
