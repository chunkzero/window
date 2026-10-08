import assert from "node:assert/strict";
import { test } from "node:test";

import { Fragment, createElement, jsx } from "../.rpp/sdk/jsx.ts";
import { builtin, collection, selection, shape, sprite, text, texture, toggle, value } from "../src/authoring/index.ts";
import { raw } from "../src/authoring/index.ts";
import { Box, Case, Collection, Hud, Image, Region, Section, Switch, Text, Window } from "../src/authoring/jsx.ts";

test("JSX text defaults do not mutate reusable children", () => {
    const child = Box({ children: Text({ children: "Hello" }) });
    const red = Box({ text: { color: "#ff0000" }, children: child });
    const blue = Box({ text: { color: "#0000ff" }, children: child });
    const label = (node: typeof child) => {
        assert.equal(node.type, "flex");
        const nested = node.children?.[0];
        assert.equal(nested?.type, "flex");
        const text = nested.children?.[0];
        assert.equal(text?.type, "label");
        return text;
    };
    assert.equal(label(red).color, "#ff0000");
    assert.equal(label(blue).color, "#0000ff");
    assert.equal(label(Box({ children: child })).color, undefined);
});

test("JSX fragments flatten and ignore conditional booleans", () => {
    const box = Box({ children: Fragment({ children: [true, false, null, Text({ children: "All" })] }) });
    assert.ok(box.type === "flex");
    assert.equal(box.children?.length, 1);
    assert.equal(box.children?.[0]?.type, "label");
});

test("bound props take handles, not names", () => {
    assert.throws(() => Text({ bind: "name" as never }), /<Text bind> must be a `text` handle/);
    assert.throws(() => Switch({ bind: "mode" as never, children: [] }), /<Switch bind> must be/);
    assert.throws(() => Image({ art: "coin" as never }), /<Image> requires `art`/);
});

test("bound text carries its fitting options", () => {
    assert.deepEqual(Text({ bind: text("name"), width: 46, lines: 2, lineHeight: 7 }), {
        type: "slot",
        handle: { kind: "text", id: "name" },
        width: 46,
        lines: 2,
        line_height: 7,
    });
    assert.throws(() => Text({ overflow: "ellipsis", children: "Static" }), /<Text overflow> requires `bind`/);
});

test("cases belong directly inside a switch", () => {
    assert.throws(() => Box({ children: Case({ value: "buy" }) }), /<Case> inside <Switch>/);
    assert.throws(() => Switch({ bind: value("mode", ["buy"]), children: "Buy" }), /children must be <Case>/);
});

test("explicit collection spans override the full-width default", () => {
    assert.deepEqual(Collection({ bind: collection("items"), span: [3, 2] }).layout, {
        column: { span: 3 },
        row: { span: 2 },
    });
});

test("the JSX factories preserve a children prop without positional children", () => {
    const props = { children: "Hello" };
    assert.deepEqual(createElement(Text, props), Text(props));
    assert.deepEqual(createElement(Text, props, "Other"), Text({ children: "Other" }));
    assert.deepEqual(jsx(Text, { children: "Other" }), Text({ children: "Other" }));
});

const fields = (element: unknown): Record<string, unknown> => element as Record<string, unknown>;

test("handle props serialize the handle in place of a name", () => {
    const title = text("title");
    assert.deepEqual(Text({ bind: title, width: 8 }), {
        type: "slot",
        handle: { kind: "text", id: "title" },
        width: 8,
    });
    assert.deepEqual(fields(Region({ onClick: builtin("window:close") }))["on_click"], {
        kind: "builtin",
        id: "window:close",
    });
});

test("selection helpers carry the compared or assigned value", () => {
    const category = selection("category", ["all", "gear"], { initial: "gear" });
    assert.deepEqual(fields(Region({ onClick: category.set("all") }))["on_click"], {
        kind: "selection",
        id: "category",
        values: ["all", "gear"],
        initial: "gear",
        set: "all",
    });
    const gear = fields(
        Switch({
            bind: category.is("gear"),
            children: [Case({ value: "true", children: "Gear" }), Case({ value: "false" })],
        }),
    );
    assert.deepEqual(gear["handle"], {
        kind: "selection",
        id: "category",
        values: ["all", "gear"],
        initial: "gear",
        is: "gear",
    });
    const mode = value("mode", ["buy", "sell"]);
    const cases = Switch({ on: mode, children: { buy: Text({ children: "Buy" }), sell: null } });
    assert.ok(cases.type === "switch");
    assert.deepEqual(
        cases.children.map((c) => c.value),
        ["buy", "sell"],
    );
    const lamp = Switch({ on: toggle("lamp"), children: { true: Text({ children: "On" }), false: null } });
    assert.ok(lamp.type === "switch");
    assert.deepEqual(
        lamp.children.map((c) => c.value),
        ["true", "false"],
    );
    assert.throws(() => category.set("magic" as "all"), /not one of its values/);
    assert.throws(() => Switch({ on: mode, children: { buy: null, sell: null, rent: null } }), /`rent`/);
});

test("indexed handles select one entry with at()", () => {
    const names = text("names", { shape: [2] });
    assert.deepEqual(Text({ bind: names.at(1), width: 8 }), {
        type: "slot",
        handle: { kind: "text", id: "names", shape: [2], at: [1] },
        width: 8,
    });
    assert.throws(() => names.at(2), /outside its shape/);
});

test("primitives serialize inline art, regions, sections, and debug names", () => {
    const bevel = shape({ kind: "button", fill: "#3a3a3a" }, { name: "industrial/button" });
    assert.deepEqual(bevel, { kind: "button", fill: "#3a3a3a", name: "industrial/button", art: "shape" });
    assert.deepEqual(texture("window/coin.png", { insets: 2 }), {
        art: "texture",
        texture: "window/coin.png",
        insets: 2,
    });
    assert.throws(() => texture("window/coin.png", { width: 4 }), /together/);

    const category = selection("category", ["all", "gear"]);
    const tab = Section({
        of: "container",
        children: Box({
            span: 3,
            frame: bevel,
            debugName: "tab-all",
            children: [
                Switch({
                    bind: category.is("all"),
                    children: [Case({ value: "true", children: Image({ art: bevel }) }), Case({ value: "false" })],
                }),
                Region({ onClick: category.set("all"), tooltip: "All" }),
            ],
        }),
    });
    assert.equal(tab.type, "section");
    assert.equal(tab.section, "container");
    const box = tab.children?.[0];
    assert.ok(box?.type === "flex");
    assert.equal(box.frame, bevel);
    assert.equal(box.debug_name, "tab-all");
    const [toggle, region] = box.children ?? [];
    assert.ok(toggle?.type === "switch");
    assert.deepEqual(fields(toggle).handle, {
        kind: "selection",
        id: "category",
        values: ["all", "gear"],
        initial: "all",
        is: "all",
    });
    assert.deepEqual(toggle.children[0]?.children, [{ type: "sprite", art: bevel }]);
    assert.deepEqual(region, {
        type: "region",
        on_click: { kind: "selection", id: "category", values: ["all", "gear"], initial: "all", set: "all" },
        tooltip: "All",
    });
});

test("an image binds a sprite handle as a sized runtime slot", () => {
    const lamp = sprite("lamp", { only: ["lamp_on", "lamp_off"] });
    assert.deepEqual(Image({ bind: lamp, size: [8, 6], debugName: "lamp" }), {
        type: "sprite_slot",
        handle: { kind: "sprite", id: "lamp", only: ["lamp_on", "lamp_off"] },
        width: 8,
        height: 6,
        debug_name: "lamp",
    });
});

test("Window and Hud carry debugName to the compiler", () => {
    assert.equal(Window({ name: "w", container: "generic_9x3", debugName: "shop" }).windows[0]?.debug_name, "shop");
    assert.equal(Hud({ name: "h", debugName: "bar" }).huds[0]?.debug_name, "bar");
    assert.equal(raw.ui({ name: "w", container: "generic_9x3", debug_name: "shop" }).windows[0]?.debug_name, "shop");
});
