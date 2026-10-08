//! View members declared by typed handles, and the bindings of every entry that uses them.

use std::collections::BTreeMap;

use crate::codegen::literals::kt_string;
use crate::codegen::naming;
use crate::codegen::writer::KotlinWriter;
use crate::ir::{Handle, HandleKind, HandleRole, HandleUse, indexed_entry};

use super::Taken;

/// The members one handle declares.
pub(super) struct HandleMember {
    pub(super) id: String,
    pub(super) handle: Handle,
    /// The function, property, or handler the handle declares; empty for a runtime action.
    pub(super) member: String,
    /// The generated enum of a value or selection handle.
    pub(super) enum_name: Option<String>,
    /// The generated enum of a sprite handle narrowed with `only`.
    pub(super) sprite_enum: Option<String>,
    /// The `on…Changed` hook of UI-owned state.
    pub(super) hook: Option<String>,
    /// The shape constants: `<ID>_SIZE`, or `<ID>_ROWS` and `<ID>_COLUMNS`; a collection's `<ID>_SIZE` is its
    /// cell count.
    pub(super) constants: Vec<(String, u32)>,
}

/// The JVM getter and setter names Kotlin gives a property: `isFoo` keeps its getter name and sets via `setFoo`.
fn jvm_accessors(property: &str) -> (String, Option<String>) {
    let capitalize = |name: &str| {
        let (first, rest) = name.split_at(name.chars().next().map_or(0, char::len_utf8));
        format!("{}{rest}", first.to_uppercase())
    };
    match property.strip_prefix("is") {
        Some(rest) if rest.chars().next().is_some_and(|c| !c.is_lowercase()) => {
            (property.into(), Some(format!("set{rest}")))
        }
        _ => (format!("get{}", capitalize(property)), Some(format!("set{}", capitalize(property)))),
    }
}

impl HandleMember {
    /// The members of every handle of a view class named `class_name`; `cells` gives a collection handle's cell
    /// count. Rejects invalid, `reserved`, or colliding member names.
    pub(super) fn all(
        class_name: &str,
        reserved: &[&str],
        handles: &BTreeMap<String, Handle>,
        cells: impl Fn(&str) -> Option<u32>,
    ) -> crate::Result<Vec<Self>> {
        let mut taken = Taken::new();
        handles
            .iter()
            .map(|(id, handle)| {
                Self::new(id, handle, cells(id), |member, source| {
                    naming::check_member(member, &source, class_name, reserved, &mut taken)
                })
            })
            .collect()
    }

    /// Names the members of `handle`; `cells` is a collection's cell count, and `claim` rejects invalid or colliding
    /// names.
    fn new(
        id: &str,
        handle: &Handle,
        cells: Option<u32>,
        mut claim: impl FnMut(&str, String) -> crate::Result<()>,
    ) -> crate::Result<Self> {
        let kind = handle.kind.id();
        let member = match handle.kind {
            HandleKind::Sprite => format!("{}Sprite", naming::slot_member(id)),
            HandleKind::Action => naming::button_member(id),
            HandleKind::Input => format!("{}Changed", naming::button_member(id)),
            HandleKind::Builtin => String::new(),
            _ => naming::slot_member(id),
        };
        if !member.is_empty() {
            claim(&member, format!("{kind} `{id}`"))?;
        }
        if matches!(handle.kind, HandleKind::Toggle | HandleKind::Selection | HandleKind::Collection) {
            let (getter, setter) = jvm_accessors(&member);
            let mutable = handle.kind != HandleKind::Collection;
            for accessor in
                [Some(getter).filter(|getter| *getter != member), setter.filter(|_| mutable)].into_iter().flatten()
            {
                claim(&accessor, format!("{kind} `{id}` property accessor"))?;
            }
        }
        let enum_name = matches!(handle.kind, HandleKind::Value | HandleKind::Selection).then(|| naming::type_name(id));
        if let Some(name) = &enum_name {
            claim(name, format!("{kind} `{id}` enum"))?;
        }
        let sprite_enum = (!handle.only.is_empty()).then(|| format!("{}Sprite", naming::type_name(id)));
        if let Some(name) = &sprite_enum {
            claim(name, format!("{kind} `{id}` enum"))?;
        }
        let hook = matches!(handle.kind, HandleKind::Toggle | HandleKind::Selection)
            .then(|| format!("{}Changed", naming::button_member(id)));
        if let Some(hook) = &hook {
            claim(hook, format!("{kind} `{id}` change hook"))?;
        }
        let upper = id.to_uppercase();
        let constants = match (handle.shape.as_slice(), cells) {
            ([size], _) => vec![(format!("{upper}_SIZE"), *size)],
            ([rows, columns], _) => vec![(format!("{upper}_ROWS"), *rows), (format!("{upper}_COLUMNS"), *columns)],
            (_, Some(cells)) if handle.kind == HandleKind::Collection => vec![(format!("{upper}_SIZE"), cells)],
            _ => Vec::new(),
        };
        for (constant, _) in &constants {
            claim(constant, format!("{kind} `{id}` shape constant"))?;
        }
        Ok(Self { id: id.into(), handle: handle.clone(), member, enum_name, sprite_enum, hook, constants })
    }

    /// Whether the view needs `import` for any of `handles`.
    pub(super) fn imports(handles: &[HandleMember]) -> Vec<&'static str> {
        let has = |test: fn(&HandleMember) -> bool| handles.iter().any(test);
        [
            (has(|h| h.handle.kind == HandleKind::Action && h.handle.shape.is_empty()), "com.chunkzero.window.Click"),
            (
                has(|h| h.handle.kind == HandleKind::Action && !h.handle.shape.is_empty()),
                "com.chunkzero.window.IndexedClick",
            ),
            (has(|h| h.handle.kind == HandleKind::Collection), "com.chunkzero.window.WindowCollection"),
            (has(|h| h.handle.kind == HandleKind::Text), "net.kyori.adventure.text.Component"),
        ]
        .into_iter()
        .filter_map(|(used, import)| used.then_some(import))
        .collect()
    }

    pub(super) fn declare(&self, w: &mut KotlinWriter, item: &str) {
        let (id, member, params) = (&self.id, &self.member, index_params(&self.handle.shape));
        if let Some(name) = &self.enum_name {
            w.doc(format_args!("The values of `{id}`."));
            w.open(format_args!("public enum class {name}(public val value: String) {{"));
            for value in &self.handle.values {
                w.line(format_args!("{}({}),", value.to_uppercase(), kt_string(value)));
            }
            w.close("}");
            w.blank();
        }
        if let Some(name) = &self.sprite_enum {
            w.doc(format_args!("The sprites `{id}` shows."));
            w.open(format_args!("public enum class {name}(public val sprite: WindowSprite) {{"));
            for sprite in &self.handle.only {
                let constant = sprite.to_uppercase();
                w.line(format_args!("{constant}(WindowSprite.{constant}),"));
            }
            w.close("}");
            w.blank();
        }
        let enum_name = self.enum_name.as_deref().unwrap_or_default();
        let abstract_fun = |w: &mut KotlinWriter, doc: String, signature: String| {
            w.doc(doc);
            w.line(format_args!("protected abstract fun {signature}"));
            w.blank();
        };
        match self.handle.kind {
            HandleKind::Flag => abstract_fun(w, format!("The `{id}` flag."), format!("{member}({params}): Boolean")),
            HandleKind::Value => {
                abstract_fun(w, format!("The `{id}` value."), format!("{member}({params}): {enum_name}"))
            }
            HandleKind::Text => {
                abstract_fun(w, format!("Render the `{id}` text."), format!("{member}({params}): Component"))
            }
            HandleKind::Sprite => {
                let ty = self.sprite_enum.as_deref().unwrap_or("WindowSprite");
                abstract_fun(w, format!("Render the `{id}` sprite."), format!("{member}({params}): {ty}?"))
            }
            HandleKind::Items => {
                abstract_fun(w, format!("Render the `{id}` item."), format!("{member}({params}): {item}?"))
            }
            HandleKind::Action if self.handle.shape.is_empty() => {
                abstract_fun(w, format!("Handle a `{id}` click."), format!("{member}(click: Click)"))
            }
            HandleKind::Action => abstract_fun(
                w,
                format!("Handle a `{id}` click; `click.index` is the clicked entry, row-major."),
                format!("{member}(click: IndexedClick)"),
            ),
            HandleKind::Input => abstract_fun(
                w,
                format!("Handle a value change from the native `{id}` anvil input."),
                format!("{member}(value: String)"),
            ),
            HandleKind::Collection => {
                w.doc(format_args!("The cells of the `{id}` collection."));
                w.line(format_args!("protected abstract val {member}: WindowCollection<{item}>"));
                w.blank();
            }
            HandleKind::Toggle | HandleKind::Selection => {
                let (kind, ty, initial) = match self.handle.kind {
                    HandleKind::Toggle => ("toggle", "Boolean", self.initial("false")),
                    _ => ("selection", enum_name, format!("{enum_name}.{}", self.initial("").to_uppercase())),
                };
                let hook = self.hook.as_deref().unwrap_or_default();
                w.doc(format_args!("The `{id}` {kind}. Clicks change it, then call [{hook}]."));
                w.line(format_args!("protected var {member}: {ty} by state({initial})"));
                w.blank();
                w.doc(format_args!("Called after a click changes `{id}`; assigning it does not call this."));
                w.line(format_args!("protected open fun {hook}(value: {ty}) {{}}"));
                w.blank();
            }
            HandleKind::Builtin => {}
        }
    }

    fn initial(&self, default: &str) -> String {
        self.handle.initial.clone().unwrap_or_else(|| default.to_string())
    }

    /// A Kotlin expression reading the handle at `args`.
    fn read(&self, args: &str) -> String {
        match self.handle.kind {
            HandleKind::Toggle | HandleKind::Selection | HandleKind::Collection => self.member.clone(),
            _ => format!("{}({args})", self.member),
        }
    }

    /// The Kotlin constant naming `value`.
    fn constant(&self, value: &str) -> String {
        format!("{}.{}", self.enum_name.as_deref().unwrap_or_default(), value.to_uppercase())
    }

    /// A Boolean expression for a condition use: the flag or toggle, or a comparison with `value`.
    fn condition(&self, args: &str, value: Option<&str>) -> String {
        match value {
            Some(value) => format!("{} == {}", self.read(args), self.constant(value)),
            None => self.read(args),
        }
    }
}

/// Writes the companion object holding every handle's shape constants.
pub(super) fn constants(w: &mut KotlinWriter, handles: &[HandleMember]) {
    if handles.iter().all(|handle| handle.constants.is_empty()) {
        return;
    }
    w.blank();
    w.open("public companion object {");
    for handle in handles {
        for (constant, value) in &handle.constants {
            match handle.handle.kind {
                HandleKind::Collection => w.doc(format_args!("The number of cells of `{}`.", handle.id)),
                _ => w.doc(format_args!("A shape dimension of `{}`.", handle.id)),
            }
            w.line(format_args!("public const val {constant}: Int = {value}"));
        }
    }
    w.close("}");
}

/// Binds every use of `handles`; an indexed handle whose uses cover every index binds them in a loop.
pub(super) fn bind(w: &mut KotlinWriter, handles: &[HandleMember]) {
    for handle in handles {
        let mut groups: BTreeMap<(HandleRole, Option<&str>), Vec<&HandleUse>> = BTreeMap::new();
        for use_ in &handle.handle.uses {
            if handle.handle.shape.is_empty() {
                bind_use(w, handle, use_, "", &kt_string(&use_.entry));
            } else {
                groups.entry((use_.role, use_.value.as_deref())).or_default().push(use_);
            }
        }
        for ((_, value), mut uses) in groups {
            uses.sort_by(|a, b| a.at.cmp(&b.at));
            let shape = &handle.handle.shape;
            let suffix = match (uses[0].role, value) {
                (HandleRole::Click, Some(value)) => format!("={value}"),
                (_, Some(value)) => format!("?{value}"),
                (_, None) => String::new(),
            };
            let all = indices(shape);
            let canonical = uses.len() == all.len()
                && uses.iter().zip(&all).all(|(use_, index)| {
                    use_.at == *index && use_.entry == format!("{}{suffix}", indexed_entry(&handle.id, index))
                });
            if !canonical {
                for use_ in uses {
                    bind_use(w, handle, use_, &literal_args(&use_.at), &kt_string(&use_.entry));
                }
                continue;
            }
            let id = &handle.id;
            let (first, constants) = (&uses[0], &handle.constants);
            match shape.as_slice() {
                [_] => {
                    w.open(format_args!("for (index in 0 until {}) {{", constants[0].0));
                    bind_use(w, handle, first, "index", &format!("\"{id}[$index]{suffix}\""));
                    w.close("}");
                }
                _ => {
                    w.open(format_args!("for (row in 0 until {}) {{", constants[0].0));
                    w.open(format_args!("for (column in 0 until {}) {{", constants[1].0));
                    let entry = format!("\"{id}[$row][$column]{suffix}\"");
                    bind_use(w, handle, first, "row, column", &entry);
                    w.close("}");
                    w.close("}");
                }
            }
        }
    }
}

/// Binds one use at `args`, the Kotlin arguments of its index, to the manifest entry the expression `entry` names.
fn bind_use(w: &mut KotlinWriter, handle: &HandleMember, use_: &HandleUse, args: &str, entry: &str) {
    let member = &handle.member;
    match use_.role {
        HandleRole::Slot => w.line(format_args!("slot({entry}) {{ {member}({args}) }}")),
        HandleRole::SpriteSlot => {
            let sprite = if handle.sprite_enum.is_some() { "?.sprite" } else { "" };
            w.line(format_args!("sprite({entry}) {{ {member}({args}){sprite}?.id }}"));
        }
        HandleRole::Item => w.line(format_args!("item({entry}) {{ {member}({args}) }}")),
        HandleRole::Collection => w.line(format_args!("collection({entry}, {member})")),
        HandleRole::Input => w.line(format_args!("anvilInput({entry}, ::{member})")),
        HandleRole::Switch => {
            let value = match (handle.handle.kind, &use_.value) {
                (HandleKind::Value | HandleKind::Selection, None) => format!("{}.value", handle.read(args)),
                (_, Some(value)) => format!("({}).toString()", handle.condition(args, Some(value))),
                (_, None) => format!("{}.toString()", handle.read(args)),
            };
            w.line(format_args!("switch({entry}) {{ {value} }}"));
        }
        // A runtime action cannot be bound; the runtime runs it.
        HandleRole::Click if handle.handle.kind == HandleKind::Builtin => {}
        HandleRole::Click => Handler::of(handle, use_, args).call(w, &format!("button({entry}")),
    }
}

/// A click handler: a member reference, or a lambda with its parameters and body lines.
enum Handler {
    Reference(String),
    Body(&'static str, Vec<String>),
}

impl Handler {
    fn of(handle: &HandleMember, click: &HandleUse, args: &str) -> Self {
        let (member, hook) = (&handle.member, handle.hook.as_deref().unwrap_or_default());
        match handle.handle.kind {
            HandleKind::Action if handle.handle.shape.is_empty() => Self::Reference(format!("::{member}")),
            HandleKind::Action => {
                let index = match (handle.handle.shape.as_slice(), args) {
                    ([_, columns], "row, column") => format!("row * {columns} + column"),
                    ([_, columns], _) => (click.at[0] * columns + click.at[1]).to_string(),
                    _ => args.to_string(),
                };
                Self::Body(
                    "click -> ",
                    vec![format!("{member}(IndexedClick(click.slot, {index}, click.shift, click.right))")],
                )
            }
            HandleKind::Toggle => {
                Self::Body("_ -> ", vec![format!("{member} = !{member}"), format!("{hook}({member})")])
            }
            HandleKind::Selection => {
                let value = handle.constant(click.value.as_deref().unwrap_or_default());
                Self::Body(
                    "_ -> ",
                    vec![
                        format!("if ({member} != {value}) {{"),
                        format!("    {member} = {value}"),
                        format!("    {hook}({member})"),
                        "}".to_string(),
                    ],
                )
            }
            _ => unreachable!("only action, toggle, and selection handles take clicks"),
        }
    }

    /// Writes `call` (an unclosed argument list) completed with this handler.
    fn call(self, w: &mut KotlinWriter, call: &str) {
        match self {
            Self::Reference(reference) => w.line(format_args!("{call}, {reference})")),
            Self::Body(params, lines) => {
                w.open(format!("{call}) {{ {params}").trim_end());
                for line in lines {
                    w.line(line);
                }
                w.close("}");
            }
        }
    }
}

/// The parameter list of an indexed member: none, `index`, or `row` and `column`.
pub(super) fn index_params(shape: &[u32]) -> &'static str {
    match shape.len() {
        0 => "",
        1 => "index: Int",
        _ => "row: Int, column: Int",
    }
}

fn literal_args(at: &[u32]) -> String {
    at.iter().map(u32::to_string).collect::<Vec<_>>().join(", ")
}

/// Every index of `shape`, row-major.
fn indices(shape: &[u32]) -> Vec<Vec<u32>> {
    match shape {
        [len] => (0..*len).map(|i| vec![i]).collect(),
        [rows, columns] => (0..*rows).flat_map(|i| (0..*columns).map(move |j| vec![i, j])).collect(),
        _ => Vec::new(),
    }
}
