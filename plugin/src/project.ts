import type { Hud, Theme, Window, WindowDocument } from "./authoring/types.ts";

export interface WindowOptions {
    /** Namespace of the generated assets. Defaults to `window`. */
    namespace?: string;
    /** Emit HUD shader assets. */
    hudShaders?: boolean;
    /** A 110x16 theme sprite that restyles every anvil's native text field; see docs/AUTHORING.md. */
    anvilFieldSprite?: string;
    /**
     * Experimental: let anvil input windows change their title at runtime. Each change reopens the anvil,
     * which can flicker and drop keystrokes while the player types.
     */
    experimentalAnvilUpdates?: boolean;
    /** Kotlin package of the generated bindings. */
    kotlinPackage?: string;
}

export interface SourceFile {
    path: string;
    contents: Uint8Array;
}

export interface CompileOutput {
    files: SourceFile[];
    kotlinFiles: SourceFile[];
    warnings: string[];
    /** JSON objects to append to `pack.mcmeta` `overlays.entries`. */
    packOverlays: string[];
}

export type Compile = (
    namespace: string,
    projectJson: string,
    files: SourceFile[],
    kotlinPackage: string | undefined,
) => CompileOutput;

export interface ProjectJson {
    themes?: Theme[];
    windows: Window[];
    huds: Hud[];
    options: { hud_shaders: boolean; anvil_field_sprite?: string; experimental_anvil_updates?: boolean };
    target: Partial<FormatRange>;
}

/** A `min_format`/`max_format` value: a major version or a `[major, minor]` pair. */
export type FormatVersion = number | [number, number];

export interface FormatRange {
    min_format: FormatVersion;
    max_format: FormatVersion;
}

/** The slice of the generator context the Window plugin uses. */
export interface WindowContext {
    readonly options: WindowOptions;
    readonly pack: { readonly format?: number };
    discovered(name: string): readonly { readonly path: string; readonly module: Record<string, unknown> }[];
    sourceFiles(glob?: string): string[];
    read(path: string): Uint8Array | undefined;
    readSource(path: string): Uint8Array | undefined;
    readSourceText(path: string): string | undefined;
    remove(path: string): void;
    emit(path: string, contents: Uint8Array): void;
    emitOutput(root: string, path: string, contents: Uint8Array): void;
}

const DEFINITIONS = "definitions";
const TEXTURE_REFERENCE = /^([\w.-]+):(.+)$/s;

type Fields = Record<string, unknown>;

function isFields(value: unknown): value is Fields {
    return typeof value === "object" && value !== null;
}

function numberField(value: Fields, key: string): number | undefined {
    const field = value[key];
    return typeof field === "number" ? field : undefined;
}

function formatVersion(value: unknown): FormatVersion | undefined {
    if (typeof value === "number") {
        return value;
    }
    if (Array.isArray(value) && typeof value[0] === "number") {
        return typeof value[1] === "number" ? [value[0], value[1]] : value[0];
    }
    return undefined;
}

/** A legacy `supported_formats` value: `n`, `[min, max]`, or `{ min_inclusive, max_inclusive }`. */
function legacyRange(value: unknown): FormatRange | undefined {
    if (typeof value === "number") {
        return { min_format: value, max_format: value };
    }
    const [min, max] = Array.isArray(value)
        ? value
        : isFields(value)
          ? [value["min_inclusive"], value["max_inclusive"]]
          : [];
    return typeof min === "number" && typeof max === "number" ? { min_format: min, max_format: max } : undefined;
}

function rangeFromMcmeta(text: string | undefined): FormatRange | undefined {
    if (text === undefined) {
        return undefined;
    }
    let data: unknown;
    try {
        data = JSON.parse(text);
    } catch {
        return undefined;
    }
    if (!isFields(data) || !isFields(data["pack"])) {
        return undefined;
    }
    const pack = data["pack"];
    const legacy = legacyRange(pack["supported_formats"]) ?? legacyRange(numberField(pack, "pack_format"));
    const min = formatVersion(pack["min_format"]) ?? legacy?.min_format;
    const max = formatVersion(pack["max_format"]) ?? legacy?.max_format;
    const either = min ?? max;
    return either === undefined ? undefined : { min_format: min ?? either, max_format: max ?? either };
}

/** The format range `pack.mcmeta` declares, else the configured pack format. */
export function detectPackFormats(
    pack: { readonly format?: number },
    mcmeta: string | undefined,
): FormatRange | undefined {
    return (
        rangeFromMcmeta(mcmeta) ??
        (pack.format === undefined ? undefined : { min_format: pack.format, max_format: pack.format })
    );
}

/** `pack.mcmeta` with `entries` appended to its overlays, keeping the entries it already declares. */
export function addPackOverlays(mcmeta: string | undefined, entries: readonly string[]): string {
    const data: unknown = mcmeta === undefined ? undefined : JSON.parse(mcmeta);
    if (!isFields(data)) {
        throw new Error("HUD shader overlays need a pack.mcmeta object to declare them in");
    }
    const overlays = isFields(data["overlays"]) ? data["overlays"] : {};
    const existing: unknown[] = Array.isArray(overlays["entries"]) ? overlays["entries"] : [];
    const added = entries.map((entry) => JSON.parse(entry) as Fields);
    const taken = new Set(existing.map((entry) => (isFields(entry) ? entry["directory"] : undefined)));
    const clash = added.find((entry) => taken.has(entry["directory"]));
    if (clash !== undefined) {
        throw new Error(
            `pack.mcmeta already declares an overlay in \`${String(clash["directory"])}\`, which Window uses for HUD ` +
                "shaders; rename that overlay directory",
        );
    }
    data["overlays"] = { ...overlays, entries: [...existing, ...added] };
    return `${JSON.stringify(data, null, 4)}\n`;
}

/** The pack path of a `namespace:path` texture reference, or undefined for other values. */
export function resourceTexturePath(texture: unknown): string | undefined {
    if (typeof texture !== "string") {
        return undefined;
    }
    const match = TEXTURE_REFERENCE.exec(texture);
    if (match === null) {
        return undefined;
    }
    const [, namespace, path] = match;
    return `assets/${namespace}/textures/${path!.endsWith(".png") ? path : `${path}.png`}`;
}

export function buildProject(
    documents: readonly WindowDocument[],
    options: WindowOptions,
    formats: FormatRange | undefined,
): ProjectJson {
    const themes: Theme[] = [];
    const windows: Window[] = [];
    const huds: Hud[] = [];
    for (const doc of documents) {
        if (doc.theme !== undefined) {
            themes.push(doc.theme);
        }
        windows.push(...(doc.windows ?? []));
        huds.push(...(doc.huds ?? []));
        if (doc.window !== undefined) {
            windows.push(doc.window);
        }
        if (doc.hud !== undefined) {
            huds.push(doc.hud);
        }
    }
    return {
        ...(themes.length > 0 ? { themes } : {}),
        windows,
        huds,
        options: {
            hud_shaders: options.hudShaders === true,
            ...(options.anvilFieldSprite === undefined ? {} : { anvil_field_sprite: options.anvilFieldSprite }),
            ...(options.experimentalAnvilUpdates === true ? { experimental_anvil_updates: true } : {}),
        },
        target: formats ?? {},
    };
}

function defaultExport(path: string, module: Record<string, unknown>): WindowDocument {
    const doc = module["default"];
    if (!isFields(doc)) {
        throw new Error(`${path} must export default a Window document`);
    }
    return doc as WindowDocument;
}

/**
 * The definition documents in path order and the compiler's source files: the non-TypeScript files
 * under `window/` plus every texture the themes reference. Everything under `window/` leaves the pack.
 */
export function collectInputs(ctx: WindowContext): { documents: WindowDocument[]; files: SourceFile[] } {
    const modules = [...ctx.discovered(DEFINITIONS)].sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
    const documents = modules.map(({ path, module }) => defaultExport(path, module));
    const files: SourceFile[] = [];
    const seen = new Set<string>();
    const addOnce = (path: string): void => {
        if (seen.has(path)) {
            return;
        }
        const contents = ctx.read(path) ?? ctx.readSource(path);
        if (contents !== undefined) {
            seen.add(path);
            files.push({ path, contents });
        }
    };

    for (const path of ctx.sourceFiles("window/**")) {
        addOnce(path);
        ctx.remove(path);
    }
    for (const doc of documents) {
        for (const sprite of Object.values(doc.theme?.sprites ?? {})) {
            const path = resourceTexturePath(sprite.texture);
            if (path !== undefined) {
                addOnce(path);
            }
        }
    }
    return { documents, files };
}

/** Compile the project's definitions and write the pack files, warnings, and Kotlin bindings. */
export function generate(ctx: WindowContext, compile: Compile): void {
    const { documents, files } = collectInputs(ctx);
    if (documents.length === 0) {
        return;
    }
    const { options } = ctx;
    const formats = detectPackFormats(ctx.pack, ctx.readSourceText("pack.mcmeta"));
    const project = buildProject(documents, options, formats);
    const output = compile(options.namespace ?? "window", JSON.stringify(project), files, options.kotlinPackage);
    for (const file of output.files) {
        ctx.emit(file.path, file.contents);
    }
    if (output.packOverlays.length > 0) {
        const mcmeta = ctx.read("pack.mcmeta");
        const text = mcmeta === undefined ? undefined : new TextDecoder().decode(mcmeta);
        ctx.emit("pack.mcmeta", new TextEncoder().encode(addPackOverlays(text, output.packOverlays)));
    }
    for (const warning of output.warnings) {
        console.warn(warning);
    }
    if (options.kotlinPackage !== undefined) {
        for (const file of output.kotlinFiles) {
            ctx.emitOutput("kotlin", file.path, file.contents);
        }
    }
}
