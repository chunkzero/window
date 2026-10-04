import assert from "node:assert/strict";
import { test } from "node:test";

import { hud, theme, ui } from "../src/authoring/index.ts";
import type { WindowDocument } from "../src/authoring/types.ts";
import {
    addPackOverlays,
    buildProject,
    collectInputs,
    detectPackFormats,
    generate,
    resourceTexturePath,
} from "../src/project.ts";
import type { CompileOutput, SourceFile, WindowContext, WindowOptions } from "../src/project.ts";

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
    format?: number,
): Fake {
    const emitted = new Map<string, Uint8Array>();
    const outputs: [string, string][] = [];
    const removed: string[] = [];
    const ctx: WindowContext = {
        options,
        pack: format === undefined ? {} : { format },
        discovered: () => Object.entries(modules).map(([path, doc]) => ({ path, module: { default: doc } })),
        sourceFiles: () => Object.keys(sources).filter((path) => path.startsWith("window/")),
        read: (path) => emitted.get(path),
        readSource: (path) => (sources[path] === undefined ? undefined : bytes(sources[path])),
        readSourceText: (path) => sources[path],
        remove: (path) => removed.push(path),
        emit: (path, contents) => emitted.set(path, contents),
        emitOutput: (root, path) => outputs.push([root, path]),
    };
    return { ctx, emitted, outputs, removed };
}

const shop = ui({ name: "shop", container: "generic_9x6" });
const status = hud({ name: "status", width: 10, height: 10 });

test("documents are collected in path order", () => {
    const { ctx } = fake({ "window/b.ts": status, "window/a.ts": shop });
    const { documents } = collectInputs(ctx);
    assert.deepEqual(buildProject(documents, {}, undefined).windows, shop.windows);
    assert.equal(documents[0], shop);
    assert.equal(documents[1], status);
});

test("a discovered module without a default document fails naming its path", () => {
    const { ctx } = fake({ "window/a.ts": undefined });
    assert.throws(() => collectInputs(ctx), /window\/a\.ts must export default/);
});

test("pack formats keep the declared mcmeta range, then fall back to the context", () => {
    const mcmeta = (pack: unknown): string => JSON.stringify({ pack });
    const range = (min_format: unknown, max_format: unknown) => ({ min_format, max_format });
    assert.deepEqual(detectPackFormats({ format: 84 }, mcmeta({ pack_format: 84 })), range(84, 84));
    assert.deepEqual(detectPackFormats({}, mcmeta({ pack_format: 84, min_format: 1 })), range(1, 84));
    assert.deepEqual(
        detectPackFormats({}, mcmeta({ supported_formats: { min_inclusive: 3, max_inclusive: 9 } })),
        range(3, 9),
    );
    assert.deepEqual(detectPackFormats({}, mcmeta({ pack_format: 34, supported_formats: [18, 34] })), range(18, 34));
    assert.deepEqual(detectPackFormats({}, mcmeta({ min_format: 5 })), range(5, 5));
    assert.deepEqual(
        detectPackFormats({}, mcmeta({ min_format: [84, 0], max_format: [88, 1] })),
        range([84, 0], [88, 1]),
    );
    assert.deepEqual(detectPackFormats({ format: 7 }, "not json"), range(7, 7));
    assert.equal(detectPackFormats({}, undefined), undefined);
});

test("pack overlays are appended after the declared ones", () => {
    const mine = { directory: "mine", min_format: 84, max_format: 84 };
    const pack = { pack: { pack_format: 84 }, overlays: { entries: [mine] } };
    const window = { directory: "window_hud_85_88", min_format: 85, max_format: 88 };

    assert.deepEqual(JSON.parse(addPackOverlays(JSON.stringify(pack), [JSON.stringify(window)])), {
        ...pack,
        overlays: { entries: [mine, window] },
    });
    assert.throws(
        () => addPackOverlays(JSON.stringify(pack), [JSON.stringify({ ...window, directory: "mine" })]),
        /already declares an overlay in `mine`/,
    );
});

test("project JSON omits empty themes and an unknown pack format", () => {
    assert.deepEqual(JSON.parse(JSON.stringify(buildProject([shop], {}, undefined))), {
        windows: shop.windows,
        huds: [],
        options: { hud_shaders: false },
        target: {},
    });
    const themed: WindowDocument = theme({ sprites: { badge: { kind: "badge", width: 1, height: 1 } } });
    const project = buildProject(
        [themed, { ...status, window: shop.windows[0]! }],
        { hudShaders: true },
        {
            min_format: 84,
            max_format: [88, 1],
        },
    );
    assert.equal(project.themes?.length, 1);
    assert.deepEqual(project.windows, shop.windows);
    assert.deepEqual(project.huds, status.huds);
    assert.deepEqual(project.options, { hud_shaders: true });
    assert.deepEqual(project.target, { min_format: 84, max_format: [88, 1] });
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
    const { ctx, removed } = fake({ "window/a.ts": shop }, { "window/badge.png": "png", "pack.mcmeta": "{}" });
    assert.deepEqual(
        collectInputs(ctx).files.map((file) => file.path),
        ["window/badge.png"],
    );
    assert.deepEqual(removed, ["window/badge.png"]);
});

test("compilation is skipped without definitions", () => {
    const { ctx, removed } = fake({}, { "window/badge.png": "png" });
    generate(ctx, () => assert.fail("compile must not run"));
    assert.deepEqual(removed, ["window/badge.png"]);
});

test("Kotlin files are emitted only with a package", () => {
    const output: CompileOutput = {
        files: [{ path: "assets/window/a.json", contents: bytes("{}") }],
        kotlinFiles: [{ path: "A.kt", contents: bytes("class A") }],
        warnings: [],
        packOverlays: [],
    };
    const calls: (string | undefined)[] = [];
    const compile = (_namespace: string, _json: string, _files: SourceFile[], kotlinPackage: string | undefined) => {
        calls.push(kotlinPackage);
        return output;
    };

    const without = fake({ "window/a.ts": shop });
    generate(without.ctx, compile);
    assert.deepEqual([...without.emitted.keys()], ["assets/window/a.json"]);
    assert.deepEqual(without.outputs, []);

    const withPackage = fake({ "window/a.ts": shop }, {}, { kotlinPackage: "dev.example" });
    generate(withPackage.ctx, compile);
    assert.deepEqual(withPackage.outputs, [["kotlin", "A.kt"]]);
    assert.deepEqual(calls, [undefined, "dev.example"]);
});
