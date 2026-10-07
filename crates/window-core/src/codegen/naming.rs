use std::collections::BTreeMap;

use crate::{Error, Result};

pub(super) fn check_member(
    member: &str,
    source: &str,
    class_name: &str,
    extra_reserved: &[&str],
    taken: &mut BTreeMap<String, String>,
) -> Result<()> {
    if !is_valid_identifier(member) {
        return Err(Error::Validation(format!(
            "{source} maps to invalid Kotlin identifier `{member}` (class {class_name})"
        )));
    }
    if RESERVED_MEMBERS.contains(&member) || RESERVED_TYPES.contains(&member) || extra_reserved.contains(&member) {
        return Err(Error::Validation(format!(
            "{source} collides with reserved WindowView member `{member}` (class {class_name})"
        )));
    }
    if let Some(existing) = taken.insert(member.to_string(), source.to_string()) {
        return Err(Error::Validation(format!(
            "{source} and {existing} both map to member `{member}` (class {class_name})"
        )));
    }
    Ok(())
}

pub(super) fn validate_package(package_name: &str) -> Result<()> {
    let valid = !package_name.is_empty() && package_name.split('.').all(is_valid_identifier);
    if valid { Ok(()) } else { Err(Error::Validation(format!("Kotlin package `{package_name}` is invalid"))) }
}

pub(super) fn class_name(window: &str) -> String {
    format!("{}View", pascal(window))
}

pub(super) fn hud_class_name(hud: &str) -> String {
    format!("{}Hud", pascal(hud))
}

/// The nested Kotlin type name for `name`.
pub(super) fn type_name(name: &str) -> String {
    pascal(name)
}

/// The typed definition property for the window or HUD `name`.
pub(super) fn definition_member(name: &str) -> String {
    camel(name)
}

pub(super) fn slot_member(slot: &str) -> String {
    camel(slot)
}

pub(super) fn button_member(button: &str) -> String {
    format!("on{}", pascal(button))
}

fn pascal(name: &str) -> String {
    name.split('_').filter(|part| !part.is_empty()).map(capitalize).collect()
}

fn camel(name: &str) -> String {
    let mut parts = name.split('_').filter(|part| !part.is_empty());
    let Some(first) = parts.next() else {
        return String::new();
    };
    let mut out = first.to_string();
    for part in parts {
        out.push_str(&capitalize(part));
    }
    out
}

fn capitalize(part: &str) -> String {
    let mut chars = part.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

pub(super) fn is_valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_alphabetic() || first == '_') {
        return false;
    }
    if !chars.all(|c| c.is_alphanumeric() || c == '_') {
        return false;
    }
    !HARD_KEYWORDS.contains(&name)
}

/// Members `HudView` declares beyond the shared set.
pub(super) const HUD_RESERVED_MEMBERS: &[&str] = &["render", "channel"];

const RESERVED_MEMBERS: &[&str] = &[
    "player",
    "host",
    "open",
    "show",
    "state",
    "bind",
    "onOpen",
    "onClose",
    "close",
    "refresh",
    "windowName",
    "hudName",
    "onShow",
    "onHide",
    "hide",
];

/// Types generated views reference by simple name, which a nested type must not shadow.
const RESERVED_TYPES: &[&str] = &[
    "Boolean",
    "Click",
    "Component",
    "HudScope",
    "HudView",
    "I",
    "IndexedClick",
    "Int",
    "ItemStack",
    "MinestomHost",
    "MultistomHost",
    "Player",
    "String",
    "Unit",
    "WindowDefinitions",
    "WindowHost",
    "WindowHudDefinitions",
    "WindowScope",
    "WindowSprite",
    "WindowView",
];

const HARD_KEYWORDS: &[&str] = &[
    "as",
    "break",
    "class",
    "continue",
    "do",
    "else",
    "false",
    "for",
    "fun",
    "if",
    "in",
    "interface",
    "is",
    "null",
    "object",
    "package",
    "return",
    "super",
    "this",
    "throw",
    "true",
    "try",
    "typealias",
    "typeof",
    "val",
    "var",
    "when",
    "while",
];
