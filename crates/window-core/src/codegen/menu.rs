use std::collections::{BTreeMap, BTreeSet};

use crate::Result;
use crate::ir::ButtonDefault;
use crate::manifest::{Manifest, WindowEntry};
use crate::pipeline::OutputFile;

use super::views::{
    anvil_input_map_expr, button_map_expr, collection_map_expr, item_map_expr, kt_string, repeat_group_map_expr,
    slot_map_expr, slot_rect_map_expr, sprite_slot_map_expr, surface_entry_expr,
};
use super::{HEADER, INDENT, INDENT2, naming};

pub(super) fn generate_window_definitions(manifest: &Manifest, package_name: &str) -> OutputFile {
    OutputFile {
        path: "WindowDefinitions.kt".into(),
        contents: render_window_definitions(manifest, package_name).into_bytes(),
    }
}

fn render_window_definitions(manifest: &Manifest, package_name: &str) -> String {
    let has_slots = manifest.windows.values().any(|w| !w.slots.is_empty());
    let has_sprite_slots = manifest.windows.values().any(|w| !w.sprite_slots.is_empty());
    let has_buttons = manifest.windows.values().any(|w| !w.buttons.is_empty());
    let has_button_default = manifest.windows.values().any(|w| w.buttons.values().any(|b| b.default.is_some()));
    let has_button_state = manifest.windows.values().any(|w| w.buttons.values().any(|b| !b.states.is_empty()));
    let has_tooltip = manifest.windows.values().any(|w| {
        w.buttons.values().any(|b| b.tooltip.is_some() || b.states.values().any(|state| state.tooltip.is_some()))
    });
    let has_items = manifest.windows.values().any(|w| !w.items.is_empty());
    let has_collections = manifest.windows.values().any(|w| !w.collections.is_empty());
    let has_inputs = manifest.windows.values().any(|w| !w.inputs.is_empty());
    let has_slot_rects = manifest.windows.values().any(|w| !w.slot_rects.is_empty());
    let has_groups = manifest.windows.values().any(|w| !w.groups.is_empty());
    let has_slot_refs = manifest.windows.values().any(|w| {
        w.buttons.values().any(|b| !b.slots.is_empty())
            || w.items.values().any(|item| !item.slots.is_empty())
            || w.collections.values().any(|collection| !collection.slots.is_empty())
            || w.slot_rects.values().any(|slot_rect| !slot_rect.slots.is_empty())
            || !w.inputs.is_empty()
    });

    let mut out = String::new();
    out.push_str(HEADER);
    out.push('\n');
    out.push_str("package ");
    out.push_str(package_name);
    out.push_str("\n\n");
    if has_slots || has_sprite_slots {
        out.push_str("import dev.oglass.window.manifest.Align\n");
    }
    if has_button_default {
        out.push_str("import dev.oglass.window.manifest.ButtonDefault\n");
    }
    if has_buttons {
        out.push_str("import dev.oglass.window.manifest.ButtonEntry\n");
    }
    if has_button_state {
        out.push_str("import dev.oglass.window.manifest.ButtonState\n");
    }
    if has_tooltip {
        out.push_str("import dev.oglass.window.manifest.ButtonTooltip\n");
    }
    if has_collections {
        out.push_str("import dev.oglass.window.manifest.CollectionEntry\n");
    }
    if has_inputs {
        out.push_str("import dev.oglass.window.manifest.AnvilInputEntry\n");
    }
    if has_items {
        out.push_str("import dev.oglass.window.manifest.ItemEntry\n");
    }
    if has_groups {
        out.push_str("import dev.oglass.window.manifest.RepeatGroupEntry\n");
    }
    if has_slot_refs {
        out.push_str("import dev.oglass.window.manifest.SlotAreaEntry\n");
    }
    if has_slots {
        out.push_str("import dev.oglass.window.manifest.SlotEntry\n");
    }
    if has_slot_rects {
        out.push_str("import dev.oglass.window.manifest.SlotRectEntry\n");
    }
    if has_slot_refs {
        out.push_str("import dev.oglass.window.manifest.SlotRefEntry\n");
    }
    if has_sprite_slots {
        out.push_str("import dev.oglass.window.manifest.SpriteSlotEntry\n");
    }
    out.push_str("import dev.oglass.window.manifest.SurfaceEntry\n");
    out.push_str("import dev.oglass.window.manifest.WindowEntry\n\n");
    out.push_str("/** Generated window definitions for this Window pack. */\n");
    out.push_str("public object WindowDefinitions {\n");
    out.push_str(INDENT);
    out.push_str("val all: Map<String, WindowEntry> =\n");
    if manifest.windows.is_empty() {
        out.push_str(INDENT2);
        out.push_str("emptyMap()\n");
    } else {
        out.push_str(INDENT2);
        out.push_str("mapOf(\n");
        for (name, window) in &manifest.windows {
            out.push_str(INDENT2);
            out.push_str(INDENT);
            out.push_str(&kt_string(name));
            out.push_str(" to\n");
            out.push_str(&super::views::indent(4));
            out.push_str(&window_entry_expr(window, 4));
            out.push_str(",\n");
        }
        out.push_str(INDENT2);
        out.push_str(")\n");
    }
    out.push_str("}\n");
    out
}

fn window_entry_expr(window: &WindowEntry, level: usize) -> String {
    let i = super::views::indent(level);
    let ii = super::views::indent(level + 1);
    let mut out = String::new();
    out.push_str("WindowEntry(\n");
    out.push_str(&ii);
    out.push_str("surface = ");
    out.push_str(&surface_entry_expr(&window.surface, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("static = ");
    out.push_str(&kt_string(&window.static_text));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("slots = ");
    out.push_str(&slot_map_expr(&window.slots, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("spriteSlots = ");
    out.push_str(&sprite_slot_map_expr(&window.sprite_slots, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("buttons = ");
    out.push_str(&button_map_expr(&window.buttons, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("items = ");
    out.push_str(&item_map_expr(&window.items, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("collections = ");
    out.push_str(&collection_map_expr(&window.collections, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("inputs = ");
    out.push_str(&anvil_input_map_expr(&window.inputs, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("slotRects = ");
    out.push_str(&slot_rect_map_expr(&window.slot_rects, level + 1));
    out.push_str(",\n");
    out.push_str(&ii);
    out.push_str("groups = ");
    out.push_str(&repeat_group_map_expr(&window.groups, level + 1));
    out.push_str(",\n");
    out.push_str(&i);
    out.push(')');
    out
}

pub(super) fn generate_window(name: &str, window: &WindowEntry, package_name: &str) -> Result<OutputFile> {
    let class_name = naming::class_name(name);
    let mut taken: BTreeMap<String, String> = BTreeMap::new();
    let mut members = Vec::new();
    let mut grouped_slots = BTreeSet::new();
    let mut grouped_sprite_slots = BTreeSet::new();
    let mut grouped_items = BTreeSet::new();
    let mut grouped_buttons = BTreeSet::new();

    for (group_name, group) in &window.groups {
        for (field, sources) in &group.slots {
            let sources: Vec<String> = sources.iter().filter(|s| !s.is_empty()).cloned().collect();
            if sources.is_empty() {
                continue;
            }
            let member = naming::slot_member(&format!("{group_name}_{field}"));
            naming::check_member(&member, &format!("repeater `{group_name}` slot `{field}`"), &class_name, &mut taken)?;
            for source in &sources {
                grouped_slots.insert(source.clone());
            }
            members.push(Member::GroupSlot { group: group_name.clone(), field: field.clone(), sources, member });
        }
        for (field, sources) in &group.sprite_slots {
            let sources: Vec<String> = sources.iter().filter(|s| !s.is_empty()).cloned().collect();
            if sources.is_empty() {
                continue;
            }
            let member = format!("{}Sprite", naming::slot_member(&format!("{group_name}_{field}")));
            naming::check_member(
                &member,
                &format!("repeater `{group_name}` sprite slot `{field}`"),
                &class_name,
                &mut taken,
            )?;
            for source in &sources {
                grouped_sprite_slots.insert(source.clone());
            }
            members.push(Member::GroupSprite { group: group_name.clone(), field: field.clone(), sources, member });
        }
        for (field, sources) in &group.items {
            let sources: Vec<String> = sources.iter().filter(|s| !s.is_empty()).cloned().collect();
            if sources.is_empty() {
                continue;
            }
            let member = format!("{}Item", naming::slot_member(&format!("{group_name}_{field}")));
            naming::check_member(&member, &format!("repeater `{group_name}` item `{field}`"), &class_name, &mut taken)?;
            for source in &sources {
                grouped_items.insert(source.clone());
            }
            members.push(Member::GroupItem { group: group_name.clone(), field: field.clone(), sources, member });
        }
        if !group.buttons.is_empty() {
            let sources: Vec<String> = group.buttons.iter().filter(|s| !s.is_empty()).cloned().collect();
            if sources.is_empty() {
                continue;
            }
            let member = naming::button_member(group_name);
            naming::check_member(&member, &format!("repeater `{group_name}` button"), &class_name, &mut taken)?;
            for source in &sources {
                grouped_buttons.insert(source.clone());
            }
            members.push(Member::GroupButton { group: group_name.clone(), sources, member });
        }
    }

    for slot in window.slots.keys().filter(|slot| window.slots[*slot].text.is_none() && !grouped_slots.contains(*slot))
    {
        let member = naming::slot_member(slot);
        naming::check_member(&member, &format!("slot `{slot}`"), &class_name, &mut taken)?;
        members.push(Member::Slot { source: slot.clone(), member });
    }

    for slot in window
        .sprite_slots
        .keys()
        .filter(|slot| window.sprite_slots[*slot].sprite.is_none() && !grouped_sprite_slots.contains(*slot))
    {
        let member = format!("{}Sprite", naming::slot_member(slot));
        naming::check_member(&member, &format!("sprite slot `{slot}`"), &class_name, &mut taken)?;
        members.push(Member::Sprite { source: slot.clone(), member });
    }

    for (button, entry) in
        window.buttons.iter().filter(|(button, entry)| entry.action && !grouped_buttons.contains(*button))
    {
        let member = naming::button_member(button);
        naming::check_member(&member, &format!("button `{button}`"), &class_name, &mut taken)?;
        members.push(Member::Button {
            source: button.clone(),
            member,
            is_close: entry.default == Some(ButtonDefault::Close),
        });
    }

    for item in window.items.keys().filter(|item| !grouped_items.contains(*item)) {
        let member = format!("{}Item", naming::slot_member(item));
        naming::check_member(&member, &format!("item `{item}`"), &class_name, &mut taken)?;
        members.push(Member::Item { source: item.clone(), member });
    }

    for (collection, entry) in &window.collections {
        let item_member = format!("{}Item", naming::slot_member(collection));
        naming::check_member(&item_member, &format!("collection `{collection}` item"), &class_name, &mut taken)?;
        let handler = if entry.action {
            let member = naming::button_member(collection);
            naming::check_member(&member, &format!("collection `{collection}`"), &class_name, &mut taken)?;
            Some(member)
        } else {
            None
        };
        members.push(Member::Collection { source: collection.clone(), item_member, handler });
    }

    for input in window.inputs.keys() {
        let member = format!("{}Changed", naming::button_member(input));
        naming::check_member(&member, &format!("anvil input `{input}`"), &class_name, &mut taken)?;
        members.push(Member::AnvilInput { source: input.clone(), member });
    }

    Ok(OutputFile {
        path: format!("{class_name}.kt"),
        contents: render(package_name, name, &class_name, &members).into_bytes(),
    })
}

fn render(package_name: &str, window_name: &str, class_name: &str, members: &[Member]) -> String {
    let has_button = members.iter().any(|m| matches!(m, Member::Button { .. }));
    let has_slot = members.iter().any(|m| matches!(m, Member::Slot { .. } | Member::GroupSlot { .. }));
    let has_item =
        members.iter().any(|m| matches!(m, Member::Item { .. } | Member::GroupItem { .. } | Member::Collection { .. }));
    let has_indexed_click =
        members.iter().any(|m| matches!(m, Member::Collection { handler: Some(_), .. } | Member::GroupButton { .. }));
    let class_keyword = if members.is_empty() { "open" } else { "abstract" };

    let mut out = String::new();
    out.push_str(HEADER);
    out.push('\n');
    out.push_str("package ");
    out.push_str(package_name);
    out.push_str("\n\n");
    if has_button {
        out.push_str("import dev.oglass.window.Click\n");
    }
    if has_indexed_click {
        out.push_str("import dev.oglass.window.IndexedClick\n");
    }
    out.push_str("import dev.oglass.window.WindowScope\n");
    out.push_str("import dev.oglass.window.WindowView\n");
    if has_slot {
        out.push_str("import net.kyori.adventure.text.Component\n");
    }
    if has_item {
        out.push_str("import net.minestom.server.item.ItemStack\n");
    }
    out.push('\n');
    out.push_str("/** Typed view for the `");
    out.push_str(window_name);
    out.push_str("` window. Implement the abstract members. */\n");
    out.push_str("public ");
    out.push_str(class_keyword);
    out.push_str(" class ");
    out.push_str(class_name);
    out.push_str(" : WindowView(\"");
    out.push_str(window_name);
    out.push_str("\") {\n");

    for member in members {
        match member {
            Member::Slot { source, member } => {
                out.push_str(INDENT);
                out.push_str("/** Render the `");
                out.push_str(source);
                out.push_str("` slot. */\n");
                out.push_str(INDENT);
                out.push_str("protected abstract fun ");
                out.push_str(member);
                out.push_str("(): Component\n\n");
            }
            Member::Sprite { source, member } => {
                out.push_str(INDENT);
                out.push_str("/** Render the `");
                out.push_str(source);
                out.push_str("` runtime sprite id. */\n");
                out.push_str(INDENT);
                out.push_str("protected abstract fun ");
                out.push_str(member);
                out.push_str("(): String?\n\n");
            }
            Member::GroupSlot { group, field, member, .. } => {
                out.push_str(INDENT);
                out.push_str("/** Render one `");
                out.push_str(field);
                out.push_str("` slot in the `");
                out.push_str(group);
                out.push_str("` repeater. */\n");
                out.push_str(INDENT);
                out.push_str("protected abstract fun ");
                out.push_str(member);
                out.push_str("(index: Int): Component\n\n");
            }
            Member::GroupSprite { group, field, member, .. } => {
                out.push_str(INDENT);
                out.push_str("/** Render one `");
                out.push_str(field);
                out.push_str("` runtime sprite id in the `");
                out.push_str(group);
                out.push_str("` repeater. */\n");
                out.push_str(INDENT);
                out.push_str("protected abstract fun ");
                out.push_str(member);
                out.push_str("(index: Int): String?\n\n");
            }
            Member::GroupItem { group, field, member, .. } => {
                out.push_str(INDENT);
                out.push_str("/** Render one `");
                out.push_str(field);
                out.push_str("` inventory item in the `");
                out.push_str(group);
                out.push_str("` repeater. */\n");
                out.push_str(INDENT);
                out.push_str("protected abstract fun ");
                out.push_str(member);
                out.push_str("(index: Int): ItemStack?\n\n");
            }
            Member::GroupButton { group, member, .. } => {
                out.push_str(INDENT);
                out.push_str("/** Handle a click on the `");
                out.push_str(group);
                out.push_str("` repeater. */\n");
                out.push_str(INDENT);
                out.push_str("protected abstract fun ");
                out.push_str(member);
                out.push_str("(click: IndexedClick)\n\n");
            }
            Member::Button { source, member, is_close } => {
                out.push_str(INDENT);
                out.push_str("/** Handle a click on the `");
                out.push_str(source);
                if *is_close {
                    out.push_str("` button (default: close the window). */\n");
                    out.push_str(INDENT);
                    out.push_str("protected open fun ");
                    out.push_str(member);
                    out.push_str("(click: Click): Unit = close()\n\n");
                } else {
                    out.push_str("` button. */\n");
                    out.push_str(INDENT);
                    out.push_str("protected abstract fun ");
                    out.push_str(member);
                    out.push_str("(click: Click)\n\n");
                }
            }
            Member::AnvilInput { source, member } => {
                out.push_str(INDENT);
                out.push_str("/** Handle a value change from the native `");
                out.push_str(source);
                out.push_str("` anvil input. */\n");
                out.push_str(INDENT);
                out.push_str("protected abstract fun ");
                out.push_str(member);
                out.push_str("(value: String)\n\n");
            }
            Member::Item { source, member } => {
                out.push_str(INDENT);
                out.push_str("/** Render the `");
                out.push_str(source);
                out.push_str("` inventory item. */\n");
                out.push_str(INDENT);
                out.push_str("protected abstract fun ");
                out.push_str(member);
                out.push_str("(): ItemStack?\n\n");
            }
            Member::Collection { source, item_member, handler } => {
                out.push_str(INDENT);
                out.push_str("/** Render one cell in the `");
                out.push_str(source);
                out.push_str("` collection. */\n");
                out.push_str(INDENT);
                out.push_str("protected abstract fun ");
                out.push_str(item_member);
                out.push_str("(index: Int): ItemStack?\n\n");
                if let Some(handler) = handler {
                    out.push_str(INDENT);
                    out.push_str("/** Handle a click on the `");
                    out.push_str(source);
                    out.push_str("` collection. */\n");
                    out.push_str(INDENT);
                    out.push_str("protected abstract fun ");
                    out.push_str(handler);
                    out.push_str("(click: IndexedClick)\n\n");
                }
            }
        }
    }

    if members.is_empty() {
        out.push_str(INDENT);
        out.push_str("final override fun WindowScope.bind() {}\n");
    } else {
        out.push_str(INDENT);
        out.push_str("final override fun WindowScope.bind() {\n");
        for member in members {
            match member {
                Member::Slot { source, member } => {
                    out.push_str(INDENT2);
                    out.push_str("slot(\"");
                    out.push_str(source);
                    out.push_str("\") { ");
                    out.push_str(member);
                    out.push_str("() }\n");
                }
                Member::Sprite { source, member } => {
                    out.push_str(INDENT2);
                    out.push_str("sprite(\"");
                    out.push_str(source);
                    out.push_str("\") { ");
                    out.push_str(member);
                    out.push_str("() }\n");
                }
                Member::GroupSlot { sources, member, .. } => {
                    for (index, source) in sources.iter().enumerate() {
                        out.push_str(INDENT2);
                        out.push_str("slot(\"");
                        out.push_str(source);
                        out.push_str("\") { ");
                        out.push_str(member);
                        out.push('(');
                        out.push_str(&index.to_string());
                        out.push_str(") }\n");
                    }
                }
                Member::GroupSprite { sources, member, .. } => {
                    for (index, source) in sources.iter().enumerate() {
                        out.push_str(INDENT2);
                        out.push_str("sprite(\"");
                        out.push_str(source);
                        out.push_str("\") { ");
                        out.push_str(member);
                        out.push('(');
                        out.push_str(&index.to_string());
                        out.push_str(") }\n");
                    }
                }
                Member::GroupItem { sources, member, .. } => {
                    for (index, source) in sources.iter().enumerate() {
                        out.push_str(INDENT2);
                        out.push_str("item(\"");
                        out.push_str(source);
                        out.push_str("\") { ");
                        out.push_str(member);
                        out.push('(');
                        out.push_str(&index.to_string());
                        out.push_str(") }\n");
                    }
                }
                Member::GroupButton { sources, member, .. } => {
                    for (index, source) in sources.iter().enumerate() {
                        out.push_str(INDENT2);
                        out.push_str("button(\"");
                        out.push_str(source);
                        out.push_str("\") { click ->\n");
                        out.push_str(INDENT2);
                        out.push_str(INDENT);
                        out.push_str(member);
                        out.push_str("(\n");
                        out.push_str(INDENT2);
                        out.push_str(INDENT2);
                        out.push_str("IndexedClick(click.player, click.slot, ");
                        out.push_str(&index.to_string());
                        out.push_str(", click.shift, click.right)\n");
                        out.push_str(INDENT2);
                        out.push_str(INDENT);
                        out.push_str(")\n");
                        out.push_str(INDENT2);
                        out.push_str("}\n");
                    }
                }
                Member::Button { source, member, .. } => {
                    out.push_str(INDENT2);
                    out.push_str("button(\"");
                    out.push_str(source);
                    out.push_str("\", ::");
                    out.push_str(member);
                    out.push_str(")\n");
                }
                Member::Item { source, member } => {
                    out.push_str(INDENT2);
                    out.push_str("item(\"");
                    out.push_str(source);
                    out.push_str("\") { ");
                    out.push_str(member);
                    out.push_str("() }\n");
                }
                Member::Collection { source, item_member, handler } => {
                    out.push_str(INDENT2);
                    if let Some(handler) = handler {
                        out.push_str("collection(\"");
                        out.push_str(source);
                        out.push_str("\", ::");
                        out.push_str(item_member);
                        out.push_str(", ::");
                        out.push_str(handler);
                        out.push_str(")\n");
                    } else {
                        out.push_str("collectionItem(\"");
                        out.push_str(source);
                        out.push_str("\", ::");
                        out.push_str(item_member);
                        out.push_str(")\n");
                    }
                }
                Member::AnvilInput { source, member } => {
                    out.push_str(INDENT2);
                    out.push_str("anvilInput(\"");
                    out.push_str(source);
                    out.push_str("\", ::");
                    out.push_str(member);
                    out.push_str(")\n");
                }
            }
        }
        out.push_str(INDENT);
        out.push_str("}\n");
    }

    out.push_str("}\n");
    out
}

enum Member {
    Slot { source: String, member: String },
    Sprite { source: String, member: String },
    GroupSlot { group: String, field: String, sources: Vec<String>, member: String },
    GroupSprite { group: String, field: String, sources: Vec<String>, member: String },
    GroupItem { group: String, field: String, sources: Vec<String>, member: String },
    GroupButton { group: String, sources: Vec<String>, member: String },
    Button { source: String, member: String, is_close: bool },
    Item { source: String, member: String },
    Collection { source: String, item_member: String, handler: Option<String> },
    AnvilInput { source: String, member: String },
}
