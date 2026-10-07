/** What a handle declares. See docs/AUTHORING.md. */
export type HandleKind =
    | "flag"
    | "toggle"
    | "value"
    | "selection"
    | "text"
    | "sprite"
    | "items"
    | "collection"
    | "action"
    | "input"
    | "builtin";

/** The extent of an indexed handle: `[n]` or `[rows, columns]`. */
export type Shape = readonly [number] | readonly [number, number];

/** The index arguments `.at()` takes for `S`. */
export type IndexOf<S extends Shape> = S extends readonly [number] ? [i: number] : [row: number, column: number];

/** A handle, or one entry of an indexed handle, as a prop reads it. */
export interface Ref<K extends HandleKind = HandleKind, V extends string = string> {
    readonly kind: K;
    readonly id: string;
    readonly values?: readonly V[];
    readonly shape?: Shape;
    readonly initial?: string | boolean;
    readonly selectable?: boolean;
    /** The entry an indexed handle's `.at()` selected. */
    readonly at?: readonly number[];
    /** Type-only: an indexed handle is read through `.at()`. */
    readonly indexed?: false;
}

/** An indexed handle; select an entry with `.at()`. */
export type Indexed<R extends Ref, S extends Shape> = Omit<R, "indexed" | "is" | "set"> & {
    readonly shape: S;
    readonly indexed?: true;
    at(...index: IndexOf<S>): R;
};

/** A value or selection compared with one of its values: `mode.is("buy")`. */
export type Is = Ref<"value" | "selection"> & { readonly is: string };
/** A selection click that assigns one of its values: `category.set("gear")`. */
export type Set = Ref<"selection"> & { readonly set: string };

/** A Boolean a `<Show when>` or `<Button enabled>` reads. */
export type Condition = Ref<"flag" | "toggle"> | Is;
/** What a click does: call an action, change UI-owned state, or run a runtime action. */
export type ClickAction = Ref<"action" | "builtin" | "toggle"> | Set;

export type Flag = Ref<"flag">;
export type Toggle = Ref<"toggle">;
export type Text = Ref<"text">;
export type Sprite = Ref<"sprite">;
export type Items = Ref<"items">;
export type Collection = Ref<"collection">;
export type Action = Ref<"action">;
export type Input = Ref<"input">;
export type Builtin = Ref<"builtin">;

export interface Value<V extends string = string> extends Ref<"value", V> {
    readonly values: readonly V[];
    /** A condition that holds while the value is `value`. */
    is(value: V): Is;
}

export interface Selection<V extends string = string> extends Ref<"selection", V> {
    readonly values: readonly V[];
    /** A condition that holds while the selection is `value`. */
    is(value: V): Is;
    /** A click that selects `value`. */
    set(value: V): Set;
}

/** Runtime actions the runtime resolves without a Kotlin member. */
export type BuiltinId = "window:close";

type Fields = Record<string, unknown>;

/** The handle fields of an element's JSON: declaration, entry, and comparison or assigned value. */
export function refJson(ref: { readonly kind: HandleKind; readonly id: string }): Fields {
    const out: Fields = { kind: ref.kind, id: ref.id };
    const fields = ref as unknown as Fields;
    for (const key of ["values", "shape", "initial", "selectable", "at", "is", "set"]) {
        // An indexed handle's `at` is its entry selector, not an entry.
        if (fields[key] !== undefined && typeof fields[key] !== "function") {
            out[key] = fields[key];
        }
    }
    return out;
}

/** Whether `value` is a handle reference rather than a string binding name. */
export function isRef(value: unknown): value is Ref {
    return typeof value === "object" && value !== null && "kind" in value && "id" in value;
}

function requireId(id: unknown, kind: string): asserts id is string {
    if (typeof id !== "string" || !/^[a-z][a-z0-9_]*$/.test(id)) {
        throw new Error(`${kind} id ${JSON.stringify(id)} must match ^[a-z][a-z0-9_]*$`);
    }
}

function requireShape(shape: unknown, id: string): void {
    const valid =
        Array.isArray(shape) &&
        (shape.length === 1 || shape.length === 2) &&
        shape.every((n) => Number.isInteger(n) && (n as number) > 0);
    if (!valid) {
        throw new Error(`handle \`${id}\` shape must be [n] or [rows, columns] of positive integers`);
    }
}

function make<R extends Ref>(base: R, extend: (ref: R) => object): R {
    return Object.freeze({ ...base, ...extend(base) }) as R;
}

/** Adds `.at()` to a shaped handle; each entry keeps the handle's other helpers. */
function shaped<R extends Ref>(base: R, shape: Shape | undefined, extend: (ref: R) => object): unknown {
    if (shape === undefined) {
        return make(base, extend);
    }
    requireShape(shape, base.id);
    const ref = { ...base, shape } as R;
    return Object.freeze({
        ...ref,
        at(...index: number[]): R {
            if (
                index.length !== shape.length ||
                index.some((n, d) => !Number.isInteger(n) || n < 0 || n >= shape[d]!)
            ) {
                throw new Error(`\`${base.id}.at(${index.join(", ")})\` is outside its shape [${shape.join(", ")}]`);
            }
            return make({ ...ref, at: index }, extend);
        },
    });
}

const none = (): object => ({});

function enumerated<V extends string>(kind: string, id: string, values: readonly V[]): readonly V[] {
    requireId(id, kind);
    if (!Array.isArray(values) || values.length === 0 || new globalThis.Set(values).size !== values.length) {
        throw new Error(`${kind} \`${id}\` requires distinct values`);
    }
    return Object.freeze([...values]);
}

function comparable<V extends string>(ref: Ref<"value" | "selection", V>): { is(value: V): Is } {
    return {
        is(value: V): Is {
            if (!ref.values?.includes(value)) {
                throw new Error(`\`${ref.id}.is(${JSON.stringify(value)})\` is not one of its values`);
            }
            return Object.freeze({ ...refJson(ref), is: value }) as unknown as Is;
        },
    };
}

/** A Boolean Kotlin computes: `protected abstract fun canBuy(): Boolean`. */
export function flag(id: string): Flag;
export function flag<const S extends Shape>(id: string, options: { shape: S }): Indexed<Flag, S>;
export function flag(id: string, options?: { shape?: Shape }): unknown {
    requireId(id, "flag");
    return shaped({ kind: "flag", id } as Flag, options?.shape, none);
}

/** A Boolean the UI owns: `protected var favorites: Boolean` plus `onFavoritesChanged(value)`. */
export function toggleHandle(id: string, options: { initial?: boolean } = {}): Toggle {
    requireId(id, "toggle");
    return make({ kind: "toggle", id, initial: options.initial ?? false } as Toggle, none);
}

/** One of `values`, computed by Kotlin as a generated enum. */
export function value<const V extends string>(id: string, values: readonly [V, ...V[]]): Value<V>;
export function value<const V extends string, const S extends Shape>(
    id: string,
    values: readonly [V, ...V[]],
    options: { shape: S },
): Indexed<Value<V>, S>;
export function value(id: string, values: readonly string[], options?: { shape?: Shape }): unknown {
    const base = { kind: "value", id, values: enumerated("value", id, values) } as Value;
    return shaped(base, options?.shape, comparable);
}

/** One of `values`, owned by the UI: `protected var category: Category` plus `onCategoryChanged(value)`. */
export function selection<const V extends string>(
    id: string,
    values: readonly [V, ...V[]],
    options: { initial?: NoInfer<V> } = {},
): Selection<V> {
    const all = enumerated("selection", id, values);
    const initial = options.initial ?? all[0]!;
    if (!all.includes(initial)) {
        throw new Error(`selection \`${id}\` initial ${JSON.stringify(initial)} is not one of its values`);
    }
    const base = { kind: "selection", id, values: all, initial } as unknown as Selection<V>;
    return make(base, (ref) => ({
        ...comparable(ref),
        set(value: V): Set {
            if (!all.includes(value)) {
                throw new Error(`\`${id}.set(${JSON.stringify(value)})\` is not one of its values`);
            }
            return Object.freeze({ ...refJson(ref), set: value }) as unknown as Set;
        },
    }));
}

/** Dynamic text: `protected abstract fun title(): Component`. */
export function textHandle(id: string): Text;
export function textHandle<const S extends Shape>(id: string, options: { shape: S }): Indexed<Text, S>;
export function textHandle(id: string, options?: { shape?: Shape }): unknown {
    requireId(id, "text");
    return shaped({ kind: "text", id } as Text, options?.shape, none);
}

/** A runtime sprite: `protected abstract fun iconSprite(): WindowSprite?`. */
export function spriteHandle(id: string): Sprite;
export function spriteHandle<const S extends Shape>(id: string, options: { shape: S }): Indexed<Sprite, S>;
export function spriteHandle(id: string, options?: { shape?: Shape }): unknown {
    requireId(id, "sprite");
    return shaped({ kind: "sprite", id } as Sprite, options?.shape, none);
}

/** An inventory item: `protected abstract fun stack(): ItemStack?`. */
export function items(id: string): Items;
export function items<const S extends Shape>(id: string, options: { shape: S }): Indexed<Items, S>;
export function items(id: string, options?: { shape?: Shape }): unknown {
    requireId(id, "items");
    return shaped({ kind: "items", id } as Items, options?.shape, none);
}

/**
 * A collection's cells: `protected abstract val products: WindowCollection<ItemStack>`. `selectable` lets its
 * `<Collection selected>` sprite mark the selected cell.
 */
export function collectionHandle(id: string, options: { selectable?: boolean } = {}): Collection {
    requireId(id, "collection");
    const base = { kind: "collection", id, ...(options.selectable === true ? { selectable: true } : {}) };
    return make(base as Collection, none);
}

/** A click Kotlin handles: `protected abstract fun onBuy(click: Click)`, or `IndexedClick` when indexed. */
export function action(id: string): Action;
export function action<const S extends Shape>(id: string, options: { shape: S }): Indexed<Action, S>;
export function action(id: string, options?: { shape?: Shape }): unknown {
    requireId(id, "action");
    return shaped({ kind: "action", id } as Action, options?.shape, none);
}

/** An anvil text input: `protected abstract fun onQueryChanged(value: String)`. */
export function input(id: string): Input {
    requireId(id, "input");
    return make({ kind: "input", id } as Input, none);
}

/** A runtime action with no Kotlin member, such as `window:close`. */
export function builtin(id: BuiltinId): Builtin {
    if (id !== "window:close") {
        throw new Error(`unknown runtime action ${JSON.stringify(id)}`);
    }
    return make({ kind: "builtin", id } as Builtin, none);
}
