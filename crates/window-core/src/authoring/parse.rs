use std::collections::BTreeSet;

use crate::{Error, Result};

pub(super) fn json_object_fields<E>(
    value: &serde_json::Value,
    context: &str,
) -> std::result::Result<BTreeSet<String>, E>
where
    E: serde::de::Error,
{
    match value {
        serde_json::Value::Object(object) => Ok(object.keys().cloned().collect()),
        _ => Err(E::custom(format!("{context} must be an object"))),
    }
}

pub(super) fn reject_unexpected_fields(fields: &BTreeSet<String>, allowed: &[&str], context: &str) -> Result<()> {
    for field in fields {
        if !allowed.contains(&field.as_str()) {
            return Err(Error::Validation(format!("{context} does not accept field `{field}`")));
        }
    }
    Ok(())
}

pub(super) fn validate_name(name: &str, kind: &str) -> Result<()> {
    let mut bytes = name.bytes();
    let valid = bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if valid {
        Ok(())
    } else {
        Err(Error::Validation(format!("{kind} name `{name}` is invalid; must match ^[a-z][a-z0-9_]*$")))
    }
}

/// The snake_case name of a runtime sprite authored as `snake_case` or `camelCase`, such as `lamp_on` for `lampOn`.
pub(super) fn sprite_key(name: &str, kind: &str) -> Result<String> {
    let mut bytes = name.bytes();
    let alphanumeric =
        bytes.next().is_some_and(|b| b.is_ascii_lowercase()) && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_');
    let mixed = name.bytes().any(|b| b.is_ascii_uppercase()) && name.contains('_');
    if !alphanumeric || mixed {
        return Err(Error::Validation(format!(
            "{kind} name `{name}` is invalid; must be snake_case (^[a-z][a-z0-9_]*$) or camelCase (^[a-z][a-zA-Z0-9]*$)"
        )));
    }
    let mut key = String::with_capacity(name.len() + 2);
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            key.push('_');
        }
        key.push(c.to_ascii_lowercase());
    }
    Ok(key)
}
