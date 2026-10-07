use std::collections::BTreeMap;

use super::ElementDto;
use crate::authoring::parse::validate_name;
use crate::ir::{IndexedBinding, IndexedKind};
use crate::{Error, Result};

/// One indexed binding family while its elements are collected.
struct Family {
    kind: IndexedKind,
    rank: usize,
    /// Each authored index and the switch cases it appears in; `None` outside any switch case.
    seen: BTreeMap<Vec<u32>, Vec<Option<CaseScope>>>,
    /// Case values of a switch family, in authoring order.
    cases: Vec<String>,
}

/// Where an element sits: its enclosing repeater, its closest switch, and the switch case it is in, with each
/// switch numbered in authoring order.
#[derive(Clone, Default)]
struct Scope {
    repeater: Option<String>,
    switch: Option<usize>,
    case: Option<CaseScope>,
}

type CaseScope = (usize, String);

/// Renames every element authored with an `index` to its flattened entry name and returns the families by name.
///
/// Families must index one kind with a consistent rank, cover every index within their shape, and stay outside
/// repeaters; switch families must share their case values.
pub(in crate::authoring) fn flatten_indexed(
    children: &mut [ElementDto],
    owner: &str,
) -> Result<BTreeMap<String, IndexedBinding>> {
    let mut families = BTreeMap::new();
    let mut switches = 0;
    for child in children.iter_mut() {
        collect(child, &Scope::default(), owner, &mut families, &mut switches)?;
    }
    families.into_iter().map(|(name, family)| Ok((name.clone(), finish(&name, family, owner)?))).collect()
}

fn collect(
    dto: &mut ElementDto,
    scope: &Scope,
    owner: &str,
    families: &mut BTreeMap<String, Family>,
    switches: &mut usize,
) -> Result<()> {
    if let Some(index) = dto.index.clone() {
        let name = flatten(dto, &index, scope, owner, families)?;
        dto.name = Some(IndexedBinding::entry_name(&name, &index));
    }
    let mut inner = scope.clone();
    match dto.kind.as_str() {
        "repeater" => inner.repeater = inner.repeater.or_else(|| dto.name.clone()),
        "switch" => {
            *switches += 1;
            inner.switch = Some(*switches);
        }
        "case" => inner.case = inner.switch.zip(dto.value.clone()),
        _ => {}
    }
    for child in &mut dto.children {
        collect(child, &inner, owner, families, switches)?;
    }
    Ok(())
}

/// Records `dto` in its family and returns the family name.
fn flatten(
    dto: &ElementDto,
    index: &[u32],
    scope: &Scope,
    owner: &str,
    families: &mut BTreeMap<String, Family>,
) -> Result<String> {
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
    if let Some(repeater) = &scope.repeater {
        return Err(Error::Validation(format!(
            "{owner}: indexed binding `{name}` is inside repeater `{repeater}`, whose cells are already indexed"
        )));
    }
    let cases: Vec<String> = dto.children.iter().filter_map(|case| case.value.clone()).collect();
    let family = families.entry(name.clone()).or_insert_with(|| Family {
        kind,
        rank: index.len(),
        seen: BTreeMap::new(),
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
    // An index may repeat only once in each case of one switch, where the copies share the entry's binding.
    let scopes = family.seen.entry(index.to_vec()).or_default();
    let repeats = match &scope.case {
        Some((switch, case)) => {
            !scopes.iter().all(|other| other.as_ref().is_some_and(|(other, value)| other == switch && value != case))
        }
        None => !scopes.is_empty(),
    };
    if repeats {
        return Err(Error::Validation(format!("{owner}: indexed binding `{name}` repeats index {index:?}")));
    }
    scopes.push(scope.case.clone());
    Ok(name)
}

fn finish(name: &str, family: Family, owner: &str) -> Result<IndexedBinding> {
    let too_large = || Error::Validation(format!("{owner}: indexed binding `{name}` has an index too large to cover"));
    let shape = (0..family.rank)
        .map(|dim| family.seen.keys().try_fold(0, |len: u32, index| index[dim].checked_add(1).map(|end| len.max(end))))
        .map(|len| len.ok_or_else(too_large))
        .collect::<Result<Vec<u32>>>()?;
    // A family missing an index is missing one among its first `seen.len() + 1` indices.
    let limit = family.seen.len() + 1;
    let missing = match shape.as_slice() {
        [len] => (0..*len).map(|i| vec![i]).take(limit).find(|index| !family.seen.contains_key(index)),
        [rows, columns] => (0..*rows)
            .flat_map(|i| (0..*columns).map(move |j| vec![i, j]))
            .take(limit)
            .find(|index| !family.seen.contains_key(index)),
        _ => None,
    };
    if let Some(index) = missing {
        return Err(Error::Validation(format!("{owner}: indexed binding `{name}` is missing index {index:?}")));
    }
    Ok(IndexedBinding { kind: family.kind, shape })
}
