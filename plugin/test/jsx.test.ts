import assert from "node:assert/strict";
import { test } from "node:test";

import { Fragment, createElement, jsx } from "../.rpp/sdk/jsx.ts";
import { Box, Button, Case, Collection, Show, Switch, Tab, Tabs, Text } from "../src/authoring/jsx.ts";

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

test("JSX fragments preserve tabs and ignore conditional booleans", () => {
    const tabs = Tabs({
        name: "kind",
        children: Fragment({ children: [true, false, null, Tab({ value: "all", children: "All" })] }),
    });
    assert.equal(tabs.length, 1);
    assert.equal(tabs[0]?.type, "button");
    assert.equal(tabs[0]?.name, "kind_all");
    assert.throws(() => Box({ children: Tab({ value: "bad" }) }), /inside <Tabs>/);
});

test("hoverOutline reaches buttons, including every tab", () => {
    const button = Button({ name: "buy", hoverOutline: false });
    assert.equal(button.type === "button" && button.hover_outline, false);
    const tabs = Tabs({ name: "kind", hoverOutline: true, children: Tab({ value: "all", children: "All" }) });
    assert.equal(tabs[0]?.type === "button" && tabs[0].hover_outline, true);
});

test("Show is a true/false switch whose box props style the shown case", () => {
    const show = Show({ when: "on_sale", grow: 1, justify: "center", children: Text({ children: "Sale" }) });
    assert.equal(show.type, "switch");
    assert.equal(show.name, "on_sale");
    assert.deepEqual(show.layout, { grow: 1 });
    const [shown, hidden] = show.children;
    assert.equal(shown?.value, "true");
    assert.deepEqual(shown.style, { direction: "column", justify: "center" });
    assert.equal(shown.children?.[0]?.type, "label");
    assert.deepEqual(hidden, { type: "case", value: "false", style: { direction: "column" }, children: [] });
});

test("bindings carry their index into indexed families", () => {
    assert.deepEqual(Text({ bind: "hole", index: [1, 2], width: 8 }), {
        type: "slot",
        name: "hole",
        index: [1, 2],
        width: 8,
    });
    const show = Show({ when: "lit", index: 3, children: Text({ children: "On" }) });
    assert.ok(show.type === "switch");
    assert.equal(show.index, 3);
    assert.throws(() => Text({ index: 0, children: "Static" }), /requires `bind`/);
});

test("bound text carries its fitting options", () => {
    assert.deepEqual(Text({ bind: "name", width: 46, lines: 2, lineHeight: 7 }), {
        type: "slot",
        name: "name",
        width: 46,
        lines: 2,
        line_height: 7,
    });
    assert.throws(() => Text({ overflow: "ellipsis", children: "Static" }), /<Text overflow> requires `bind`/);
});

test("cases belong directly inside a switch", () => {
    assert.throws(() => Box({ children: Case({ value: "buy" }) }), /<Case> inside <Switch>/);
    assert.throws(() => Switch({ bind: "mode", children: "Buy" }), /children must be <Case>/);
});

test("explicit collection spans override the full-width default", () => {
    assert.deepEqual(Collection({ name: "items", span: [3, 2] }).layout, {
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

test("button text defaults respect the button's explicit alignment", () => {
    const button = Button({ name: "go", text: { align: "right" }, children: "Go" });
    assert.equal(button.type, "button");
    const row = button.children?.[0];
    assert.equal(row?.type, "flex");
    const label = row.children?.[0];
    assert.equal(label?.type, "label");
    assert.equal(label.align, "right");
});
