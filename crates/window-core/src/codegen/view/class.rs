use std::fmt::Display;

use crate::codegen::writer::KotlinWriter;

use super::members::{Member, ValueKind};

/// The runtime base a typed view extends: `WindowView` or `HudView`.
pub(super) struct ViewBase {
    /// Prefix of the runtime `View` and `Scope` classes.
    pub(super) prefix: &'static str,
    /// Noun used in the class documentation.
    pub(super) noun: &'static str,
}

pub(super) fn render(package_name: &str, base: &ViewBase, name: &str, class_name: &str, members: &[Member]) -> String {
    let prefix = base.prefix;
    let mut w = KotlinWriter::file(package_name, imports(prefix, members));
    w.doc(format_args!("Typed view for the `{name}` {}. Implement the abstract members.", base.noun));
    let keyword = if members.is_empty() { "open" } else { "abstract" };
    w.open(format_args!("public {keyword} class {class_name} : {prefix}View(\"{name}\") {{"));
    for member in members {
        declare(&mut w, member);
    }
    if members.is_empty() {
        w.line(format_args!("final override fun {prefix}Scope.bind() {{}}"));
    } else {
        w.open(format_args!("final override fun {prefix}Scope.bind() {{"));
        for member in members {
            bind(&mut w, member);
        }
        w.close("}");
    }
    w.close("}");
    w.finish()
}

fn imports(prefix: &str, members: &[Member]) -> Vec<String> {
    let any = |test: fn(&Member) -> bool| members.iter().any(test);
    let has_value = |kind: ValueKind| {
        members
            .iter()
            .any(|m| matches!(m, Member::Value { kind: k, .. } | Member::GroupValue { kind: k, .. } if *k == kind))
    };
    [
        (any(|m| matches!(m, Member::Button { .. })), "dev.oglass.window.Click".to_string()),
        (
            any(|m| matches!(m, Member::Collection { handler: Some(_), .. } | Member::GroupButton { .. })),
            "dev.oglass.window.IndexedClick".to_string(),
        ),
        (true, format!("dev.oglass.window.{prefix}Scope")),
        (true, format!("dev.oglass.window.{prefix}View")),
        (has_value(ValueKind::Slot), "net.kyori.adventure.text.Component".to_string()),
        (
            has_value(ValueKind::Item) || any(|m| matches!(m, Member::Collection { .. })),
            "net.minestom.server.item.ItemStack".to_string(),
        ),
    ]
    .into_iter()
    .filter_map(|(used, import)| used.then_some(import))
    .collect()
}

fn declare(w: &mut KotlinWriter, member: &Member) {
    match member {
        Member::Value { kind, source, member } => abstract_fun(
            w,
            format_args!("Render the `{source}` {}.", kind.noun()),
            format_args!("{member}(): {}", kind.return_type()),
        ),
        Member::GroupValue { kind, group, field, member, .. } => abstract_fun(
            w,
            format_args!("Render one `{field}` {} in the `{group}` repeater.", kind.noun()),
            format_args!("{member}(index: Int): {}", kind.return_type()),
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
        Member::Collection { source, item_member, handler } => {
            abstract_fun(
                w,
                format_args!("Render one cell in the `{source}` collection."),
                format_args!("{item_member}(index: Int): ItemStack?"),
            );
            if let Some(handler) = handler {
                abstract_fun(
                    w,
                    format_args!("Handle a click on the `{source}` collection."),
                    format_args!("{handler}(click: IndexedClick)"),
                );
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
                w.line(format_args!("IndexedClick(click.player, click.slot, {index}, click.shift, click.right)"));
                w.close(")");
                w.close("}");
            }
        }
        Member::Button { source, member, .. } => w.line(format_args!("button(\"{source}\", ::{member})")),
        Member::Collection { source, item_member, handler: Some(handler) } => {
            w.line(format_args!("collection(\"{source}\", ::{item_member}, ::{handler})"))
        }
        Member::Collection { source, item_member, handler: None } => {
            w.line(format_args!("collectionItem(\"{source}\", ::{item_member})"))
        }
        Member::AnvilInput { source, member } => w.line(format_args!("anvilInput(\"{source}\", ::{member})")),
    }
}
