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
