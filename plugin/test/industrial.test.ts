import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import * as industrial from "../src/theme/industrial/index.ts";
import { containerLayout, createTheme } from "../src/ui/index.ts";
import { action, collection, flag, items, selection, toggle } from "../src/bind/index.ts";
import { Image, Window } from "../src/ui/components.ts";
import { resolveTokens } from "../src/ui/tokens.ts";
import type { Element } from "../src/ui/document.ts";

type Node = Record<string, unknown> & { children?: Node[] };

/** Every node under `root`, depth first. */
function walk(root: unknown, out: Node[] = []): Node[] {
    if (Array.isArray(root)) {
        root.forEach((entry) => walk(entry, out));
    } else if (typeof root === "object" && root !== null) {
        const node = root as Node;
        out.push(node);
        walk(node.children ?? [], out);
    }
    return out;
}

const ofType = (root: unknown, type: string): Node[] => walk(resolveTokens(root)).filter((node) => node.type === type);
const snake = (key: string): string => key.replace(/[A-Z]/g, (c) => "_" + c.toLowerCase());

test("industrial art reproduces the recorded industrial theme", () => {
    const expected = JSON.parse(readFileSync(new URL("industrial.json", import.meta.url), "utf8"));
    const recorded: Record<string, unknown> = { ...expected.frames, ...expected.sprites };
    const resolved = resolveTokens(industrial.art) as unknown as Record<string, Record<string, unknown>>;
    for (const [key, value] of Object.entries(resolved)) {
        const { name: _, art: __, ...style } = value;
        assert.deepEqual(style, recorded[snake(key)], key);
    }
    assert.equal(Object.keys(resolved).length, Object.keys(recorded).length);
});

test("a theme over industrial colors recolors its art", () => {
    const doc = Window({
        name: "themed",
        container: "generic_9x3",
        theme: createTheme(industrial.colors, { button: "#808080" }),
        children: [industrial.Button({ onClick: action("go"), children: "Go" })],
    });
    const [face] = walk(doc.windows[0]!.children).filter((node) => node.frame !== undefined);
    assert.equal((face?.frame as { fill: string }).fill, "#808080");
});

test("a disabled button shows its disabled state and takes no clicks", () => {
    const button = industrial.Button({
        onClick: action("buy"),
        enabled: flag("can_buy"),
        tooltip: "Buy",
        disabled: { tooltip: "Buy unavailable" },
        children: "Buy",
    });
    const [on, off] = ofType(button, "case");
    const region = (node: Node | undefined) => ofType(node, "region")[0];
    assert.deepEqual(region(on), { type: "region", on_click: { kind: "action", id: "buy" }, tooltip: "Buy" });
    assert.deepEqual(region(off), { type: "region", tooltip: "Buy unavailable" });
    assert.equal((off?.frame as { name: string }).name, "industrial/button-disabled");
    assert.throws(() => industrial.Button({ onClick: action("x"), disabled: {} }), /enabled/);
});

test("tabs draw one selected and unselected state per value and set the selection", () => {
    const category = selection("category", ["all", "gear"]);
    const tabs = industrial.Tabs({ bind: category, span: 3, children: (value) => value });
    assert.equal(tabs.length, 2);
    const [gear] = ofType(tabs[1], "switch");
    assert.equal((gear?.handle as { is: string }).is, "gear");
    const clicks = ofType(gear, "region").map((node) => (node.on_click as { set: string }).set);
    assert.deepEqual(clicks, ["gear", "gear"]);
});

test("a toggle switches on its handle and flips it on click", () => {
    const auto = toggle("auto");
    const node = industrial.Toggle({ bind: auto, on: { tooltip: "On" }, off: { tooltip: "Off" }, children: "Auto" });
    const [state] = ofType(node, "switch");
    assert.equal((state?.handle as { id: string }).id, "auto");
    const regions = ofType(node, "region");
    assert.deepEqual(
        regions.map((region) => region.tooltip),
        ["On", "Off"],
    );
    assert.ok(regions.every((region) => (region.on_click as { kind: string }).kind === "toggle"));
});

test("a repeater cell's region covers the whole cell, including its item slot", () => {
    const repeater: Element = industrial.Repeater({
        onClick: action("recipe", { shape: [2] }),
        item: items("stack", { shape: [2] }),
        cell: [3, 1],
        columns: 2,
        rows: 1,
    });
    assert.equal(ofType(repeater, "item").length, 2);
    const regions = ofType(repeater, "region");
    assert.equal(regions.length, 2);
    for (const region of regions) {
        assert.deepEqual([region.width, region.height, region.layout], [undefined, undefined, undefined]);
    }
});

test("a theme over the rivet colors recolors the rivet", () => {
    const rivet = (overrides: object) =>
        JSON.stringify(
            Window({
                name: "rivets",
                container: "generic_9x3",
                theme: createTheme(industrial.colors, overrides),
                children: [Image({ art: industrial.art.rivet })],
            }),
        );
    assert.ok(rivet({ rivet: "#ff0000" }).includes("#ff0000"));
    assert.ok(!rivet({}).includes("#ff0000"));
});

test("a collection style overrides the default frame, and an explicit frame overrides the style", () => {
    const frame = (props: object) =>
        ofType(industrial.Collection({ bind: collection("shop"), ...props }), "collection")[0]?.frame;
    assert.equal((frame({}) as { name: string }).name, "industrial/slot");
    assert.equal(frame({ style: { frame: "custom" } }), "custom");
    assert.equal(frame({ style: { frame: "custom" }, frame: "explicit" }), "explicit");
});

test("explicitly undefined props keep their defaults", () => {
    const [cell] = ofType(industrial.Collection({ bind: collection("shop"), selected: undefined }), "collection");
    assert.equal((cell?.selected_sprite as { name: string }).name, "industrial/slot-selected");
    const repeater = industrial.Repeater({ cell: [2, 1], columns: 3, rows: 1, span: undefined });
    assert.deepEqual(ofType(repeater, "flex")[0]?.layout, { column: { span: 6 }, row: { span: 1 } });
});

test("tabs rendered from plain text get the text as their tooltip", () => {
    const tabs = industrial.Tabs({ bind: selection("category", ["all", "gear"]), children: (value) => value });
    assert.deepEqual(
        ofType(tabs, "region").map((region) => region.tooltip),
        ["all", "all", "gear", "gear"],
    );
});

test("a face's text font replaces a layer's small caps", () => {
    const button = industrial.Button({
        onClick: action("go"),
        style: { base: { text: { smallCaps: true } } },
        text: { font: "custom" },
        children: "Go",
    });
    const [label] = ofType(button, "label");
    assert.equal(label?.font, "custom");
    assert.equal(label?.small_caps, undefined);
});

test("container layouts match the compiler's screen metrics", () => {
    const chest = containerLayout("generic_9x6");
    assert.equal(chest.height, 222);
    assert.deepEqual(chest.sections.container.slots[10], { x: 26, y: 36, width: 16, height: 16 });
    assert.deepEqual(chest.sections.player.bounds, { x: 8, y: 139, width: 160, height: 52 });
    assert.deepEqual(chest.sections.hotbar.bounds, { x: 8, y: 197, width: 160, height: 16 });
    const anvil = containerLayout("anvil");
    assert.deepEqual(anvil.title, { x: 60, y: 6 });
    assert.deepEqual(
        anvil.sections.container.slots.map((slot) => slot.x),
        [27, 76, 134],
    );
    assert.deepEqual(anvil.sections.hotbar.slots[0], { x: 8, y: 142, width: 16, height: 16 });
    assert.deepEqual(anvil.input, { x: 59, y: 20, width: 110, height: 16 });
});

test("an industrial window without its inventory claims it and ends below the container", () => {
    const shell = (inventory: boolean) => industrial.Window({ name: "w", container: "anvil", inventory }).windows[0]!;
    const open = shell(true);
    assert.deepEqual(open.bleed, { top: 1, right: 4, bottom: 8, left: 4 });
    assert.equal(ofType(open.children, "section").length, 0);
    const closed = shell(false);
    assert.deepEqual(closed.bleed, { top: 1, right: 4, bottom: 0, left: 4 });
    assert.deepEqual(
        ofType(closed.children, "section").map((section) => [section.section, section.claim]),
        [
            ["player", "all"],
            ["hotbar", "all"],
        ],
    );
});
