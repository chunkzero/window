//! Typed handles: compiled views declare one member per handle a window uses and bind every use.

use std::collections::BTreeMap;

use serde_json::{Value, json};
use window_core::codegen::{KotlinTarget, generate_kotlin};
use window_core::pipeline::{CompileInput, compile_project_json};

fn compile(project: &Value) -> window_core::Result<BTreeMap<String, String>> {
    let input = CompileInput { namespace: "window".into(), files: BTreeMap::new() };
    let output = compile_project_json(project.to_string().as_bytes(), &input)?;
    let files = generate_kotlin(&output.manifest, "demo", KotlinTarget::Minestom)?;
    Ok(files
        .into_iter()
        .map(|file| (file.path, String::from_utf8(file.contents.as_bytes().to_vec()).unwrap()))
        .collect())
}

fn error(project: &Value) -> String {
    compile(project).expect_err("project should be rejected").to_string()
}

fn window(name: &str, container: &str, children: Value) -> Value {
    json!({ "name": name, "container": container, "children": children })
}

fn project(windows: Value) -> Value {
    json!({ "windows": windows })
}

fn slots(x: u32, width: u32) -> Value {
    json!({ "kind": "rect", "section": "container", "x": x, "y": 0, "width": width, "height": 1 })
}

/// A region in the first container row at `column` that runs `click`.
fn region(click: Value, column: u32) -> Value {
    json!({ "type": "region", "on_click": click, "layout": { "column": column + 1, "row": 1 } })
}

/// The container section holding `children`.
fn section(children: Value) -> Value {
    json!({ "type": "section", "section": "container", "claim": "none", "children": children })
}

fn assert_lines(source: &str, lines: &[&str]) {
    for line in lines {
        assert!(source.contains(line), "missing `{line}` in:\n{source}");
    }
}

#[test]
fn every_handle_kind_declares_its_member_and_binds_each_use() {
    let category = json!({ "kind": "selection", "id": "category", "values": ["all", "gear"], "initial": "gear" });
    let mode = json!({ "kind": "value", "id": "mode", "values": ["buy", "sell"] });
    let favorites = json!({ "kind": "toggle", "id": "favorites", "initial": true });
    let can_buy = json!({ "kind": "flag", "id": "can_buy" });
    let with = |field: &str, value: &str| {
        let mut handle = category.clone();
        handle[field] = json!(value);
        handle
    };
    let shop = window(
        "shop",
        "generic_9x3",
        json!([
            { "type": "slot", "handle": { "kind": "text", "id": "title" }, "x": 8, "y": 60, "width": 60 },
            { "type": "sprite_slot", "handle": { "kind": "sprite", "id": "icon" }, "x": 80, "y": 60, "width": 8, "height": 8 },
            { "type": "switch", "handle": mode.clone(), "x": 100, "y": 60, "children": [
                { "type": "case", "value": "buy", "children": [] },
                { "type": "case", "value": "sell", "children": [] }
            ] },
            { "type": "switch", "handle": with("is", "gear"), "x": 120, "y": 60, "children": [
                { "type": "case", "value": "true", "children": [] },
                { "type": "case", "value": "false", "children": [] }
            ] },
            { "type": "switch", "handle": can_buy, "x": 140, "y": 60, "children": [
                { "type": "case", "value": "true", "children": [] },
                { "type": "case", "value": "false", "children": [] }
            ] },
            section(json!([
                region(json!({ "kind": "action", "id": "buy" }), 0),
                region(favorites, 1),
                region(with("set", "all"), 2),
                region(with("set", "gear"), 3),
                region(json!({ "kind": "builtin", "id": "window:close" }), 5),
            ])),
            { "type": "item", "handle": { "kind": "items", "id": "stack" }, "pattern": slots(6, 1) },
            { "type": "collection", "handle": { "kind": "collection", "id": "products", "selectable": true },
              "selected_sprite": { "art": "shape", "kind": "slot", "width": 18, "height": 18 },
              "pattern": { "kind": "rect", "section": "container", "x": 0, "y": 1, "width": 9, "height": 1 } }
        ]),
    );
    let search =
        window("search", "anvil", json!([{ "type": "anvil_input", "handle": { "kind": "input", "id": "query" } }]));
    let files = compile(&project(json!([shop, search]))).unwrap();
    let shop = &files["ShopView.kt"];
    assert_lines(
        shop,
        &[
            "import com.chunkzero.window.WindowCollection",
            "protected abstract fun title(): Component",
            "protected abstract fun iconSprite(): WindowSprite?",
            "protected abstract fun stack(): ItemStack?",
            "protected abstract fun canBuy(): Boolean",
            "public enum class Mode(public val value: String) {",
            "protected abstract fun mode(): Mode",
            "protected var category: Category by state(Category.GEAR)",
            "protected open fun onCategoryChanged(value: Category) {}",
            "protected var favorites: Boolean by state(true)",
            "protected open fun onFavoritesChanged(value: Boolean) {}",
            "protected abstract val products: WindowCollection<ItemStack>",
            "protected abstract fun onBuy(click: Click)",
            "slot(\"title\") { title() }",
            "sprite(\"icon\") { iconSprite()?.id }",
            "item(\"stack\") { stack() }",
            "switch(\"mode\") { mode().value }",
            "switch(\"category?gear\") { (category == Category.GEAR).toString() }",
            "switch(\"can_buy\") { canBuy().toString() }",
            "button(\"buy\", ::onBuy)",
            "button(\"favorites\") { _ ->",
            "    favorites = !favorites",
            "button(\"category=all\") { _ ->",
            "    if (category != Category.ALL) {",
            "button(\"category=gear\") { _ ->",
            "collection(\"products\", products)",
            "public const val PRODUCTS_SIZE: Int = 9",
        ],
    );
    assert!(!shop.contains("window:close"), "a runtime action binds nothing:\n{shop}");
    assert_lines(
        &files["SearchView.kt"],
        &["protected abstract fun onQueryChanged(value: String)", "anvilInput(\"query\", ::onQueryChanged)"],
    );
}

#[test]
fn indexed_handles_bind_in_loops_with_shape_constants() {
    let cell = |i: u32| {
        json!([
            { "type": "item", "handle": { "kind": "items", "id": "entry_stack", "shape": [2], "at": [i] }, "pattern": slots(3 * i, 1) },
            { "type": "slot", "handle": { "kind": "text", "id": "entry_name", "shape": [2], "at": [i] }, "x": 8 + 54 * i, "y": 40, "width": 30 }
        ])
    };
    let lamp = |row: u32, column: u32| {
        json!({ "type": "switch", "handle": { "kind": "flag", "id": "lamp", "shape": [1, 2], "at": [row, column] },
                "x": 8 + 10 * column, "y": 60, "children": [
                    { "type": "case", "value": "true", "children": [] },
                    { "type": "case", "value": "false", "children": [] }
                ] })
    };
    let pick = |i: u32| json!({ "kind": "action", "id": "pick", "shape": [2], "at": [i] });
    let [a, b] = [cell(0), cell(1)];
    let mut children = vec![section(json!([region(pick(0), 1), region(pick(1), 4)])), lamp(0, 0), lamp(0, 1)];
    children.extend(a.as_array().unwrap().iter().chain(b.as_array().unwrap()).cloned());
    let shop = window("shop", "generic_9x3", Value::Array(children));
    let files = compile(&project(json!([shop]))).unwrap();
    let shop = &files["ShopView.kt"];
    assert_lines(
        shop,
        &[
            "protected abstract fun onPick(click: IndexedClick)",
            "protected abstract fun entryStack(index: Int): ItemStack?",
            "protected abstract fun lamp(row: Int, column: Int): Boolean",
            "for (index in 0 until PICK_SIZE) {",
            "button(\"pick[$index]\") { click ->",
            "onPick(IndexedClick(click.slot, index, click.shift, click.right))",
            "item(\"entry_stack[$index]\") { entryStack(index) }",
            "slot(\"entry_name[$index]\") { entryName(index) }",
            "for (column in 0 until LAMP_COLUMNS) {",
            "switch(\"lamp[$row][$column]\") { lamp(row, column).toString() }",
            "public const val ENTRY_STACK_SIZE: Int = 2",
            "public const val LAMP_ROWS: Int = 1",
            "public const val LAMP_COLUMNS: Int = 2",
        ],
    );
}

#[test]
fn handles_shared_by_windows_give_each_view_a_member_and_must_agree() {
    let title = |id: &str, x: u32| json!({ "type": "slot", "handle": { "kind": "text", "id": id }, "x": x, "y": 6, "width": 40 });
    let a = window("a", "generic_9x1", json!([title("title", 8), title("title", 60)]));
    let b = window("b", "generic_9x1", json!([title("title", 8)]));
    let files = compile(&project(json!([a, b]))).unwrap();
    assert_lines(&files["AView.kt"], &["slot(\"title\") { title() }", "slot(\"title~2\") { title() }"]);
    assert_lines(&files["BView.kt"], &["protected abstract fun title(): Component"]);

    let flag = json!({ "type": "switch", "handle": { "kind": "flag", "id": "title" }, "x": 8, "y": 20, "children": [
        { "type": "case", "value": "true", "children": [] },
        { "type": "case", "value": "false", "children": [] }
    ] });
    let c = window("c", "generic_9x1", json!([flag]));
    assert!(
        error(&project(json!([window("a", "generic_9x1", json!([title("title", 8)])), c])))
            .contains("handle `title` is declared differently by window `a` and window `c`")
    );
}

#[test]
fn handle_values_and_uses_are_checked() {
    let show = |handle: Value| {
        project(json!([window(
            "a",
            "generic_9x1",
            json!([{ "type": "switch", "handle": handle, "x": 8, "y": 6, "children": [
            { "type": "case", "value": "true", "children": [] },
            { "type": "case", "value": "false", "children": [] }
        ] }])
        )]))
    };
    let typo = json!({ "kind": "value", "id": "mode", "values": ["buy", "sell"], "is": "sel" });
    assert!(error(&show(typo)).contains("`mode.is(\"sel\")` is not one of its values"));
    let indexed = json!({ "kind": "flag", "id": "lamp", "shape": [2] });
    assert!(error(&show(indexed)).contains("read it with `.at()`"));
    let text = json!({ "kind": "text", "id": "title" });
    assert!(error(&show(text)).contains("a text handle cannot be used as a switch or show condition"));
}

#[test]
fn selection_clicks_assign_state_named_value() {
    let value = json!({ "kind": "selection", "id": "value", "values": ["a", "b"] });
    let tabs = window(
        "tabs",
        "generic_9x1",
        json!([section(json!([region(json!({ "kind": "selection", "id": "value", "values": ["a", "b"], "set": "a" }), 0)])),
               { "type": "switch", "handle": value, "x": 8, "y": 6, "children": [
                 { "type": "case", "value": "a", "children": [] }, { "type": "case", "value": "b", "children": [] }] }]),
    );
    let files = compile(&project(json!([tabs]))).unwrap();
    assert_lines(&files["TabsView.kt"], &["if (value != Value.A) {", "    value = Value.A"]);
}

#[test]
fn click_lambdas_do_not_shadow_state_named_it() {
    let toggle = json!({ "kind": "toggle", "id": "it" });
    let other = window("flip", "generic_9x1", json!([section(json!([region(toggle, 0)]))]));
    let pick = json!({ "kind": "selection", "id": "pick", "values": ["a", "b"], "set": "a" });
    let named = json!({ "kind": "selection", "id": "it", "values": ["a", "b"], "set": "b" });
    let tabs = window("tabs", "generic_9x1", json!([section(json!([region(named, 1), region(pick, 2)]))]));
    let mut files = compile(&project(json!([tabs]))).unwrap();
    files.extend(compile(&project(json!([other]))).unwrap());
    assert_lines(&files["FlipView.kt"], &["button(\"it\") { _ ->", "    it = !it"]);
    assert_lines(&files["TabsView.kt"], &["    if (it != It.B) {", "    it = It.B"]);
}

#[test]
fn nested_enums_cannot_shadow_runtime_types() {
    let mode = json!({ "kind": "value", "id": "window_collection", "values": ["a"] });
    let flag = json!({ "type": "switch", "handle": mode, "x": 8, "y": 6, "children": [
        { "type": "case", "value": "a", "children": [] }] });
    let message = error(&project(json!([window("a", "generic_9x1", json!([flag]))])));
    assert!(message.contains("reserved WindowView member `WindowCollection`"), "{message}");
}

#[test]
fn is_prefixed_toggles_claim_kotlins_real_accessor_names() {
    let toggle = |id: &str, column: u32| section(json!([region(json!({ "kind": "toggle", "id": id }), column)]));
    let flag = |id: &str| {
        json!({ "type": "switch", "handle": { "kind": "flag", "id": id }, "x": 8, "y": 6, "children": [
            { "type": "case", "value": "true", "children": [] },
            { "type": "case", "value": "false", "children": [] }] })
    };
    let clash = json!([toggle("foo", 0), toggle("is_foo", 1)]);
    let message = error(&project(json!([window("a", "generic_9x1", clash)])));
    assert!(message.contains("both map to member `setFoo`"), "{message}");
    let distinct = json!([toggle("is_foo", 0), flag("get_is_foo")]);
    compile(&project(json!([window("a", "generic_9x1", distinct)]))).unwrap();
}

#[test]
fn handle_names_cannot_clash_with_generated_accessors_or_the_companion() {
    let toggle = section(json!([region(json!({ "kind": "toggle", "id": "favorites" }), 0)]));
    let flag = json!({ "type": "switch", "handle": { "kind": "flag", "id": "get_favorites" }, "x": 8, "y": 6, "children": [
        { "type": "case", "value": "true", "children": [] },
        { "type": "case", "value": "false", "children": [] }] });
    for children in [json!([toggle, flag]), json!([flag, toggle])] {
        let message = error(&project(json!([window("a", "generic_9x1", children)])));
        assert!(message.contains("both map to member `getFavorites`"), "{message}");
    }
    let companion = json!({ "type": "switch", "handle": { "kind": "value", "id": "companion", "values": ["a"] },
                            "x": 8, "y": 6, "children": [{ "type": "case", "value": "a", "children": [] }] });
    let message = error(&project(json!([window("a", "generic_9x1", json!([companion]))])));
    assert!(message.contains("reserved WindowView member `Companion`"), "{message}");
}

#[test]
fn sprite_handles_narrowed_with_only_return_their_own_enum() {
    let lamp = |fill: &str| json!({ "art": "shape", "kind": "badge", "width": 8, "height": 8, "fill": fill });
    let slot = |only: Value| {
        json!({ "type": "sprite_slot", "handle": { "kind": "sprite", "id": "lamp", "only": only },
                "x": 8, "y": 6, "width": 8, "height": 8 })
    };
    let catalog = |only: Value| {
        let mut project = project(json!([window("panel", "generic_9x1", json!([slot(only)]))]));
        project["sprites"] = json!({ "lamp_on": lamp("#00ff00"), "lamp_off": lamp("#ff0000") });
        project
    };
    let files = compile(&catalog(json!(["lamp_on", "lamp_off"]))).unwrap();
    assert_lines(&files["WindowSprite.kt"], &["LAMP_ON(\"lamp_on\"),", "LAMP_OFF(\"lamp_off\"),"]);
    assert_lines(
        &files["PanelView.kt"],
        &[
            "public enum class LampSprite(public val sprite: WindowSprite) {",
            "LAMP_ON(WindowSprite.LAMP_ON),",
            "protected abstract fun lampSprite(): LampSprite?",
            "sprite(\"lamp\") { lampSprite()?.sprite?.id }",
        ],
    );
    let message = error(&catalog(json!(["lamp_on", "lamp_dim"])));
    assert!(message.contains("lamp_dim"), "{message}");
}
