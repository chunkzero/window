import type { Hud, Theme, Window, WindowDocument } from "./authoring/types.ts";

export interface WindowOptions {
    namespace?: string;
    hudShaders?: boolean;
    nativeAnvilInput?: string;
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
    options: { hud_shaders: boolean; native_anvil_input?: string };
    target: { pack_format?: number };
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

/** A `min_format`/`max_format` value: a major version or a `[major, minor]` pair. */
function formatVersionField(value: Fields, key: string): number | undefined {
    const field = value[key];
    if (Array.isArray(field)) {
        return typeof field[0] === "number" ? field[0] : undefined;
    }
    return typeof field === "number" ? field : undefined;
}

function packFormatValue(value: unknown): number | undefined {
    if (typeof value === "number") {
        return value;
    }
    if (!isFields(value)) {
        return undefined;
    }
    return (
        numberField(value, "pack_format") ??
        numberField(value, "format") ??
        formatVersionField(value, "max_format") ??
        numberField(value, "max_inclusive") ??
        numberField(value, "1") ??
        formatVersionField(value, "min_format") ??
        numberField(value, "min_inclusive") ??
        numberField(value, "0")
    );
}

function packFormatFromMcmeta(text: string | undefined): number | undefined {
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
    return packFormatValue(pack["pack_format"]) ?? packFormatValue(pack["supported_formats"]) ?? packFormatValue(pack);
}

/** The configured pack format, else the one declared by `pack.mcmeta`. */
export function detectPackFormat(pack: { readonly format?: number }, mcmeta: string | undefined): number | undefined {
    return pack.format ?? packFormatFromMcmeta(mcmeta);
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
    packFormat: number | undefined,
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
            ...(options.nativeAnvilInput === undefined ? {} : { native_anvil_input: options.nativeAnvilInput }),
        },
        target: packFormat === undefined ? {} : { pack_format: packFormat },
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
    const packFormat = detectPackFormat(ctx.pack, ctx.readSourceText("pack.mcmeta"));
    const project = buildProject(documents, options, packFormat);
    const output = compile(options.namespace ?? "window", JSON.stringify(project), files, options.kotlinPackage);
    for (const file of output.files) {
        ctx.emit(file.path, file.contents);
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
