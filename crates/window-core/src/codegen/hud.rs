use std::collections::BTreeMap;

use crate::Result;
use crate::manifest::{HudEntry, Manifest};
use crate::pipeline::OutputFile;

use super::views::{hud_surface_entry_expr, kt_string, optional_hud_shader_entry_expr, slot_map_expr};
use super::{HEADER, INDENT, INDENT2, naming};

pub(super) fn generate_hud_definitions(manifest: &Manifest, package_name: &str) -> OutputFile {
    OutputFile {
        path: "WindowHudDefinitions.kt".into(),
        contents: render_hud_definitions(manifest, package_name).into_bytes(),
    }
}

fn render_hud_definitions(manifest: &Manifest, package_name: &str) -> String {
    let mut out = String::new();
    out.push_str(HEADER);
    out.push('\n');
    out.push_str("package ");
    out.push_str(package_name);
    out.push_str("\n\n");
    out.push_str("import dev.oglass.window.manifest.Align\n");
    out.push_str("import dev.oglass.window.manifest.HudEntry\n");
    out.push_str("import dev.oglass.window.manifest.HudShaderEntry\n");
    out.push_str("import dev.oglass.window.manifest.HudSurfaceEntry\n");
    out.push_str("import dev.oglass.window.manifest.SlotEntry\n\n");
    out.push_str("/** Generated HUD definitions for this Window pack. */\n");
    out.push_str("public object WindowHudDefinitions {\n");
    out.push_str(INDENT);
    out.push_str("val all: Map<String, HudEntry> =\n");
    if manifest.huds.is_empty() {
        out.push_str(INDENT2);
        out.push_str("emptyMap()\n");
    } else {
        out.push_str(INDENT2);
        out.push_str("mapOf(\n");
        for (name, hud) in &manifest.huds {
            out.push_str(INDENT2);
            out.push_str(INDENT);
            out.push_str(&kt_string(name));
            out.push_str(" to ");
            out.push_str(&hud_entry_expr(hud, 3));
            out.push_str(",\n");
        }
        out.push_str(INDENT2);
        out.push_str(")\n");
    }
    out.push_str("}\n");
    out
}

fn hud_entry_expr(hud: &HudEntry, level: usize) -> String {
    let i = super::views::indent(level);
    let ii = super::views::indent(level + 1);
    let mut out = String::new();
    out.push_str("HudEntry(\n");
    out.push_str(&ii);
    out.push_str("surface = ");
    out.push_str(&hud_surface_entry_expr(&hud.surface, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("static = ");
    out.push_str(&kt_string(&hud.static_text));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("slots = ");
    out.push_str(&slot_map_expr(&hud.slots, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("shader = ");
    out.push_str(&optional_hud_shader_entry_expr(hud.shader.as_ref(), level + 1));
    out.push_str(",\n");
    out.push_str(&i);
    out.push(')');
    out
}

pub(super) fn generate_hud(name: &str, hud: &HudEntry, package_name: &str) -> Result<OutputFile> {
    let class_name = naming::hud_class_name(name);
    let mut taken: BTreeMap<String, String> = BTreeMap::new();
    let mut members = Vec::new();

    for slot in hud.slots.keys().filter(|slot| hud.slots[*slot].text.is_none()) {
        let member = naming::slot_member(slot);
        naming::check_member(&member, &format!("slot `{slot}`"), &class_name, &mut taken)?;
        members.push(Member::Slot { source: slot.clone(), member });
    }

    Ok(OutputFile {
        path: format!("{class_name}.kt"),
        contents: render_hud(package_name, name, &class_name, &members).into_bytes(),
    })
}

fn render_hud(package_name: &str, hud_name: &str, class_name: &str, members: &[Member]) -> String {
    let has_slot = members.iter().any(|m| matches!(m, Member::Slot { .. }));
    let class_keyword = if members.is_empty() { "open" } else { "abstract" };

    let mut out = String::new();
    out.push_str(HEADER);
    out.push('\n');
    out.push_str("package ");
    out.push_str(package_name);
    out.push_str("\n\n");
    out.push_str("import dev.oglass.window.HudScope\n");
    out.push_str("import dev.oglass.window.HudView\n");
    if has_slot {
        out.push_str("import net.kyori.adventure.text.Component\n");
    }
    out.push('\n');
    out.push_str("/** Typed view for the `");
    out.push_str(hud_name);
    out.push_str("` HUD. Implement the abstract members. */\n");
    out.push_str("public ");
    out.push_str(class_keyword);
    out.push_str(" class ");
    out.push_str(class_name);
    out.push_str(" : HudView(\"");
    out.push_str(hud_name);
    out.push_str("\") {\n");

    for member in members {
        let Member::Slot { source, member } = member;
        out.push_str(INDENT);
        out.push_str("/** Render the `");
        out.push_str(source);
        out.push_str("` slot. */\n");
        out.push_str(INDENT);
        out.push_str("protected abstract fun ");
        out.push_str(member);
        out.push_str("(): Component\n\n");
    }

    if members.is_empty() {
        out.push_str(INDENT);
        out.push_str("final override fun HudScope.bind() {}\n");
    } else {
        out.push_str(INDENT);
        out.push_str("final override fun HudScope.bind() {\n");
        for member in members {
            let Member::Slot { source, member } = member;
            out.push_str(INDENT2);
            out.push_str("slot(\"");
            out.push_str(source);
            out.push_str("\") { ");
            out.push_str(member);
            out.push_str("() }\n");
        }
        out.push_str(INDENT);
        out.push_str("}\n");
    }

    out.push_str("}\n");
    out
}

enum Member {
    Slot { source: String, member: String },
}
