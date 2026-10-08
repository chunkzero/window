import type { ArtName, GeneratedStyle, Insets, ShapeArt, TextureArt } from "./types.ts";

export interface TextureOptions extends ArtName {
    /** Nine-slice inset pixels, used where the art stretches over a box. */
    insets?: Insets;
    /** Size drawn as an image; set with `height`, or omit both for the texture's own size. */
    width?: number;
    height?: number;
}

/** Generated art's style and, for images, its size. */
export interface ShapeStyle extends GeneratedStyle {
    width?: number;
    height?: number;
}

function defined<T extends object>(fields: T): T {
    return Object.fromEntries(Object.entries(fields).filter(([, value]) => value !== undefined)) as T;
}

/** Inline bitmap art from a pack-source texture path or a texture id, optionally nine-sliced with `insets`. */
export function texture(path: string, options: TextureOptions = {}): TextureArt {
    if (typeof path !== "string" || path === "") {
        throw new Error("texture() requires a texture path");
    }
    if ((options.width === undefined) !== (options.height === undefined)) {
        throw new Error(`texture(${JSON.stringify(path)}) sets \`width\` and \`height\` together, or neither`);
    }
    return Object.freeze(defined({ art: "texture" as const, texture: path, ...options }));
}

/** Inline generated art: a frame over its box, or an image of `width` x `height`. */
export function shape(style: ShapeStyle, options: ArtName = {}): ShapeArt {
    if ("texture" in style) {
        throw new Error("shape() draws generated art; use texture() for bitmaps");
    }
    return Object.freeze(defined({ ...style, ...options, art: "shape" as const }));
}
