use crate::codegen::KotlinTarget;
use crate::codegen::naming;
use crate::codegen::writer::KotlinWriter;

use super::handles::{self, HandleMember};

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
    handles: &[HandleMember],
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

    let mut w = KotlinWriter::file(package_name, imports(base, target, handles));
    w.doc(format_args!("Typed view for the `{name}` {}. Implement the abstract members.", base.noun));
    let empty = handles.iter().all(|handle| handle.member.is_empty());
    let keyword = if empty { "open" } else { "abstract" };
    w.open(format_args!(
        "public {keyword} class {class_name}{type_params}{params} : {prefix}View{type_args}({definition}{host}) {{"
    ));
    for handle in handles {
        handle.declare(&mut w, item);
    }
    if handles.iter().all(|handle| handle.handle.uses.is_empty()) {
        w.line(format_args!("final override fun {prefix}Scope{type_args}.bind() {{}}"));
    } else {
        w.open(format_args!("final override fun {prefix}Scope{type_args}.bind() {{"));
        handles::bind(&mut w, handles);
        w.close("}");
    }
    handles::constants(&mut w, handles);
    w.close("}");
    w.finish()
}

fn imports(base: &ViewBase, target: KotlinTarget, handles: &[HandleMember]) -> Vec<String> {
    let prefix = base.prefix;
    let host = if base.hosted { target.player_host() } else { None };
    let mut imports: Vec<String> = [
        (base.hosted && host.is_none(), "com.chunkzero.window.host.WindowHost"),
        (host.is_some(), "net.minestom.server.entity.Player"),
        (host.is_some(), "net.minestom.server.item.ItemStack"),
    ]
    .into_iter()
    .filter_map(|(used, import)| used.then_some(import.to_string()))
    .chain([format!("com.chunkzero.window.{prefix}Scope"), format!("com.chunkzero.window.{prefix}View")])
    .chain(host.map(str::to_string))
    .chain(HandleMember::imports(handles).into_iter().map(str::to_string))
    .collect();
    imports.sort();
    imports.dedup();
    imports
}
