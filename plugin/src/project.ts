import { root } from "./ui/elements.ts";
import { resolveTokens } from "./ui/tokens.ts";
import type { Hud, Window, WindowDocument } from "./ui/document.ts";
import type { Preview } from "./ui/previews.ts";
import type { Art, TextFont } from "./ui/types.ts";

/** Core shader overrides Window generates. Each is opt-in and client-version-sensitive. */
export interface CoreShaderOptions {
    /** Relocate `<Hud anchor>` HUDs with generated core text shaders. */
    hud?: boolean;
}

/**
 * How a vanilla texture is treated: a string names a catalog sprite that replaces it, `false` replaces it
 * with a transparent texture, and `true` or omitted leaves vanilla's alone.
 */
export type VanillaTexture = string | boolean;

export interface VanillaAnvilOptions {
    /** A 110x16 catalog sprite or `false` for both of the anvil's native text-field sprites. */
    field?: VanillaTexture;
    /**
     * The anvil background (256x256) and its missing-result error icon. Omitted: both are blanked exactly when
     * some window has an anvil `<Input>`. `true` keeps vanilla's even then; a string also blanks the error icon.
     */
    background?: VanillaTexture;
}

export const hudGroups = ["hotbar", "health", "food", "armor", "air", "experience", "crosshair"] as const;

/** `false` blanks every vanilla GUI sprite of the group. Needs pack format 18 or newer. */
export type VanillaHudOptions = { [Group in (typeof hudGroups)[number]]?: boolean };

export interface VanillaOptions {
    anvil?: VanillaAnvilOptions;
    hud?: VanillaHudOptions;
}

export interface ExperimentalOptions {
    /**
     * Let anvil input windows change their title at runtime. Each change reopens the anvil, which can flicker
     * and drop keystrokes while the player types.
     */
    anvilUpdates?: boolean;
}

export interface SizeOptions {
    /** Print a breakdown of the compiler's output after a successful build. */
    report?: boolean;
    /** Fail the build when the compiler's output exceeds this many bytes. A positive integer; omitted means no limit. */
    budget?: number;
}

export interface PreviewsOptions {
    /** Directory the `defineWindows({ previews })` PNGs are written to, relative to the project root. */
    output: string;
}

export interface WindowOptions {
    /** Namespace of the generated assets. Defaults to `window`. */
    namespace?: string;
    coreShaders?: CoreShaderOptions;
    /** Overrides for vanilla textures; see docs/AUTHORING.md. */
    vanilla?: VanillaOptions;
    experimental?: ExperimentalOptions;
    /** Output size report and budget; see docs/AUTHORING.md. */
    size?: SizeOptions;
    /** Render `defineWindows({ previews })`; see docs/AUTHORING.md. */
    previews?: PreviewsOptions;
    /** Generate Kotlin bindings. */
    kotlin?: KotlinOptions;
}

type LeafKind = "boolean" | "string" | "string or boolean" | "positive integer";

/** An option object: each key maps to its leaf kind or, for a nested object, its own keys. */
interface OptionTree {
    readonly [key: string]: LeafKind | OptionTree;
}

const OPTION_TREE: OptionTree = {
    namespace: "string",
    coreShaders: { hud: "boolean" },
    vanilla: {
        anvil: { field: "string or boolean", background: "string or boolean" },
        hud: Object.fromEntries(hudGroups.map((group) => [group, "boolean"])),
    },
    experimental: { anvilUpdates: "boolean" },
    size: { report: "boolean", budget: "positive integer" },
    previews: { output: "string" },
};

/** The `kotlin` keys of `window()`'s config. */
const AUTHORED_KOTLIN: OptionTree = { package: "string", output: "string", target: "string" };

/** The `kotlin` keys of the plugin options `window()` hands to the generator. */
const PLUGIN_KOTLIN: OptionTree = { packageName: "string", target: "string" };

function hasKind(kind: LeafKind, value: unknown): boolean {
    if (kind === "positive integer") {
        return typeof value === "number" && Number.isSafeInteger(value) && value > 0;
    }
    return (kind !== "string" && typeof value === "boolean") || (kind !== "boolean" && typeof value === "string");
}

function checkTree(config: unknown, tree: OptionTree, path: string): void {
    if (typeof config !== "object" || config === null || Array.isArray(config)) {
        throw new Error(`Window option \`${path}\` must be an object`);
    }
    const prefix = path === "" ? "" : `${path}.`;
    for (const [key, value] of Object.entries(config)) {
        const child = Object.hasOwn(tree, key) ? tree[key] : undefined;
        if (child === undefined) {
            const scope = path === "" ? "" : ` in \`${path}\``;
            throw new Error(
                `Window option \`${prefix}${key}\` is unknown; accepted keys${scope}: ${Object.keys(tree).join(", ")}`,
            );
        }
        if (value === undefined) {
            continue;
        }
        if (typeof child === "string") {
            if (!hasKind(child, value)) {
                const got = JSON.stringify(value);
                throw new Error(`Window option \`${prefix}${key}\` must be a ${child}; got ${got}`);
            }
        } else {
            checkTree(value, child, `${prefix}${key}`);
        }
    }
}

/**
 * Throws naming the first plugin option in `config` that is unknown or has the wrong type. `config` is either
 * `window()`'s authored config or, with `kotlin: "plugin"`, the options the generator receives.
 */
export function checkOptions(config: unknown, kotlin: "authored" | "plugin" = "authored"): void {
    checkTree(config, { ...OPTION_TREE, kotlin: kotlin === "authored" ? AUTHORED_KOTLIN : PLUGIN_KOTLIN }, "");
    const previews = (config as WindowOptions).previews;
    if (previews !== undefined && previews.output === undefined) {
        throw new Error("Window option `previews.output` is required when `previews` is set");
    }
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
    /** The size breakdown, present when `size.report` is set. */
    sizeReport?: string;
}

export type Compile = (
    namespace: string,
    projectJson: string,
    files: SourceFile[],
    kotlin: KotlinOptions | undefined,
) => CompileOutput;

/** Renders the JSON list of previews from the project `Compile` takes, as `<name>.png` files. */
export type RenderPreviews = (projectJson: string, files: SourceFile[], previewsJson: string) => OutputFile[];

export interface ProjectJson {
    fonts?: Record<string, TextFont>;
    sprites?: Record<string, Art>;
    windows: Window[];
    huds: Hud[];
    options: {
        core_shaders: { hud: boolean };
        vanilla?: VanillaOptions;
        experimental?: { anvil_updates: true };
        size?: { report: boolean; budget?: number };
    };
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
            core_shaders: { hud: options.coreShaders?.hud === true },
            ...(options.vanilla === undefined ? {} : { vanilla: options.vanilla }),
            ...(options.experimental?.anvilUpdates === true ? { experimental: { anvil_updates: true } } : {}),
            ...(options.size === undefined
                ? {}
                : {
                      size: {
                          report: options.size.report === true,
                          ...(options.size.budget === undefined ? {} : { budget: options.size.budget }),
                      },
                  }),
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

/** The previews of a `defineWindows` definition. */
function entryPreviews(path: string, value: unknown): Preview[] {
    const previews = isFields(value) ? value["previews"] : undefined;
    if (previews === undefined) {
        return [];
    }
    if (!Array.isArray(previews) || !previews.every((entry) => isFields(entry) && isFields(entry["target"]))) {
        throw new Error(`${path}: defineWindows \`previews\` must be a list of preview(...)`);
    }
    return previews as Preview[];
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
 * `warnings` asks for an entry. Only an entry declares `previews`.
 */
export function collectInputs(ctx: WindowContext): {
    documents: WindowDocument[];
    previews: Preview[];
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
    let previews: Preview[] = [];
    if (entries.length === 1) {
        documents = entryDocuments(entries[0]!.path, entries[0]!.module["default"]);
        previews = entryPreviews(entries[0]!.path, entries[0]!.module["default"]);
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
    return { documents, previews, files, warnings };
}

/** The newest format of the pack's range. rpp's version check ignores pre-release tags, so older hosts are rejected here. */
function packFormat(format: unknown): number {
    const max = isFields(format) ? format["max"] : undefined;
    if (typeof max !== "number") {
        throw new Error("Window requires rpp 0.1.0-nightly.20261004 or newer; upgrade rpp and set pack.format.");
    }
    return max;
}

/** Whether `vanilla` blanks or replaces some texture, which needs a build even without definitions. */
function replacesVanillaTextures(vanilla: VanillaOptions | undefined): boolean {
    const values = [...Object.values(vanilla?.anvil ?? {}), ...Object.values(vanilla?.hud ?? {})];
    return values.some((value) => value === false || typeof value === "string");
}

/**
 * Compile the project's definitions and write the pack files, warnings, Kotlin bindings, and previews. Previews are
 * rendered only with `previews.output` set; declared without it, they warn and write nothing.
 */
export function generate(ctx: WindowContext, compile: Compile, renderPreviews?: RenderPreviews): void {
    const { documents, previews, files, warnings } = collectInputs(ctx);
    for (const warning of warnings) {
        console.warn(warning);
    }
    const { options } = ctx;
    checkOptions(options, "plugin");
    if (documents.length === 0 && !replacesVanillaTextures(options.vanilla)) {
        return;
    }
    const kotlin =
        options.kotlin === undefined
            ? undefined
            : { packageName: options.kotlin.packageName, target: kotlinTarget(options.kotlin.target) };
    const project = buildProject(documents, options, packFormat(ctx.pack.format));
    const projectJson = JSON.stringify(project);
    const output = compile(options.namespace ?? "window", projectJson, files, kotlin);
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
    if (output.sizeReport !== undefined) {
        console.log(output.sizeReport);
    }
    if (previews.length === 0) {
        return;
    }
    if (options.previews === undefined) {
        console.warn(
            `Window: ${previews.length} previews are declared, but \`previews.output\` is not set; none is rendered.`,
        );
        return;
    }
    if (renderPreviews === undefined) {
        throw new Error("Window: the compiler component cannot render previews; rebuild plugin/window.wasm");
    }
    for (const file of renderPreviews(projectJson, files, JSON.stringify(previews))) {
        ctx.emitOutput("previews", file.path, file.contents.val);
    }
}
