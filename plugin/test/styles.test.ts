import assert from "node:assert/strict";
import { test } from "node:test";

import { create, createTheme, defineVars, derive, mix, raw, shape, variants } from "../src/authoring/index.ts";
import type { Color, Var } from "../src/authoring/index.ts";
import { defineWindows } from "../src/authoring/index.ts";
import { collectInputs } from "../src/project.ts";
import { Box, Case, Hud, Image, Switch, Text, Window } from "../src/authoring/jsx.ts";
import type { Element } from "../src/authoring/types.ts";

const colors = defineVars({ face: "#0994c6", text: "#ffffff" });
const sizes = defineVars({ pad: 2 });
const ember = createTheme(colors, { face: "#c0503a" });
const moss = createTheme(colors, { face: "#3a8c40", text: "#eeeeaa" });

/** A raised bevel whose edge colors derive from the resolved fill. */
const raised = (fill: Var<Color>) =>
    derive((get) => {
        const base = get(fill);
        return shape({ kind: "button", fill: base, highlight_color: mix(base, "#ffffff", 0.45) });
    });

function children(el: { children?: Element[] | undefined }): Element[] {
    return el.children ?? [];
}

test("two themes in one window resolve the same art differently", () => {
    const art = shape({ kind: "panel", fill: colors.face });
    const doc = Window({
        name: "pair",
        container: "generic_9x3",
        frame: art,
        children: [Box({ theme: ember, frame: art }), Box({ theme: moss, frame: art })],
    });
    const window = doc.windows[0]!;
    const [a, b] = children(window);
    assert.deepEqual(window.frame, { kind: "panel", fill: "#0994c6", art: "shape" });
    assert.deepEqual(a, { type: "flex", frame: { kind: "panel", fill: "#c0503a", art: "shape" }, children: [] });
    assert.deepEqual(b, { type: "flex", frame: { kind: "panel", fill: "#3a8c40", art: "shape" }, children: [] });
    assert.ok(!JSON.stringify(doc).includes("theme"));
});

test("the nearest theme wins, and vars resolve where they are used", () => {
    const doc = Window({
        name: "nested",
        container: "generic_9x3",
        theme: ember,
        text: { color: colors.text },
        children: [
            Box({
                theme: moss,
                padding: sizes.pad,
                children: [Box({ theme: createTheme(colors, { face: "#101010" }), frame: raised(colors.face) })],
            }),
            Text({ children: "Outer" }),
        ],
    });
    const [inner, outer] = children(doc.windows[0]!);
    assert.ok(inner?.type === "flex");
    assert.deepEqual(inner.style, { padding: 2 });
    const innermost = children(inner)[0];
    assert.ok(innermost?.type === "flex");
    assert.deepEqual(innermost.frame, {
        kind: "button",
        fill: "#101010",
        highlight_color: mix("#101010", "#ffffff", 0.45),
        art: "shape",
    });
    assert.ok(outer?.type === "label");
    assert.equal(outer.color, "#ffffff");
});

test("a recipe derives different art under each theme", () => {
    const art = raised(colors.face);
    const window = Window({
        name: "recipes",
        container: "generic_9x1",
        children: [Image({ art }), Box({ theme: ember, children: Image({ art }) })],
    }).windows[0]!;
    const [plain, themed] = children(window);
    assert.ok(plain?.type === "sprite");
    const nested = children(themed as { children?: Element[] })[0];
    assert.ok(nested?.type === "sprite");
    assert.deepEqual(plain.art, {
        kind: "button",
        fill: "#0994c6",
        highlight_color: mix("#0994c6", "#ffffff", 0.45),
        art: "shape",
    });
    assert.deepEqual(nested.art, {
        kind: "button",
        fill: "#c0503a",
        highlight_color: mix("#c0503a", "#ffffff", 0.45),
        art: "shape",
    });
    assert.notDeepEqual(plain.art, nested.art);
});

test("styles merge in order, skip falsy entries, and lose to explicit props", () => {
    const s = create({
        card: { padding: 2, gap: 1, grow: true, frame: "panel" },
        wide: { width: 64, padding: 4 },
        label: { color: colors.text, shadow: true },
    });
    const box = Box({ style: [s.card, false, [null, s.wide]], gap: 3 });
    assert.deepEqual(box, {
        type: "flex",
        frame: "panel",
        style: { gap: 3, padding: 4, width: 64 },
        children: [],
        layout: { grow: 1 },
    });
    const label = Window({
        name: "w",
        container: "generic_9x1",
        children: Text({ style: s.label, shadow: false, children: "Hi" }),
    });
    assert.deepEqual(children(label.windows[0]!)[0], { type: "label", text: "Hi", color: "#ffffff", shadow: false });
});

test("primitives reject style properties and groups they do not take", () => {
    const s = create({
        tab: variants({ base: { padding: 1 }, selected: { padding: 2 } }),
        label: { color: "#ffffff" },
    });
    assert.throws(() => Box({ style: s.label as never }), /<Box> does not accept style property `color`/);
    assert.throws(() => Box({ style: s.tab as never }), /variants group; pass one of its variants/);
    assert.throws(() => create({ label: { colour: "#ffffff" } as never }), /unknown style property `colour`/);
    assert.throws(() => createTheme(colors, { face: "red" as Color }), /"#rrggbb" color/);
    assert.throws(() => JSON.stringify(Box({ frame: shape({ fill: colors.face }) })), /was not resolved/);
});

test("switch cases and HUDs carry styles and themes", () => {
    const hud = Hud({
        name: "status",
        theme: ember,
        style: { frame: shape({ fill: colors.face }), padding: 2 },
        children: Switch({
            bind: "mode",
            style: { grow: 1 },
            children: Case({ value: "on", theme: moss, style: { frame: shape({ fill: colors.face }) } }),
        }),
    }).huds[0]!;
    assert.deepEqual(hud.frame, { fill: "#c0503a", art: "shape" });
    const root = children(hud)[0];
    assert.ok(root?.type === "flex");
    assert.deepEqual(root.style, { direction: "column", padding: 2 });
    const toggle = children(root)[0];
    assert.ok(toggle?.type === "switch");
    assert.deepEqual(toggle.layout, { grow: 1 });
    assert.deepEqual(toggle.children[0]?.frame, { fill: "#3a8c40", art: "shape" });
});

test("raw primitives take styles and themes", () => {
    const s = create({
        panel: { padding: 2, grow: true, text: { color: colors.text } },
        label: { color: "#aaaaaa", grow: 1 },
    });
    const doc = raw.ui({
        name: "raw",
        container: "generic_9x1",
        theme: moss,
        style: { frame: shape({ fill: colors.face }) },
        children: [
            raw.box({
                style: [s.panel, { direction: "row", minWidth: 4 }],
                layout: { shrink: 0 },
                children: [raw.label("A"), raw.label("B", { style: s.label, color: "#000000" })],
            }),
            raw.switchOn("mode", [
                raw.case("on", { theme: ember, frame: shape({ fill: colors.face }), style: { justify: "center" } }),
            ]),
        ],
    });
    const window = doc.windows[0]!;
    assert.deepEqual(window.frame, { fill: "#3a8c40", art: "shape" });
    const [box, toggle] = children(window) as [Element, Element];
    assert.deepEqual(box, {
        type: "flex",
        style: { direction: "row", padding: 2, min_width: 4 },
        layout: { grow: 1, shrink: 0 },
        children: [
            { type: "label", text: "A", color: "#eeeeaa" },
            { type: "label", text: "B", color: "#000000", layout: { grow: 1 } },
        ],
    });
    assert.ok(toggle.type === "switch");
    const caseEl = toggle.children[0]!;
    assert.deepEqual(caseEl.style, { direction: "column", justify: "center" });
    assert.deepEqual(caseEl.frame, { fill: "#c0503a", art: "shape" });
});

test("font and smallCaps are one selection across styles and props", () => {
    const text = (el: Element) => el as unknown as Record<string, unknown>;
    const a = text(Text({ style: { smallCaps: true }, font: "custom", children: "A" }));
    assert.equal(a["font"], "custom");
    assert.equal(a["small_caps"], undefined);
    const b = text(Text({ style: { font: "custom" }, smallCaps: true, children: "B" }));
    assert.equal(b["small_caps"], true);
    assert.equal(b["font"], undefined);
    const c = text(Text({ style: [{ font: "custom" }, { smallCaps: true }], children: "C" }));
    assert.equal(c["small_caps"], true);
    assert.equal(c["font"], undefined);
    const d = text(Text({ style: [{ smallCaps: true }, { font: "custom" }], children: "D" }));
    assert.equal(d["font"], "custom");
    assert.equal(d["small_caps"], undefined);
});

test("raw boxes and cases take only typed styles", () => {
    const merged = raw.box({ style: [{ minWidth: 4 }, { minWidth: 8 }] });
    assert.deepEqual(merged.style, { min_width: 8 });
    assert.throws(() => raw.box({ style: { min_width: 4 } as never }), /does not accept style property `min_width`/);
    const stored = create({ label: { color: "#ffffff" } });
    assert.throws(() => raw.box({ style: stored.label as never }), /does not accept style property `color`/);
    assert.throws(() => raw.case("on", { style: stored.label as never }), /does not accept style property `color`/);
    assert.deepEqual(raw.flex({ style: { min_width: 4 } }).style, { min_width: 4 });
});

test("create rejects unknown nested text properties", () => {
    assert.throws(
        () => create({ label: { text: { color: "#ffffff", colour: "#aaaaaa" } } as never }),
        /unknown text property `colour`/,
    );
    assert.throws(() => variants({ base: { text: { colour: "#aaaaaa" } } as never }), /unknown text property `colour`/);
});

test("bare window and HUD definitions apply their styles", () => {
    const frame = shape({ fill: colors.face });
    const style = { frame, text: { color: colors.text } };
    const { documents } = collectInputs({
        discovered: (glob: unknown) =>
            glob === "jsx"
                ? []
                : [
                      {
                          path: "window/index.ts",
                          module: {
                              default: defineWindows({
                                  windows: [{ name: "w", container: "generic_9x1", style, children: [raw.label("A")] }],
                                  huds: [{ name: "h", style, children: [raw.label("B")] }],
                              }),
                          },
                      },
                  ],
        sourceFiles: () => [],
        remove: () => {},
        read: () => undefined,
        readSource: () => undefined,
    } as never);
    const [window, hud] = documents.map((doc) => (doc.window ?? doc.hud)!);
    for (const doc of [window!, hud!]) {
        assert.deepEqual(doc.frame, { fill: "#0994c6", art: "shape" });
        assert.ok(!("style" in doc));
        assert.equal((children(doc)[0] as { color?: string }).color, "#ffffff");
    }
});

test("a raw switch cascades its text style into its cases", () => {
    const sw = raw.switchOn(
        "mode",
        { on: { children: [raw.label("Hello"), raw.label("Bye", { color: "#000000" })] } },
        { style: { text: { color: "#ffffff", bold: true } } },
    );
    assert.ok(sw.type === "switch");
    assert.ok(!("style" in sw));
    const [hello, bye] = children(sw.children[0]!) as { color?: string; bold?: boolean }[];
    assert.deepEqual([hello!.color, hello!.bold], ["#ffffff", true]);
    assert.deepEqual([bye!.color, bye!.bold], ["#000000", true]);
});
