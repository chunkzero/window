/** Industrial's color tokens, bevel recipes, and named art. */
import { shape } from "../../ui/art.ts";
import { defineVars, derive, mix } from "../../ui/tokens.ts";
import type { Color, Get, Token, Var, Vars } from "../../ui/tokens.ts";
import type { ShapeArt, ShapeStyle } from "../../ui/types.ts";

const defaults = {
    border: "#03091f",
    highlight: "#25c9ec",
    shell: "#172f69",
    panel: "#087cac",
    recess: "#071a43",
    slot: "#07577d",
    button: "#0994c6",
    selected: "#05406b",
    disabled: "#0a6d99",
    accent: "#ff8300",
    danger: "#ed171b",
    confirm: "#20c900",
    hud: "#0a1d4a",
    hazard: "#ffb20b",
    hazardStripe: "#141a2e",
    lamp: "#8dff5a",
    rivet: "#6f88ba",
    rivetHighlight: "#eef5ff",
    rivetShadow: "#0a1433",
} as const;

/** Industrial's colors; `createTheme(industrial.colors, { … })` recolors every industrial art value under it. */
export const colors: Vars<typeof defaults> = defineVars(defaults);

/** Bevel tones: a raised or sunken edge shifts its fill toward `light` or `dark`, keeping the surface saturated. */
export interface Tones {
    light: Color;
    dark: Color;
}

export const tones: { readonly cool: Tones; readonly warm: Tones; readonly leaf: Tones } = {
    cool: { light: "#5ff0ff", dark: "#020a30" },
    warm: { light: "#fff04a", dark: "#5c0a00" },
    leaf: { light: "#d8ff60", dark: "#032a00" },
};

/** Options of `raised` and `sunken`; other shape fields, such as `width` or `indicator_color`, override the recipe's. */
export interface BevelOptions extends Omit<ShapeStyle, "fill"> {
    /** Generated kind; defaults to `"button"`. */
    kind?: "panel" | "button" | "slot";
    /** Border color; `null` draws no border. Defaults to `colors.border`. */
    border?: Token<Color> | null;
    /** Defaults to `tones.cool`. */
    tones?: Tones;
    /** Prefixes the art's content-hashed name. */
    name?: string;
}

function bevel(fill: Token<Color>, options: BevelOptions, sunken: boolean): Var<ShapeArt> {
    const { kind = "button", border = colors.border, tones: tone = tones.cool, name, ...rest } = options;
    return derive((get) => {
        const base = get(fill);
        const edge = border === null ? base : get(border);
        const style: ShapeStyle = {
            kind,
            fill: base,
            border_color: edge,
            border_width: border === null ? 0 : 1,
            radius: 0,
            inset_depth: 1,
            highlight_color: sunken ? mix(base, tone.dark, 0.5) : mix(base, tone.light, 0.45),
            shadow_color: sunken ? mix(base, tone.light, 0.3) : mix(base, tone.dark, 0.4),
        };
        const overrides = Object.entries(rest).filter(([, value]) => value !== undefined);
        return shape({ ...style, ...Object.fromEntries(overrides) }, name === undefined ? {} : { name });
    });
}

/** A 1px-bordered surface lit from the top-left. */
export function raised(fill: Token<Color>, options: BevelOptions = {}): Var<ShapeArt> {
    return bevel(fill, options, false);
}

/** A surface pressed into its parent: dark top-left edge, light bottom-right edge. */
export function sunken(fill: Token<Color>, options: BevelOptions = {}): Var<ShapeArt> {
    return bevel(fill, options, true);
}

/** A flat 1px-bordered surface. */
function flat(kind: "panel" | "button", fill: Token<Color>, border: (get: Get) => Color, name: string): Var<ShapeArt> {
    return derive((get) =>
        shape(
            { kind, fill: get(fill), border_color: border(get), border_width: 1, radius: 0, inset_depth: 0 },
            { name },
        ),
    );
}

const lamp = (fill: (get: Get) => Color, name: string): Var<ShapeArt> =>
    derive((get) =>
        shape(
            {
                kind: "button",
                width: 4,
                height: 4,
                fill: fill(get),
                border_color: get(colors.border),
                border_width: 1,
                radius: 0,
                inset_depth: 0,
            },
            { name },
        ),
    );

/** Industrial's frames, drawn at their box's size, and images, drawn at their own size. */
export interface IndustrialArt {
    shell: Var<ShapeArt>;
    panel: Var<ShapeArt>;
    recess: Var<ShapeArt>;
    slot: Var<ShapeArt>;
    button: Var<ShapeArt>;
    buttonSelected: Var<ShapeArt>;
    buttonDisabled: Var<ShapeArt>;
    buttonAccent: Var<ShapeArt>;
    buttonDanger: Var<ShapeArt>;
    buttonConfirm: Var<ShapeArt>;
    hud: Var<ShapeArt>;
    hazardBar: Var<ShapeArt>;
    /** 5x5. */
    rivet: Var<ShapeArt>;
    /** 4x4. */
    lampOn: Var<ShapeArt>;
    /** 4x4. */
    lampOff: Var<ShapeArt>;
    /** 18x18, over a collection's selected cell. */
    slotSelected: Var<ShapeArt>;
}

export const art: IndustrialArt = {
    shell: raised(colors.shell, { kind: "panel", name: "industrial/shell" }),
    panel: raised(colors.panel, { kind: "panel", highlight_color: colors.highlight, name: "industrial/panel" }),
    recess: sunken(colors.recess, { kind: "panel", name: "industrial/recess" }),
    slot: sunken(colors.slot, { kind: "slot", border: null, name: "industrial/slot" }),
    button: raised(colors.button, { highlight_color: colors.highlight, name: "industrial/button" }),
    buttonSelected: sunken(colors.selected, { indicator_color: colors.accent, name: "industrial/button-selected" }),
    buttonDisabled: flat(
        "button",
        colors.disabled,
        (get) => mix(get(colors.disabled), get(colors.border), 0.5),
        "industrial/button-disabled",
    ),
    buttonAccent: derive((get) =>
        get(
            raised(colors.accent, {
                border: mix(get(colors.accent), tones.warm.dark, 0.6),
                tones: tones.warm,
                name: "industrial/button-accent",
            }),
        ),
    ),
    buttonDanger: derive((get) =>
        get(
            raised(colors.danger, {
                border: mix(get(colors.danger), tones.warm.dark, 0.7),
                tones: tones.warm,
                name: "industrial/button-danger",
            }),
        ),
    ),
    buttonConfirm: derive((get) =>
        get(
            raised(colors.confirm, {
                border: mix(get(colors.confirm), tones.leaf.dark, 0.6),
                tones: tones.leaf,
                name: "industrial/button-confirm",
            }),
        ),
    ),
    hud: flat("panel", colors.hud, (get) => mix(get(colors.highlight), get(colors.border), 0.6), "industrial/hud"),
    hazardBar: derive((get) => {
        const fill = get(colors.hazard);
        return shape(
            {
                kind: "hazard_bar",
                fill,
                border_color: get(colors.border),
                border_width: 1,
                radius: 0,
                inset_depth: 0,
                highlight_color: mix(fill, tones.warm.light, 0.5),
                shadow_color: mix(fill, tones.warm.dark, 0.5),
                stripe_color: get(colors.hazardStripe),
                stripe_width: 5,
            },
            { name: "industrial/hazard-bar" },
        );
    }),
    rivet: derive((get) => {
        const fill = get(colors.rivet);
        return shape(
            {
                kind: "button",
                width: 5,
                height: 5,
                fill,
                border_color: fill,
                border_width: 0,
                radius: 3,
                inset_depth: 1,
                highlight_color: get(colors.rivetHighlight),
                shadow_color: get(colors.rivetShadow),
            },
            { name: "industrial/rivet" },
        );
    }),
    lampOn: lamp((get) => get(colors.lamp), "industrial/lamp-on"),
    lampOff: lamp((get) => mix(get(colors.disabled), get(colors.border), 0.4), "industrial/lamp-off"),
    slotSelected: sunken(colors.selected, {
        kind: "slot",
        border: colors.accent,
        width: 18,
        height: 18,
        name: "industrial/slot-selected",
    }),
};
