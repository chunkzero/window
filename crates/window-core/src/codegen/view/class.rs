use std::fmt::Display;

use crate::codegen::KotlinTarget;
use crate::codegen::literals::kt_string;
use crate::codegen::naming;
use crate::codegen::writer::KotlinWriter;

use super::members::{Member, ValueKind};

/// The runtime base a typed view extends: `WindowView` or `HudView`.
pub(super) struct ViewBase {
    /// Prefix of the runtime `View` and `Scope` classes.
    pub(super) prefix: &'static str,
    /// Noun used in the class documentation.
    pub(super) noun: &'static str,
    /// Generated object holding the typed definitions.
    pub(super) definitions: &'static str,
    /// Whether the view takes a host and the `View` and `Scope` classes take its item type argument.
    pub(super) hosted: bool,
}

pub(super) fn render(
    package_name: &str,
    base: &ViewBase,
    target: KotlinTarget,
    name: &str,
    class_name: &str,
    members: &[Member],
) -> String {
    let prefix = base.prefix;
    let item = target.item_type();
    let type_args = if base.hosted { format!("<{item}>") } else { String::new() };
    let (type_params, params, host) = match target.player_host() {
        _ if !base.hosted => ("", String::new(), String::new()),
        Some(host) => {
            let simple = host.rsplit('.').next().unwrap_or(host);
            ("", "(protected val player: Player)".to_string(), format!(", {simple}(player)"))
        }
        None => ("<I : Any>", "(host: WindowHost<I>)".to_string(), ", host".to_string()),
    };
    let definition = format!("{}.{}", base.definitions, naming::definition_member(name));

    let mut w = KotlinWriter::file(package_name, imports(base, target, members));
    w.doc(format_args!("Typed view for the `{name}` {}. Implement the abstract members.", base.noun));
    let keyword = if members.is_empty() { "open" } else { "abstract" };
    w.open(format_args!(
        "public {keyword} class {class_name}{type_params}{params} : {prefix}View{type_args}({definition}{host}) {{"
    ));
    for member in members {
        declare(&mut w, member, item);
    }
    if members.is_empty() {
        w.line(format_args!("final override fun {prefix}Scope{type_args}.bind() {{}}"));
    } else {
        w.open(format_args!("final override fun {prefix}Scope{type_args}.bind() {{"));
        for member in members {
            bind(&mut w, member);
        }
        w.close("}");
    }
    w.close("}");
    w.finish()
}

fn imports(base: &ViewBase, target: KotlinTarget, members: &[Member]) -> Vec<String> {
    let prefix = base.prefix;
    let any = |test: fn(&Member) -> bool| members.iter().any(test);
    let has_slot = any(|m| {
        matches!(m, Member::Value { kind: ValueKind::Slot, .. } | Member::GroupValue { kind: ValueKind::Slot, .. })
    });
    let host = if base.hosted { target.player_host() } else { None };
    let mut imports: Vec<String> = [
        (any(|m| matches!(m, Member::Button { .. })), "com.chunkzero.window.Click"),
        (
            any(|m| matches!(m, Member::Collection { handler: Some(_), .. } | Member::GroupButton { .. })),
            "com.chunkzero.window.IndexedClick",
        ),
        (base.hosted && host.is_none(), "com.chunkzero.window.host.WindowHost"),
        (has_slot, "net.kyori.adventure.text.Component"),
        (host.is_some(), "net.minestom.server.entity.Player"),
        (host.is_some(), "net.minestom.server.item.ItemStack"),
    ]
    .into_iter()
    .filter_map(|(used, import)| used.then_some(import.to_string()))
    .chain([format!("com.chunkzero.window.{prefix}Scope"), format!("com.chunkzero.window.{prefix}View")])
    .chain(host.map(str::to_string))
    .collect();
    imports.sort();
    imports
}

fn declare(w: &mut KotlinWriter, member: &Member, item: &str) {
    match member {
        Member::Value { kind, source, member, shape } if shape.is_empty() => abstract_fun(
            w,
            format_args!("Render the `{source}` {}.", kind.noun()),
            format_args!("{member}(): {}", kind.return_type(item)),
        ),
        Member::Value { kind, source, member, shape } => abstract_fun(
            w,
            format_args!("Render one `{source}` {} by {}.", kind.noun(), index_noun(shape)),
            format_args!("{member}({}): {}", index_params(shape), kind.return_type(item)),
        ),
        Member::GroupValue { kind, group, field, member, .. } => abstract_fun(
            w,
            format_args!("Render one `{field}` {} in the `{group}` repeater.", kind.noun()),
            format_args!("{member}(index: Int): {}", kind.return_type(item)),
        ),
        Member::GroupButton { group, member, .. } => abstract_fun(
            w,
            format_args!("Handle a click on the `{group}` repeater."),
            format_args!("{member}(click: IndexedClick)"),
        ),
        Member::Button { source, member, is_close: true } => {
            w.doc(format_args!("Handle a click on the `{source}` button (default: close the window)."));
            w.line(format_args!("protected open fun {member}(click: Click): Unit = close()"));
            w.blank();
        }
        Member::Button { source, member, is_close: false } => abstract_fun(
            w,
            format_args!("Handle a click on the `{source}` button."),
            format_args!("{member}(click: Click)"),
        ),
        Member::AnvilInput { source, member } => abstract_fun(
            w,
            format_args!("Handle a value change from the native `{source}` anvil input."),
            format_args!("{member}(value: String)"),
        ),
        Member::Switch { source, member, cases: None, shape } => abstract_fun(
            w,
            format_args!("Whether the `{source}` switch{} draws its `true` case.", indexed_suffix(shape)),
            format_args!("{member}({}): Boolean", index_params(shape)),
        ),
        Member::Switch { source, member, cases: Some(cases), shape } => {
            w.doc(format_args!("The cases of the `{source}` switch."));
            w.open(format_args!("public enum class {}(public val value: String) {{", cases.name));
            for (constant, value) in &cases.constants {
                w.line(format_args!("{constant}({}),", kt_string(value)));
            }
            w.close("}");
            w.blank();
            abstract_fun(
                w,
                format_args!("The case the `{source}` switch{} draws.", indexed_suffix(shape)),
                format_args!("{member}({}): {}", index_params(shape), cases.name),
            );
        }
        Member::Collection { source, item_member, handler, selection } => {
            abstract_fun(
                w,
                format_args!("Render one cell in the `{source}` collection."),
                format_args!("{item_member}(index: Int): {item}?"),
            );
            if let Some(handler) = handler {
                abstract_fun(
                    w,
                    format_args!("Handle a click on the `{source}` collection."),
                    format_args!("{handler}(click: IndexedClick)"),
                );
            }
            if let Some(selection) = selection {
                w.doc(format_args!("The selected cell index in the `{source}` collection, or `null` for none."));
                w.line(format_args!("protected open fun {selection}(): Int? = null"));
                w.blank();
            }
        }
    }
}

fn abstract_fun(w: &mut KotlinWriter, doc: impl Display, signature: impl Display) {
    w.doc(doc);
    w.line(format_args!("protected abstract fun {signature}"));
    w.blank();
}

fn bind(w: &mut KotlinWriter, member: &Member) {
    match member {
        Member::Value { kind, source, member, shape } => {
            let (binder, suffix) = (kind.binder(), kind.bound_suffix());
            indexed_bind(w, source, shape, |w, name, args| {
                w.line(format_args!("{binder}({name}) {{ {member}({args}){suffix} }}"))
            });
        }
        Member::GroupValue { kind, sources, member, .. } => {
            let (binder, suffix) = (kind.binder(), kind.bound_suffix());
            each_source(w, sources, |w, name, index| {
                w.line(format_args!("{binder}({name}) {{ {member}({index}){suffix} }}"))
            });
        }
        Member::GroupButton { sources, member, .. } => {
            each_source(w, sources, |w, name, index| {
                w.open(format_args!("button({name}) {{ click ->"));
                w.open(format_args!("{member}("));
                w.line(format_args!("IndexedClick(click.slot, {index}, click.shift, click.right)"));
                w.close(")");
                w.close("}");
            });
        }
        Member::Button { source, member, .. } => w.line(format_args!("button(\"{source}\", ::{member})")),
        Member::Collection { source, item_member, handler, selection } => {
            match handler {
                Some(handler) => w.line(format_args!("collection(\"{source}\", ::{item_member}, ::{handler})")),
                None => w.line(format_args!("collectionItem(\"{source}\", ::{item_member})")),
            }
            if let Some(selection) = selection {
                w.line(format_args!("collectionSelection(\"{source}\", ::{selection})"));
            }
        }
        Member::AnvilInput { source, member } => w.line(format_args!("anvilInput(\"{source}\", ::{member})")),
        Member::Switch { source, member, cases, shape } => {
            let value = if cases.is_some() { "value" } else { "toString()" };
            indexed_bind(w, source, shape, |w, name, args| {
                w.line(format_args!("switch({name}) {{ {member}({args}).{value} }}"))
            });
        }
    }
}

/// The parameter list of an indexed member: none, `index`, or `row` and `column`.
fn index_params(shape: &[u32]) -> &'static str {
    match shape.len() {
        0 => "",
        1 => "index: Int",
        _ => "row: Int, column: Int",
    }
}

fn index_noun(shape: &[u32]) -> &'static str {
    if shape.len() == 1 { "index" } else { "row and column" }
}

fn indexed_suffix(shape: &[u32]) -> &'static str {
    match shape.len() {
        0 => "",
        1 => " at `index`",
        _ => " at `row` and `column`",
    }
}

/// Binds `source`, or every entry of an indexed family in loops over its shape. `line` receives the Kotlin
/// expression naming the entry and the member arguments.
fn indexed_bind(w: &mut KotlinWriter, source: &str, shape: &[u32], line: impl Fn(&mut KotlinWriter, &str, &str)) {
    match shape {
        [] => line(w, &format!("\"{source}\""), ""),
        [len] => {
            w.open(format_args!("for (index in 0 until {len}) {{"));
            line(w, &format!("\"{source}[$index]\""), "index");
            w.close("}");
        }
        [rows, columns, ..] => {
            w.open(format_args!("for (row in 0 until {rows}) {{"));
            w.open(format_args!("for (column in 0 until {columns}) {{"));
            line(w, &format!("\"{source}[$row][$column]\""), "row, column");
            w.close("}");
            w.close("}");
        }
    }
}

/// Binds each repeater cell's `sources` entry: in one loop when they are named `<stem><index>`, otherwise one by
/// one. `line` receives the Kotlin expressions naming the entry and its index.
fn each_source(w: &mut KotlinWriter, sources: &[String], line: impl Fn(&mut KotlinWriter, &str, &str)) {
    let stem = sources.first().and_then(|first| first.strip_suffix('0'));
    match stem.filter(|stem| sources.iter().enumerate().all(|(i, source)| *source == format!("{stem}{i}"))) {
        Some(stem) => {
            w.open(format_args!("for (index in 0 until {}) {{", sources.len()));
            line(w, &format!("\"{stem}$index\""), "index");
            w.close("}");
        }
        None => {
            for (index, source) in sources.iter().enumerate() {
                line(w, &format!("\"{source}\""), &index.to_string());
            }
        }
    }
}
