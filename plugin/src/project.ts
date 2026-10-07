import type { Hud, Theme, Window, WindowDocument } from "./authoring/types.ts";

export interface WindowOptions {
    /** Namespace of the generated assets. Defaults to `window`. */
    namespace?: string;
    /** Emit HUD shader assets. */
    hudShaders?: boolean;
    /** Outline hovered buttons with generated core text shaders. */
    hoverOutlines?: boolean;
    /** A 110x16 theme sprite that restyles every anvil's native text field; see docs/AUTHORING.md. */
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
    themes?: Theme[];
    windows: Window[];
    huds: Hud[];
    options: {
        hud_shaders: boolean;
        hover_outlines?: boolean;
        anvil_field_sprite?: string;
        experimental_anvil_updates?: boolean;
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
            ...(options.hoverOutlines === true ? { hover_outlines: true } : {}),
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

/**
 * The definition documents in path order and the compiler's source files: the non-TypeScript files
 * under `window/` plus every texture the themes reference. Everything under `window/` leaves the pack.
 */
export function collectInputs(ctx: WindowContext): { documents: WindowDocument[]; files: SourceFile[] } {
    const modules = [...ctx.discovered(DEFINITIONS), ...ctx.discovered(JSX_DEFINITIONS)].sort((a, b) =>
        a.path < b.path ? -1 : a.path > b.path ? 1 : 0,
    );
    const documents = modules.flatMap(({ path, module }) => defaultExport(path, module));
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
    const { documents, files } = collectInputs(ctx);
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
