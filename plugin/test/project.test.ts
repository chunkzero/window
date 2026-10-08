import assert from "node:assert/strict";
import { test } from "node:test";

import { defineWindows, raw, texture, theme } from "../src/authoring/index.ts";
import type { WindowDocument } from "../src/authoring/types.ts";
import { buildProject, collectInputs, generate, resourceTexturePath } from "../src/project.ts";
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

test("project JSON omits empty themes", () => {
    assert.deepEqual(JSON.parse(JSON.stringify(buildProject([shop], {}, 88))), {
        windows: shop.windows,
        huds: [],
        options: { hud_shaders: false },
        target: { pack_format: 88 },
    });
    const themed: WindowDocument = theme({ sprites: { badge: { kind: "badge", width: 1, height: 1 } } });
    const project = buildProject([themed, { ...status, window: shop.windows[0]! }], { hudShaders: true }, 84);
    assert.equal(project.themes?.length, 1);
    assert.deepEqual(project.windows, shop.windows);
    assert.deepEqual(project.huds, status.huds);
    assert.deepEqual(project.options, { hud_shaders: true });
    assert.deepEqual(project.target, { pack_format: 84 });
});

test("referenced resource textures are added once", () => {
    assert.equal(resourceTexturePath("window:gui/badge"), "assets/window/textures/gui/badge.png");
    assert.equal(resourceTexturePath("window:gui/badge.png"), "assets/window/textures/gui/badge.png");
    assert.equal(resourceTexturePath("gui/badge.png"), undefined);

    const sprites = theme({
        sprites: {
            a: { texture: "window:gui/badge.png" },
            b: { texture: "window:gui/badge" },
            c: { texture: "window:gui/missing.png" },
        },
    });
    const texture = "assets/window/textures/gui/badge.png";
    const { ctx } = fake({ "window/theme.ts": sprites }, { [texture]: "png" });
    assert.deepEqual(
        collectInputs(ctx).files.map((file) => file.path),
        [texture],
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
    const themed = theme({ colors: { gold: "#ffd75e" } });
    const entry = defineWindows({ themes: [themed], windows: [shop], huds: [status.huds[0]!] });
    const { ctx } = fake({ "window/index.ts": entry, "window/handles.ts": undefined, "window/shop.tsx": shop });
    const { documents, warnings } = collectInputs(ctx);
    const project = buildProject(documents, {}, 84);
    assert.deepEqual(project.themes, [themed.theme]);
    assert.deepEqual(project.windows, shop.windows);
    assert.deepEqual(project.huds, status.huds);
    assert.deepEqual(warnings, []);
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
    const wrong = fake({ "window/index.ts": { windows: [shop, theme({})] } });
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
        children: [raw.flex({ frame: texture("window:gui/frame.png", { insets: 2 }) })],
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
