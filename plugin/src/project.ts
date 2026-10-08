import { root } from "./ui/elements.ts";
import { resolveTokens } from "./ui/tokens.ts";
import type { Hud, Window, WindowDocument } from "./ui/document.ts";
import type { Art, TextFont } from "./ui/types.ts";

export interface WindowOptions {
    /** Namespace of the generated assets. Defaults to `window`. */
    namespace?: string;
    /** Emit HUD shader assets. */
    hudShaders?: boolean;
    /** A 110x16 catalog sprite that restyles every anvil's native text field; see docs/AUTHORING.md. */
    anvilFieldSprite?: string;
    /**
     * Experimental: let anvil input windows change their title at runtime. Each change reopens the anvil,
     * which can flicker and drop keystrokes while the player types.
     */
    experimentalAnvilUpdates?: boolean;
    /** Generate Kotlin bindings. */
    kotlin?: KotlinOptions;
}

/** Server API the generated Kotlin views and HUDs bind to. */
export type KotlinTarget = "agnostic" | "minestom" | "multistom";

const kotlinTargets: readonly KotlinTarget[] = ["agnostic", "minestom", "multistom"];

export interface KotlinOptions {
    /** Kotlin package of the generated bindings. */
    packageName: string;
    target: KotlinTarget;
}

/** Returns `target` if it is a known Kotlin target, otherwise throws naming the accepted values. */
export function kotlinTarget(target: unknown): KotlinTarget {
    const known = kotlinTargets.find((value) => value === target);
    if (known === undefined) {
        const got = target === undefined ? "it is missing" : `got ${JSON.stringify(target)}`;
        throw new Error(
            `Window kotlin.target must be one of ${kotlinTargets.map((value) => `"${value}"`).join(", ")}; ${got}.`,
        );
    }
    return known;
}

export interface SourceFile {
    path: string;
    contents: Uint8Array;
}

/** Generated text crosses the component boundary as a string; images as bytes. */
export type FileContents = { tag: "text"; val: string } | { tag: "binary"; val: Uint8Array };

export interface OutputFile {
    path: string;
    contents: FileContents;
}

export interface CompileOutput {
    files: OutputFile[];
    kotlinFiles: OutputFile[];
    warnings: string[];
}

export type Compile = (
    namespace: string,
    projectJson: string,
    files: SourceFile[],
    kotlin: KotlinOptions | undefined,
) => CompileOutput;

export interface ProjectJson {
    fonts?: Record<string, TextFont>;
    sprites?: Record<string, Art>;
    windows: Window[];
    huds: Hud[];
    options: { hud_shaders: boolean; anvil_field_sprite?: string; experimental_anvil_updates?: boolean };
    target: { pack_format: number };
}

/** The slice of the generator context the Window plugin uses. */
export interface WindowContext {
    readonly options: WindowOptions;
    readonly pack: { readonly format: { readonly min: number; readonly max: number } };
    discovered(name: string): readonly { readonly path: string; readonly module: Record<string, unknown> }[];
    sourceFiles(glob?: string): string[];
    read(path: string): Uint8Array | undefined;
    readSource(path: string): Uint8Array | undefined;
    remove(path: string): void;
    emit(path: string, contents: Uint8Array | string): void;
    emitOutput(root: string, path: string, contents: Uint8Array | string): void;
}

const DEFINITIONS = "definitions";
const JSX_DEFINITIONS = "jsx";
const ENTRIES = ["window/index.ts", "window/index.tsx"];
const TEXTURE_REFERENCE = /^([\w.-]+):(.+)$/s;

type Fields = Record<string, unknown>;

function isFields(value: unknown): value is Fields {
    return typeof value === "object" && value !== null;
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
    packFormat: number,
): ProjectJson {
    const fonts: Record<string, TextFont> = Object.create(null) as Record<string, TextFont>;
    const sprites: Record<string, Art> = Object.create(null) as Record<string, Art>;
    const windows: Window[] = [];
    const huds: Hud[] = [];
    for (const doc of documents) {
        for (const [name, font] of Object.entries(doc.fonts ?? {})) {
            if (Object.hasOwn(fonts, name)) {
                throw new Error(`fonts declare \`${name}\` twice`);
            }
            fonts[name] = font;
        }
        for (const [name, art] of Object.entries(doc.sprites ?? {})) {
            if (Object.hasOwn(sprites, name)) {
                throw new Error(`sprite catalog declares \`${name}\` twice`);
            }
            sprites[name] = resolveTokens(art);
        }
        windows.push(...(doc.windows ?? []).map(resolveTokens));
        huds.push(...(doc.huds ?? []).map(resolveTokens));
        if (doc.window !== undefined) {
            windows.push(resolveTokens(doc.window));
        }
        if (doc.hud !== undefined) {
            huds.push(resolveTokens(doc.hud));
        }
    }
    return {
        ...(Object.keys(fonts).length > 0 ? { fonts: { ...fonts } } : {}),
        ...(Object.keys(sprites).length > 0 ? { sprites: { ...sprites } } : {}),
        windows,
        huds,
        options: {
            hud_shaders: options.hudShaders === true,
            ...(options.anvilFieldSprite === undefined ? {} : { anvil_field_sprite: options.anvilFieldSprite }),
            ...(options.experimentalAnvilUpdates === true ? { experimental_anvil_updates: true } : {}),
        },
        target: { pack_format: packFormat },
    };
}

/** The module's default-exported document, or each document of a default-exported array (a JSX fragment). */
function defaultExport(path: string, module: Record<string, unknown>): WindowDocument[] {
    const value = module["default"];
    const docs = Array.isArray(value) ? value : [value];
    if (docs.length === 0 || !docs.every(isFields)) {
        throw new Error(`${path} must export default a Window document`);
    }
    return docs as WindowDocument[];
}

/** Per `defineWindows` list: the key of its documents and the key of one bare definition. */
const LISTS = {
    windows: ["windows", "window"],
    huds: ["huds", "hud"],
} as const;

/** Whether `entry` is a bare definition for `list`: a window has a `container`, a HUD a `name` and no `container`. */
function isDefinition(list: keyof typeof LISTS, entry: Fields): boolean {
    return typeof entry["name"] === "string" && "container" in entry === (list === "windows");
}

/** The documents of one `defineWindows` list, flattening nested lists such as a listed fragment. */
function listDocuments(path: string, list: keyof typeof LISTS, entries: unknown): WindowDocument[] {
    if (entries === undefined) {
        return [];
    }
    if (!Array.isArray(entries)) {
        throw new Error(`${path}: defineWindows \`${list}\` must be a list`);
    }
    const [documents, single] = LISTS[list];
    return (entries as unknown[]).flat(Infinity).map((listed, i) => {
        if (isFields(listed) && documents in listed) {
            return { [documents]: listed[documents] };
        }
        const entry =
            isFields(listed) && isDefinition(list, listed)
                ? root(listed as unknown as Window | Hud, `${single} \`${String(listed["name"])}\``)
                : resolveTokens(listed);
        const other = isFields(entry) && ["windows", "huds"].some((key) => key in entry);
        if (!isFields(entry) || other || !isDefinition(list, entry)) {
            throw new Error(`${path}: defineWindows \`${list}\` entry ${i} is not a ${single} document`);
        }
        return { [single]: entry };
    });
}

/** The documents of a `defineWindows` definition. */
function entryDocuments(path: string, value: unknown): WindowDocument[] {
    if (!isFields(value)) {
        throw new Error(`${path} must export default defineWindows({ windows, huds })`);
    }
    if (value["themes"] !== undefined) {
        throw new Error(
            `${path}: defineWindows no longer takes \`themes\`; use inline art, \`sprites\`, and \`fonts\``,
        );
    }
    const documents = (["windows", "huds"] as const).flatMap((key) => listDocuments(path, key, value[key]));
    const sprites = value["sprites"];
    if (sprites !== undefined) {
        if (!isFields(sprites) || Array.isArray(sprites)) {
            throw new Error(`${path}: defineWindows \`sprites\` must map names to art`);
        }
        documents.push({ sprites: resolveTokens(sprites) as Record<string, Art> });
    }
    const fonts = value["fonts"];
    if (fonts !== undefined) {
        if (!isFields(fonts) || Array.isArray(fonts)) {
            throw new Error(`${path}: defineWindows \`fonts\` must map names to fonts`);
        }
        documents.push({ fonts: fonts as Record<string, TextFont> });
    }
    return documents;
}

/** Every texture of inline art in `value`: an object with `art: "texture"` anywhere in the documents. */
function inlineTextures(value: unknown, out: string[]): string[] {
    if (Array.isArray(value)) {
        for (const item of value) {
            inlineTextures(item, out);
        }
    } else if (isFields(value)) {
        if (value["art"] === "texture" && typeof value["texture"] === "string") {
            out.push(value["texture"]);
        }
        for (const field of Object.values(value)) {
            inlineTextures(field, out);
        }
    }
    return out;
}

/**
 * The definition documents and the compiler's source files: the non-TypeScript files under `window/` plus every
 * texture id that art and fonts reference. Everything under `window/` leaves the pack.
 *
 * A `window/index.ts(x)` entry's `defineWindows` default export lists every document, and the other modules are
 * ordinary modules. Without one, every module under `window/` default-exports documents, read in path order, and
 * `warnings` asks for an entry.
 */
export function collectInputs(ctx: WindowContext): {
    documents: WindowDocument[];
    files: SourceFile[];
    warnings: string[];
} {
    const modules = [...ctx.discovered(DEFINITIONS), ...ctx.discovered(JSX_DEFINITIONS)].sort((a, b) =>
        a.path < b.path ? -1 : a.path > b.path ? 1 : 0,
    );
    const entries = modules.filter(({ path }) => ENTRIES.includes(path));
    if (entries.length > 1) {
        throw new Error("window/index.ts and window/index.tsx are both entries; keep one");
    }
    const warnings: string[] = [];
    let documents: WindowDocument[];
    if (entries.length === 1) {
        documents = entryDocuments(entries[0]!.path, entries[0]!.module["default"]);
    } else {
        documents = modules.flatMap(({ path, module }) => defaultExport(path, module));
        if (modules.length > 0) {
            warnings.push(
                "Window: no window/index.ts(x) entry; reading the default export of every file under window/. " +
                    "Default-export defineWindows({ windows, huds }) from window/index.ts(x) instead.",
            );
        }
    }
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
    const textures = documents.flatMap((doc) => [
        ...Object.values(doc.fonts ?? {}).map((font) => font.texture),
        ...inlineTextures(doc, []),
    ]);
    for (const texture of textures) {
        const path = resourceTexturePath(texture);
        if (path !== undefined) {
            addOnce(path);
        }
    }
    return { documents, files, warnings };
}

/** The newest format of the pack's range. rpp's version check ignores pre-release tags, so older hosts are rejected here. */
function packFormat(format: unknown): number {
    const max = isFields(format) ? format["max"] : undefined;
    if (typeof max !== "number") {
        throw new Error("Window requires rpp 0.1.0-nightly.20261004 or newer; upgrade rpp and set pack.format.");
    }
    return max;
}

/** Compile the project's definitions and write the pack files, warnings, and Kotlin bindings. */
export function generate(ctx: WindowContext, compile: Compile): void {
    const { documents, files, warnings } = collectInputs(ctx);
    for (const warning of warnings) {
        console.warn(warning);
    }
    if (documents.length === 0) {
        return;
    }
    const { options } = ctx;
    const kotlin =
        options.kotlin === undefined
            ? undefined
            : { packageName: options.kotlin.packageName, target: kotlinTarget(options.kotlin.target) };
    const project = buildProject(documents, options, packFormat(ctx.pack.format));
    const output = compile(options.namespace ?? "window", JSON.stringify(project), files, kotlin);
    for (const file of output.files) {
        ctx.emit(file.path, file.contents.val);
    }
    for (const warning of output.warnings) {
        console.warn(warning);
    }
    if (kotlin !== undefined) {
        for (const file of output.kotlinFiles) {
            ctx.emitOutput("kotlin", file.path, file.contents.val);
        }
    }
}
