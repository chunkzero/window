/** Build-time tokens: vars, themes, and values derived from them. See docs/AUTHORING.md. */

/** A `#rrggbb` hex color. */
export type Color = `#${string}`;

declare const resolved: unique symbol;

/**
 * A value resolved while a window or HUD is built, under the nearest `theme`: a `defineVars` var or a `derive`d
 * value. Wherever it is used, the compiler sees only the resolved value.
 */
export interface Var<T> {
    /** Type-only: the value this resolves to. */
    readonly [resolved]: T;
}

/** A value, or a var resolving to one. */
export type Token<T> = T | Var<T>;

/** Resolves a token under the active theme. */
export type Get = <T>(token: Token<T>) => T;

/** The value types a var may hold. */
export type VarValue = Color | number;

/** The vars `defineVars(defaults)` returns, typed by their defaults. */
export type Vars<T extends Record<string, VarValue>> = {
    readonly [K in keyof T]: Var<T[K] extends number ? number : Color>;
};

/** Each var's override in a theme, typed by the var. */
export type ThemeOverrides<V> = { readonly [K in keyof V]?: V[K] extends Var<infer T> ? T : never };

const VAR: unique symbol = Symbol("window.var");
const THEME: unique symbol = Symbol("window.theme");

/** A theme from `createTheme`: overrides of some vars for the subtree of the `Window`, `Hud`, or `Box` taking it. */
export interface VarTheme {
    readonly [THEME]: ReadonlyMap<VarData, unknown>;
}

interface VarData {
    readonly name: string;
    readonly fallback?: unknown;
    readonly derive?: (get: Get) => unknown;
}

type Scope = ReadonlyMap<VarData, unknown>;

const COLOR = /^#[0-9a-fA-F]{6}$/;

function check(value: unknown, like: unknown, label: string): void {
    const ok = typeof like === "number" ? typeof value === "number" && Number.isFinite(value) : isColor(value);
    if (!ok) {
        const kind = typeof like === "number" ? "a number" : 'a "#rrggbb" color';
        throw new Error(`${label} must be ${kind}; got ${JSON.stringify(value)}`);
    }
}

function isColor(value: unknown): value is Color {
    return typeof value === "string" && COLOR.test(value);
}

function token<T>(data: VarData): Var<T> {
    return Object.freeze({
        [VAR]: data,
        toJSON(): never {
            throw new Error(`var \`${data.name}\` was not resolved; use it inside a Window or Hud`);
        },
    }) as unknown as Var<T>;
}

function varData(value: unknown): VarData | undefined {
    return typeof value === "object" && value !== null ? (value as { [VAR]?: VarData })[VAR] : undefined;
}

/**
 * Declares vars with their default values: `#rrggbb` colors or numbers. Use them in styles, props, and art; a
 * `createTheme` theme overrides them for a subtree.
 */
export function defineVars<const T extends Record<string, VarValue>>(defaults: T): Vars<T> {
    const out: Record<string, Var<unknown>> = {};
    for (const [name, value] of Object.entries(defaults)) {
        check(value, typeof value === "number" ? 0 : "", `var \`${name}\``);
        out[name] = token({ name, fallback: value });
    }
    return Object.freeze(out) as Vars<T>;
}

/** A theme overriding some of `vars`; pass it to the `theme` prop of a `Window`, `Hud`, or `Box`. */
export function createTheme<V extends Record<string, Var<Color> | Var<number>>, O extends ThemeOverrides<V>>(
    vars: V,
    overrides: O & { readonly [K in keyof O]: K extends keyof V ? O[K] : never },
): VarTheme {
    const values = new Map<VarData, unknown>();
    for (const [name, value] of Object.entries(overrides)) {
        const data = Object.hasOwn(vars, name) ? varData(vars[name]) : undefined;
        if (data === undefined || data.derive !== undefined) {
            throw new Error(`createTheme overrides \`${name}\`, which is not one of its vars`);
        }
        check(value, data.fallback, `theme value \`${name}\``);
        values.set(data, value);
    }
    return Object.freeze({ [THEME]: values });
}

function isTheme(value: unknown): value is VarTheme {
    return typeof value === "object" && value !== null && THEME in value;
}

/**
 * A value computed from resolved tokens, such as art whose bevel colors derive from a fill var. `compute` runs once
 * per use, with `get` resolving tokens under the theme active there.
 */
export function derive<T>(compute: (get: Get) => T): Var<T> {
    if (typeof compute !== "function") {
        throw new Error("derive() takes a function");
    }
    return token({ name: "derived", derive: compute });
}

/** Mixes `from` toward `to` by `amount` (0 to 1) per RGB channel. */
export function mix(from: Color, to: Color, amount: number): Color {
    check(from, "", "mix `from`");
    check(to, "", "mix `to`");
    const channel = (value: string, at: number) => parseInt(value.slice(at, at + 2), 16);
    let out = "#";
    for (const at of [1, 3, 5]) {
        const value = Math.round(channel(from, at) + (channel(to, at) - channel(from, at)) * amount);
        out += Math.min(255, Math.max(0, value)).toString(16).padStart(2, "0");
    }
    return out as Color;
}

function resolve(value: unknown, scope: Scope): unknown {
    if (typeof value !== "object" || value === null) {
        return value;
    }
    const data = varData(value);
    if (data !== undefined) {
        if (data.derive !== undefined) {
            return resolve(
                data.derive((t) => resolve(t, scope) as never),
                scope,
            );
        }
        return scope.has(data) ? scope.get(data) : data.fallback;
    }
    if (Array.isArray(value)) {
        let changed = false;
        const out = value.map((item: unknown) => {
            const next = resolve(item, scope);
            changed ||= next !== item;
            return next;
        });
        return changed ? out : value;
    }
    const proto: unknown = Object.getPrototypeOf(value);
    if (proto !== Object.prototype && proto !== null) {
        return value;
    }
    const fields = value as Record<string, unknown>;
    const theme = fields["theme"];
    const scoped = isTheme(theme);
    const inner = scoped ? new Map([...scope, ...theme[THEME]]) : scope;
    let changed = scoped;
    const out: Record<string, unknown> = {};
    for (const [key, field] of Object.entries(fields)) {
        if (scoped && key === "theme") {
            continue;
        }
        const next = resolve(field, inner);
        changed ||= next !== field;
        out[key] = next;
    }
    return changed ? out : value;
}

/**
 * `value` with every var replaced by its value under the nearest `theme` field above it, or its default, and every
 * `theme` field removed. Unchanged objects are returned as they are.
 */
export function resolveTokens<T>(value: T): T {
    return resolve(value, new Map()) as T;
}
