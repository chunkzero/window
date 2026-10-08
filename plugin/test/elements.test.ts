import assert from "node:assert/strict";
import { test } from "node:test";

import { action, collection, input, items, text, value } from "../src/bind/index.ts";
import * as raw from "../src/raw/index.ts";
import * as window from "../src/ui/index.ts";
import { Box, Region, Section, Text } from "../src/ui/components.ts";
import * as inventory from "../src/ui/inventory.ts";

const { pattern, shape } = window;

const recess = shape({ kind: "slot", fill: "#102040" });

test("inventory aliases match", () => {
    assert.equal(window.pattern, inventory.pattern);
    assert.equal(window.containerSlot, inventory.containerSlot);
    assert.equal(window.playerSlots, inventory.playerSlots);
});

test("typed slot refs keep their backing area", () => {
    assert.deepEqual(window.containerSlot(7), { area: "container", index: 7 });
    assert.deepEqual(window.playerSlot(7), { area: "player", index: 7 });
});

test("inclusive ranges remain compact for Rust expansion", () => {
    assert.deepEqual(window.containerSlots(3, 8), [{ area: "container", first: 3, last: 8 }]);
});

test("descending ranges fail at the authoring boundary", () => {
    assert.throws(() => window.playerSlots(8, 3), /last must be greater than or equal to first/);
});

test("hotbar helpers reject invisible indices", () => {
    assert.throws(() => window.hotbarSlot(9), /between 0 and 8/);
    assert.throws(() => window.hotbarSlots(0, 9), /between 0 and 8/);
});

test("rect patterns preserve slot-space coordinates", () => {
    assert.deepEqual(pattern.rect({ section: "player", x: 1, y: 0, width: 3, height: 1 }), {
        kind: "rect",
        section: "player",
        x: 1,
        y: 0,
        width: 3,
        height: 1,
    });
});

test("explicit patterns reject empty slot lists", () => {
    assert.throws(() => pattern.slots([]), /requires at least one slot index/);
    assert.deepEqual(pattern.slots([1, 2]), { kind: "slots", section: "container", slots: [1, 2] });
});

test("grid patterns validate every dimension", () => {
    assert.throws(
        () => pattern.grid({ x: 0, y: 0, columns: 2, rows: 2, cell_width: 0, cell_height: 1 }),
        /cell_width must be a positive integer/,
    );
});

test("collections draw their slot frames natively", () => {
    const results = raw.collection(collection("results"), {
        frame: recess,
        transform: { section: "container", x: 0, y: 0, width: 3, height: 1 },
    });
    assert.equal(results.frame, recess);
    assert.deepEqual(results.handle, { kind: "collection", id: "results" });
});

test("bound elements require a handle of their kind", () => {
    assert.throws(() => raw.collection("results" as never), /collection source must be a `collection` handle/);
    assert.throws(() => raw.items(text("icon") as never), /items source must be a `items` handle/);
    assert.throws(() => raw.text(items("caption") as never), /text source must be a `text` handle/);
    assert.throws(() => raw.image("rivet" as never), /image art must be an object/);
});

test("anvil input preserves initial text and item model", () => {
    assert.deepEqual(raw.input(input("query"), { initial: "maps", item_model: "window:gui/search" }), {
        type: "anvil_input",
        handle: { kind: "input", id: "query" },
        initial: "maps",
        item_model: "window:gui/search",
    });
});

test("dynamic text may defer width to its parent box", () => {
    const caption = raw.text(text("caption"), { bold: true });
    assert.equal(caption.type, "slot");
    assert.equal(caption.width, undefined);
});

test("unknown option keys are rejected at runtime", () => {
    assert.throws(() => raw.box({ bogus: 1 } as never), /box does not accept option `bogus`/);
    assert.throws(() => raw.region({ width: 18 }), /`width` and `height` together/);
});

test("switchOn builds a switch over its cases", () => {
    const buy = raw.text("Buy");
    assert.deepEqual(
        raw.switchOn(
            value("mode", ["buy", "sell"]),
            { buy: { frame: recess, children: [buy] }, sell: {} },
            { x: 4, y: 6 },
        ),
        {
            type: "switch",
            handle: { kind: "value", id: "mode", values: ["buy", "sell"] },
            x: 4,
            y: 6,
            children: [
                { type: "case", value: "buy", frame: recess, style: { direction: "column" }, children: [buy] },
                { type: "case", value: "sell", style: { direction: "column" }, children: [] },
            ],
        },
    );
    assert.throws(() => raw.switchOn(value("mode", ["buy"]), {}), /requires at least one case/);
    assert.throws(
        () => raw.switchOn(value("mode", ["buy"]), { buy: { width: 3 } as never }),
        /does not accept option `width`/,
    );
});

test("raw primitives build the same elements as their JSX components", () => {
    const fn = raw.section("container", {
        frame: recess,
        children: [
            raw.box({
                frame: recess,
                style: { direction: "row", align: "center", padding: { left: 4, right: 4 } },
                layout: { column: { span: 9 } },
                children: [raw.text(text("selection")), raw.text(text("price"), { width: 40, align: "right" })],
            }),
            raw.region({
                on_click: action("info"),
                tooltip: "Info",
                layout: { column: { start: 1, span: 2 }, row: { start: 2, span: 1 } },
            }),
        ],
    });
    const jsx = Section({
        of: "container",
        frame: recess,
        children: [
            Box({
                span: 9,
                frame: recess,
                direction: "row",
                align: "center",
                padding: { left: 4, right: 4 },
                children: [Text({ bind: text("selection") }), Text({ bind: text("price"), width: 40, align: "right" })],
            }),
            Region({ onClick: action("info"), tooltip: "Info", span: [2, 1], at: [0, 1] }),
        ],
    });
    assert.deepEqual(fn, jsx);
    assert.deepEqual(raw.section("hotbar"), Section({ of: "hotbar" }));
    assert.deepEqual(raw.box(), Box({}));
});

test("layout helpers reject options their element does not accept", () => {
    assert.throws(() => raw.section("container", { layout: {} } as never), /does not accept option `layout`/);
    assert.throws(() => raw.section("chest" as never), /section kind must be one of/);
    assert.throws(() => raw.input(input("q"), { layout: {} } as never), /does not accept option `layout`/);
});
