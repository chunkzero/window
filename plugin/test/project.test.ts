import assert from "node:assert/strict";
import { test } from "node:test";

import { flag, selection, text } from "../src/bind/index.ts";
import { defineWindows, preview, texture } from "../src/ui/index.ts";
import * as raw from "../src/raw/index.ts";
import type { WindowDocument } from "../src/ui/document.ts";
import { buildProject, checkOptions, collectInputs, generate, resourceTexturePath } from "../src/project.ts";
import type { CompileOutput, KotlinOptions, SourceFile, WindowContext, WindowOptions } from "../src/project.ts";

const encoder = new TextEncoder();
const bytes = (text: string): Uint8Array => encoder.encode(text);

interface Fake {
    ctx: WindowContext;
    emitted: Map<string, Uint8Array>;
    outputs: [string, string][];
    removed: string[];
}

function fake(
    modules: Record<string, unknown>,
    sources: Record<string, string> = {},
    options: WindowOptions = {},
    format: WindowContext["pack"]["format"] = { min: 84, max: 84 },
): Fake {
    const emitted = new Map<string, Uint8Array>();
    const outputs: [string, string][] = [];
    const removed: string[] = [];
    const ctx: WindowContext = {
        options,
        pack: { format },
        discovered: (name) =>
            Object.entries(modules)
                .filter(([path]) => name === (path.endsWith(".tsx") ? "jsx" : "definitions"))
                .map(([path, doc]) => ({ path, module: { default: doc } })),
        sourceFiles: () => Object.keys(sources).filter((path) => path.startsWith("window/")),
        read: (path) => emitted.get(path),
        readSource: (path) => (sources[path] === undefined ? undefined : bytes(sources[path])),
        remove: (path) => removed.push(path),
        emit: (path, contents) => emitted.set(path, typeof contents === "string" ? bytes(contents) : contents),
        emitOutput: (root, path) => outputs.push([root, path]),
    };
    return { ctx, emitted, outputs, removed };
}

const shop = raw.ui({ name: "shop", container: "generic_9x6" });
const status = raw.hud({ name: "status", width: 10, height: 10 });

test("documents are collected in path order", () => {
    const { ctx } = fake({ "window/b.tsx": [status], "window/a.ts": shop });
    const { documents } = collectInputs(ctx);
    assert.deepEqual(buildProject(documents, {}, 84).windows, shop.windows);
    assert.equal(documents[0], shop);
    assert.equal(documents[1], status);
});

test("a discovered module without a default document fails naming its path", () => {
    const { ctx } = fake({ "window/a.ts": undefined });
    assert.throws(() => collectInputs(ctx), /window\/a\.ts must export default/);
});

test("project JSON omits empty fonts and sprites", () => {
    assert.deepEqual(JSON.parse(JSON.stringify(buildProject([shop], {}, 88))), {
        windows: shop.windows,
        huds: [],
        options: { core_shaders: { hud: false } },
        target: { pack_format: 88 },
    });
    const runes = { texture: "window/runes.png", chars: ["ab"] };
    const project = buildProject(
        [{ fonts: { runes } }, { ...status, window: shop.windows[0]! }],
        { coreShaders: { hud: true } },
        84,
    );
    assert.deepEqual(project.fonts, { runes });
    assert.deepEqual(project.windows, shop.windows);
    assert.deepEqual(project.huds, status.huds);
    assert.deepEqual(project.options, { core_shaders: { hud: true } });
    assert.deepEqual(project.target, { pack_format: 84 });
    assert.throws(
        () => buildProject([{ fonts: { runes } }, { fonts: { runes } }], {}, 84),
        /fonts declare `runes` twice/,
    );
});

test("vanilla and experimental options reach the project JSON in snake_case", () => {
    const vanilla = { anvil: { field: "f", background: false }, hud: { hotbar: false } };
    const { options } = buildProject([shop], { vanilla, experimental: { anvilUpdates: true } }, 88);
    assert.deepEqual(options, {
        core_shaders: { hud: false },
        vanilla,
        experimental: { anvil_updates: true },
    });
});

test("the hover shader option is sent only when on, and a blanked slot highlight alone needs a build", () => {
    const { options } = buildProject([shop], { coreShaders: { hover: true }, vanilla: { slotHighlight: false } }, 88);
    assert.deepEqual(options, { core_shaders: { hud: false, hover: true }, vanilla: { slotHighlight: false } });
    assert.deepEqual(buildProject([shop], { coreShaders: { hover: false } }, 88).options, {
        core_shaders: { hud: false },
    });
    assert.throws(() => checkOptions({ coreShaders: { hover: 1 } }), /`coreShaders\.hover` must be a boolean; got 1/);
    let compiled = false;
    generate(fake({}, {}, { vanilla: { slotHighlight: false } }).ctx, () => {
        compiled = true;
        return { files: [], kotlinFiles: [], warnings: [] };
    });
    assert.equal(compiled, true);
});

test("size options reach the project JSON, and a budget must be a positive integer", () => {
    assert.deepEqual(buildProject([shop], { size: { report: true, budget: 3_000_000 } }, 88).options.size, {
        report: true,
        budget: 3_000_000,
    });
    assert.equal(buildProject([shop], {}, 88).options.size, undefined);
    for (const budget of [0, -5, 1.5, Number.NaN]) {
        assert.throws(() => checkOptions({ size: { budget } }), /`size\.budget` must be a positive integer; got /);
    }
});

test("unknown plugin option keys fail naming the key and the accepted keys", () => {
    checkOptions({
        namespace: "x",
        coreShaders: { hud: true, hover: true },
        vanilla: { anvil: { field: false, background: "bg" }, hud: { hotbar: false }, slotHighlight: false },
        experimental: { anvilUpdates: true },
        size: { report: true, budget: 1 },
        kotlin: { package: "p", output: "o", target: "minestom" },
    });
    assert.throws(
        () => checkOptions({ hudShaders: true }),
        /`hudShaders` is unknown; accepted keys: namespace, coreShaders/,
    );
    assert.throws(
        () => checkOptions({ coreShaders: { enableHud: true } }),
        /`coreShaders\.enableHud`.*in `coreShaders`: hud, hover$/,
    );
    assert.throws(
        () => checkOptions({ vanilla: { hud: { hearts: false } } }),
        /`vanilla\.hud\.hearts`.*hotbar, health/,
    );
    assert.throws(() => checkOptions({ vanilla: true }), /`vanilla` must be an object/);
});

test("plugin option leaves must have the right type", () => {
    assert.throws(
        () => checkOptions({ coreShaders: { hud: "true" } }),
        /Window option `coreShaders\.hud` must be a boolean; got "true"/,
    );
    assert.throws(
        () => checkOptions({ vanilla: { anvil: { field: 12 } } }),
        /`vanilla\.anvil\.field` must be a string or boolean; got 12/,
    );
    assert.throws(
        () => checkOptions({ vanilla: { hud: { hotbar: null } } }),
        /`vanilla\.hud\.hotbar` must be a boolean; got null/,
    );
    checkOptions({ coreShaders: { hud: undefined }, namespace: undefined });
});

test("options listed directly in the config are checked when generating", () => {
    const fail = () => assert.fail("compile must not run");
    const typo = fake({ "window/a.ts": shop }, {}, { coreShaders: { hud: true, typo: true } } as WindowOptions);
    assert.throws(() => generate(typo.ctx, fail), /`coreShaders\.typo` is unknown/);
    const kotlin = fake({ "window/a.ts": shop }, {}, { kotlin: { package: "p", target: "minestom" } } as never);
    assert.throws(
        () => generate(kotlin.ctx, fail),
        /`kotlin\.package` is unknown; accepted keys in `kotlin`: packageName, target/,
    );
});

test("a pack with only vanilla replacements still compiles", () => {
    const { ctx } = fake({}, {}, { vanilla: { hud: { hotbar: false } } });
    let compiled: unknown;
    generate(ctx, (_namespace, json) => {
        compiled = JSON.parse(json);
        return { files: [], kotlinFiles: [], warnings: [] };
    });
    assert.deepEqual(compiled, {
        windows: [],
        huds: [],
        options: { core_shaders: { hud: false }, vanilla: { hud: { hotbar: false } } },
        target: { pack_format: 84 },
    });
    const kept = fake({}, {}, { vanilla: { hud: { hotbar: true }, anvil: { field: true } } });
    generate(kept.ctx, () => assert.fail("compile must not run"));
});

test("referenced resource textures are added once", () => {
    assert.equal(resourceTexturePath("window:gui/badge"), "assets/window/textures/gui/badge.png");
    assert.equal(resourceTexturePath("window:gui/badge.png"), "assets/window/textures/gui/badge.png");
    assert.equal(resourceTexturePath("gui/badge.png"), undefined);

    const sprites: WindowDocument = {
        sprites: {
            a: texture("window:gui/badge.png"),
            b: texture("window:gui/badge"),
            c: texture("window:gui/missing.png"),
        },
    };
    const badge = "assets/window/textures/gui/badge.png";
    const { ctx } = fake({ "window/sprites.ts": sprites }, { [badge]: "png" });
    assert.deepEqual(
        collectInputs(ctx).files.map((file) => file.path),
        [badge],
    );
});

test("window files are compiler inputs and leave the pack", () => {
    const { ctx, removed } = fake(
        { "window/a.ts": shop },
        { "window/badge.png": "png", "assets/minecraft/lang/en_us.json": "{}" },
    );
    assert.deepEqual(
        collectInputs(ctx).files.map((file) => file.path),
        ["window/badge.png"],
    );
    assert.deepEqual(removed, ["window/badge.png"]);
});

test("the newest format of the pack's range is the compile target", () => {
    const { ctx } = fake({ "window/a.ts": shop }, {}, {}, { min: 61, max: 88 });
    generate(ctx, (_namespace, json) => {
        assert.deepEqual(JSON.parse(json).target, { pack_format: 88 });
        return { files: [], kotlinFiles: [], warnings: [] };
    });
});

test("an rpp without pack format ranges is rejected", () => {
    const { ctx } = fake({ "window/a.ts": shop }, {}, {}, 84 as never);
    assert.throws(
        () => generate(ctx, () => assert.fail("compile must not run")),
        /requires rpp 0\.1\.0-nightly\.20261004/,
    );
});

test("compilation is skipped without definitions", () => {
    const { ctx, removed } = fake({}, { "window/badge.png": "png" });
    generate(ctx, () => assert.fail("compile must not run"));
    assert.deepEqual(removed, ["window/badge.png"]);
});

test("Kotlin files are emitted only when configured", () => {
    const output: CompileOutput = {
        files: [{ path: "assets/window/a.json", contents: { tag: "text", val: "{}" } }],
        kotlinFiles: [{ path: "A.kt", contents: { tag: "text", val: "class A" } }],
        warnings: [],
    };
    const calls: (KotlinOptions | undefined)[] = [];
    const compile = (_namespace: string, _json: string, _files: SourceFile[], kotlin: KotlinOptions | undefined) => {
        calls.push(kotlin);
        return output;
    };

    const without = fake({ "window/a.ts": shop });
    generate(without.ctx, compile);
    assert.deepEqual([...without.emitted.keys()], ["assets/window/a.json"]);
    assert.deepEqual(without.outputs, []);

    const kotlin: KotlinOptions = { packageName: "dev.example", target: "multistom" };
    const withKotlin = fake({ "window/a.ts": shop }, {}, { kotlin });
    generate(withKotlin.ctx, compile);
    assert.deepEqual(withKotlin.outputs, [["kotlin", "A.kt"]]);
    assert.deepEqual(calls, [undefined, kotlin]);
});

test("previews are rendered into the previews output only when it is configured", (t) => {
    const warn = t.mock.method(console, "warn", () => {});
    const output: CompileOutput = { files: [], kotlinFiles: [], warnings: [] };
    const label = text("label", { shape: [2] });
    const entry = (previews: boolean) => ({
        "window/index.ts": defineWindows({
            windows: [shop],
            huds: [status],
            ...(previews
                ? {
                      previews: [
                          preview(shop, [
                              [selection("mode", ["buy", "sell"]), "sell"],
                              [label, ["Buy", "Sell"]],
                              [label.at(1), "Vend"],
                          ]),
                          preview(status, [[flag("lit"), true]], { name: "status-lit", scale: 3 }),
                      ],
                  }
                : {}),
        }),
    });
    const rendered: string[] = [];
    const render = (_json: string, _files: SourceFile[], previews: string) => {
        rendered.push(previews);
        return [{ path: "shop.png", contents: { tag: "binary" as const, val: new Uint8Array([1]) } }];
    };

    const configured = fake(entry(true), {}, { previews: { output: "build/previews" } });
    generate(configured.ctx, () => output, render);
    assert.deepEqual(configured.outputs, [["previews", "shop.png"]]);
    assert.deepEqual(configured.emitted.size, 0);
    assert.deepEqual(JSON.parse(rendered[0]!), [
        {
            name: "shop",
            target: { kind: "window", name: "shop" },
            state: [
                { handle: "mode", value: "sell" },
                { handle: "label", at: [0], value: "Buy" },
                { handle: "label", at: [1], value: "Sell" },
                { handle: "label", at: [1], value: "Vend" },
            ],
        },
        {
            name: "status-lit",
            target: { kind: "hud", name: "status" },
            state: [{ handle: "lit", value: true }],
            scale: 3,
        },
    ]);

    const never = () => assert.fail("previews must not render");
    generate(fake(entry(false), {}, { previews: { output: "build/previews" } }).ctx, () => output, never);
    assert.equal(warn.mock.callCount(), 0);
    const unconfigured = fake(entry(true));
    generate(unconfigured.ctx, () => output, never);
    assert.deepEqual(unconfigured.outputs, []);
    assert.match(String(warn.mock.calls[0]?.arguments[0]), /2 previews are declared, but `previews.output` is not set/);

    assert.throws(() => checkOptions({ previews: {} }), /`previews.output` is required/);
    assert.throws(() => checkOptions({ previews: { output: "a", scale: 2 } }), /`previews.scale` is unknown/);
    assert.throws(() => preview([shop, status]), /preview\(\) takes one window or HUD/);
});

test("previews of a target no document lists are rendered against an empty project", (t) => {
    const warn = t.mock.method(console, "warn", () => {});
    const entry = { "window/index.ts": defineWindows({ previews: [preview(shop)] }) };
    const compile = () => assert.fail("nothing is compiled");
    const calls: { project: { windows?: unknown[]; huds?: unknown[] }; previews: unknown }[] = [];
    const render = (project: string, _files: SourceFile[], previews: string) => {
        calls.push({ project: JSON.parse(project), previews: JSON.parse(previews) });
        return [];
    };

    generate(fake(entry, {}, { previews: { output: "build/previews" } }).ctx, compile, render);
    assert.equal(calls.length, 1);
    assert.deepEqual(calls[0]!.previews, [{ name: "shop", target: { kind: "window", name: "shop" }, state: [] }]);
    assert.deepEqual(calls[0]!.project.windows ?? [], []);
    assert.deepEqual(calls[0]!.project.huds ?? [], []);

    generate(fake(entry).ctx, compile, render);
    assert.equal(calls.length, 1);
    assert.match(String(warn.mock.calls[0]?.arguments[0]), /1 previews are declared, but `previews.output` is not set/);
});

test("the size report is not printed when Kotlin output fails", (t) => {
    const log = t.mock.method(console, "log", () => {});
    const output: CompileOutput = {
        files: [],
        kotlinFiles: [{ path: "A.kt", contents: { tag: "text", val: "class A" } }],
        warnings: [],
        sizeReport: "Window output: 1 B",
    };
    const kotlin: KotlinOptions = { packageName: "dev.example", target: "multistom" };
    const { ctx } = fake({ "window/a.ts": shop }, {}, { kotlin, size: { report: true } });
    ctx.emitOutput = () => {
        throw new Error("disk full");
    };
    assert.throws(() => generate(ctx, () => output), /disk full/);
    assert.equal(log.mock.callCount(), 0);
});

test("a missing or unknown Kotlin target is rejected", () => {
    for (const [target, got] of [
        [undefined, "it is missing"],
        ["paper", 'got "paper"'],
    ] as const) {
        const kotlin = { packageName: "dev.example", target } as unknown as KotlinOptions;
        const { ctx } = fake({ "window/a.ts": shop }, {}, { kotlin });
        assert.throws(
            () => generate(ctx, () => assert.fail("compile must not run")),
            new RegExp(`kotlin\\.target must be one of "agnostic", "minestom", "multistom"; ${got}`),
        );
    }
});

test("a window/index entry lists every document; other modules are ordinary", () => {
    const runes = { texture: "window/runes.png", chars: ["ab"] };
    const entry = defineWindows({ fonts: { runes }, windows: [shop], huds: [status.huds[0]!] });
    const { ctx } = fake({ "window/index.ts": entry, "window/handles.ts": undefined, "window/shop.tsx": shop });
    const { documents, warnings } = collectInputs(ctx);
    const project = buildProject(documents, {}, 84);
    assert.deepEqual(project.fonts, { runes });
    assert.deepEqual(project.windows, shop.windows);
    assert.deepEqual(project.huds, status.huds);
    assert.deepEqual(warnings, []);
    const legacy = fake({ "window/index.ts": { themes: [], windows: [shop] } });
    assert.throws(() => collectInputs(legacy.ctx), /no longer takes `themes`/);
});

test("defineWindows lists flatten nested lists and reject documents of another kind", () => {
    const fragment = [raw.hud({ name: "a", width: 1, height: 1 }), [raw.hud({ name: "b", width: 1, height: 1 })]];
    const { ctx } = fake({ "window/index.tsx": defineWindows({ windows: [[shop]], huds: [fragment] }) });
    const project = buildProject(collectInputs(ctx).documents, {}, 84);
    assert.deepEqual(project.windows, shop.windows);
    assert.deepEqual(
        project.huds.map((h) => h.name),
        ["a", "b"],
    );
    const wrong = fake({ "window/index.ts": { windows: [shop, { sprites: {} }] } });
    assert.throws(() => collectInputs(wrong.ctx), /window\/index\.ts: defineWindows `windows` entry 1 is not a window/);
    const bare = fake({ "window/index.ts": { huds: [shop.windows[0]] } });
    assert.throws(() => collectInputs(bare.ctx), /`huds` entry 0 is not a hud/);
});

test("without an entry every module's default export is read, with a warning", () => {
    const { ctx } = fake({ "window/a.ts": shop });
    const { documents, warnings } = collectInputs(ctx);
    assert.deepEqual(documents, [shop]);
    assert.equal(warnings.length, 1);
    assert.match(warnings[0]!, /defineWindows/);
});

test("defineWindows sprites become the catalog and inline art textures are inputs", () => {
    const lamp = texture("window:gui/lamp");
    const window = raw.ui({
        name: "w",
        container: "generic_9x1",
        children: [raw.box({ frame: texture("window:gui/frame.png", { insets: 2 }) })],
    });
    const entry = defineWindows({ sprites: { lamp }, windows: [window] });
    const files = { "assets/window/textures/gui/lamp.png": "png", "assets/window/textures/gui/frame.png": "png" };
    const { ctx } = fake({ "window/index.ts": entry }, files);
    const { documents, files: inputs } = collectInputs(ctx);
    assert.deepEqual(buildProject(documents, {}, 84).sprites, { lamp });
    assert.deepEqual(inputs.map((file) => file.path).sort(), Object.keys(files).sort());
});

test("sprite catalog names that match inherited properties are ordinary names", () => {
    const art = texture("window:gui/lamp");
    const documents: WindowDocument[] = [{ sprites: { constructor: art } }, { sprites: { toString: art } }];
    assert.deepEqual(Object.keys(buildProject(documents, {}, 84).sprites ?? {}), ["constructor", "toString"]);
    assert.throws(() => buildProject([documents[0]!, documents[0]!], {}, 84), /declares `constructor` twice/);
});
