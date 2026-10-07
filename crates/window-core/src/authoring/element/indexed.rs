use std::collections::{BTreeMap, BTreeSet};

use super::ElementDto;
use crate::authoring::parse::validate_name;
use crate::ir::{IndexedBinding, IndexedKind};
use crate::{Error, Result};

/// One indexed binding family while its elements are collected.
struct Family {
    kind: IndexedKind,
    rank: usize,
    seen: BTreeSet<Vec<u32>>,
    /// Case values of a switch family, in authoring order.
    cases: Vec<String>,
}

/// Renames every element authored with an `index` to its flattened entry name and returns the families by name.
///
/// Families must index one kind with a consistent rank, cover every index within their shape, and stay outside
/// repeaters; switch families must share their case values.
pub(in crate::authoring) fn flatten_indexed(
    children: &mut [ElementDto],
    owner: &str,
) -> Result<BTreeMap<String, IndexedBinding>> {
    let mut families = BTreeMap::new();
    for child in children.iter_mut() {
        collect(child, None, owner, &mut families)?;
    }
    families.into_iter().map(|(name, family)| Ok((name.clone(), finish(&name, family, owner)?))).collect()
}

fn collect(
    dto: &mut ElementDto,
    repeater: Option<&str>,
    owner: &str,
    families: &mut BTreeMap<String, Family>,
) -> Result<()> {
    if let Some(index) = dto.index.clone() {
        let (family, entry) = flatten(dto, &index, repeater, owner, families)?;
        dto.name = Some(IndexedBinding::entry_name(&family, &index));
        families.get_mut(&family).expect("family was just inserted").seen.insert(entry);
    }
    let repeater = if dto.kind == "repeater" { dto.name.as_deref().or(repeater) } else { repeater };
    let repeater = repeater.map(str::to_string);
    for child in &mut dto.children {
        collect(child, repeater.as_deref(), owner, families)?;
    }
    Ok(())
}

fn flatten(
    dto: &ElementDto,
    index: &[u32],
    repeater: Option<&str>,
    owner: &str,
    families: &mut BTreeMap<String, Family>,
) -> Result<(String, Vec<u32>)> {
    let kind = match dto.kind.as_str() {
        "slot" => IndexedKind::Slot,
        "sprite_slot" => IndexedKind::SpriteSlot,
        "switch" => IndexedKind::Switch,
        other => return Err(Error::Validation(format!("{other} element does not accept field `index`"))),
    };
    let name = dto.required("name")?;
    validate_name(&name, "indexed binding")?;
    if dto.sprite.is_some() {
        return Err(Error::Validation(format!(
            "{owner}: indexed binding `{name}` sets a fixed `sprite`, which needs no binding"
        )));
    }
    if !(1..=2).contains(&index.len()) {
        return Err(Error::Validation(format!(
            "{owner}: indexed binding `{name}` has {} index values; use one or two",
            index.len()
        )));
    }
    if let Some(repeater) = repeater {
        return Err(Error::Validation(format!(
            "{owner}: indexed binding `{name}` is inside repeater `{repeater}`, whose cells are already indexed"
        )));
    }
    let cases: Vec<String> = dto.children.iter().filter_map(|case| case.value.clone()).collect();
    let family = families.entry(name.clone()).or_insert_with(|| Family {
        kind,
        rank: index.len(),
        seen: BTreeSet::new(),
        cases: cases.clone(),
    });
    if family.kind != kind {
        return Err(Error::Validation(format!("{owner}: indexed binding `{name}` mixes element kinds")));
    }
    if family.rank != index.len() {
        return Err(Error::Validation(format!(
            "{owner}: indexed binding `{name}` mixes {} and {} index values",
            family.rank,
            index.len()
        )));
    }
    if family.cases != cases {
        return Err(Error::Validation(format!(
            "{owner}: indexed switch `{name}` must use the same case values at every index"
        )));
    }
    if family.seen.contains(index) {
        return Err(Error::Validation(format!("{owner}: indexed binding `{name}` repeats index {index:?}")));
    }
    Ok((name, index.to_vec()))
}

fn finish(name: &str, family: Family, owner: &str) -> Result<IndexedBinding> {
    let too_large = || Error::Validation(format!("{owner}: indexed binding `{name}` has an index too large to cover"));
    let shape = (0..family.rank)
        .map(|dim| family.seen.iter().try_fold(0, |len: u32, index| index[dim].checked_add(1).map(|end| len.max(end))))
        .map(|len| len.ok_or_else(too_large))
        .collect::<Result<Vec<u32>>>()?;
    // A family missing an index is missing one among its first `seen.len() + 1` indices.
    let limit = family.seen.len() + 1;
    let missing = match shape.as_slice() {
        [len] => (0..*len).map(|i| vec![i]).take(limit).find(|index| !family.seen.contains(index)),
        [rows, columns] => (0..*rows)
            .flat_map(|i| (0..*columns).map(move |j| vec![i, j]))
            .take(limit)
            .find(|index| !family.seen.contains(index)),
        _ => None,
    };
    if let Some(index) = missing {
        return Err(Error::Validation(format!("{owner}: indexed binding `{name}` is missing index {index:?}")));
    }
    Ok(IndexedBinding { kind: family.kind, shape })
}
