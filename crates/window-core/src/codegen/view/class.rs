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
            ("", "(protected val player: Player)".to_string(), format!(", {simple}.of(player)"))
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
        Member::Value { kind, source, member } => abstract_fun(
            w,
            format_args!("Render the `{source}` {}.", kind.noun()),
            format_args!("{member}(): {}", kind.return_type(item)),
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
        Member::Switch { source, member, cases: None } => abstract_fun(
            w,
            format_args!("Whether the `{source}` switch draws its `true` case."),
            format_args!("{member}(): Boolean"),
        ),
        Member::Switch { source, member, cases: Some(cases) } => {
            w.doc(format_args!("The cases of the `{source}` switch."));
            w.open(format_args!("public enum class {}(public val value: String) {{", cases.name));
            for (constant, value) in &cases.constants {
                w.line(format_args!("{constant}({}),", kt_string(value)));
            }
            w.close("}");
            w.blank();
            abstract_fun(
                w,
                format_args!("The case the `{source}` switch draws."),
                format_args!("{member}(): {}", cases.name),
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
        Member::Value { kind, source, member } => {
            w.line(format_args!("{}(\"{source}\") {{ {member}() }}", kind.binder()))
        }
        Member::GroupValue { kind, sources, member, .. } => {
            for (index, source) in sources.iter().enumerate() {
                w.line(format_args!("{}(\"{source}\") {{ {member}({index}) }}", kind.binder()));
            }
        }
        Member::GroupButton { sources, member, .. } => {
            for (index, source) in sources.iter().enumerate() {
                w.open(format_args!("button(\"{source}\") {{ click ->"));
                w.open(format_args!("{member}("));
                w.line(format_args!("IndexedClick(click.slot, {index}, click.shift, click.right)"));
                w.close(")");
                w.close("}");
            }
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
        Member::Switch { source, member, cases } => {
            let value = if cases.is_some() { "value" } else { "toString()" };
            w.line(format_args!("switch(\"{source}\") {{ {member}().{value} }}"));
        }
    }
}
