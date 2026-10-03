import type { GeneratedFrame, GeneratedSprite, IndustrialPresetOptions } from "./types.ts";

function mix(hex: string, target: string, amount: number): string {
    const channel = (value: string, at: number) => parseInt(value.slice(at, at + 2), 16);
    let out = "#";
    for (const at of [1, 3, 5]) {
        const value = Math.round(channel(hex, at) + (channel(target, at) - channel(hex, at)) * amount);
        out += value.toString(16).padStart(2, "0");
    }
    return out;
}

/** Bevel tones for a fill: light shifts toward `light`, dark toward `dark`, keeping the surface saturated. */
interface Tones {
    light: string;
    dark: string;
}

const cool: Tones = { light: "#5ff0ff", dark: "#020a30" };
const warm: Tones = { light: "#fff04a", dark: "#5c0a00" };
const leaf: Tones = { light: "#d8ff60", dark: "#032a00" };

/** A 1px-bordered surface lit from the top-left. */
function raised(kind: "panel" | "button", fill: string, border: string, tones: Tones): GeneratedFrame {
    return {
        kind,
        fill,
        border_color: border,
        border_width: 1,
        radius: 0,
        inset_depth: 1,
        highlight_color: mix(fill, tones.light, 0.45),
        shadow_color: mix(fill, tones.dark, 0.4),
    };
}

/** A surface pressed into its parent: dark top-left edge, light bottom-right edge. */
function sunken(kind: "panel" | "slot" | "button", fill: string, border: string | null, tones: Tones): GeneratedFrame {
    return {
        kind,
        fill,
        border_color: border ?? fill,
        border_width: border ? 1 : 0,
        radius: 0,
        inset_depth: 1,
        highlight_color: mix(fill, tones.dark, 0.5),
        shadow_color: mix(fill, tones.light, 0.3),
    };
}

type IndustrialFrame =
    | "shell"
    | "panel"
    | "recess"
    | "slot"
    | "button"
    | "button_selected"
    | "button_disabled"
    | "button_accent"
    | "button_danger"
    | "button_confirm"
    | "hud"
    | "hazard_bar";
type IndustrialSprite = "rivet" | "lamp_on" | "lamp_off" | "slot_selected";

/** Assignable to `Theme`, with frame and sprite names typed so callers can reference them by key. */
function industrial(opts: IndustrialPresetOptions = {}): {
    frames: Record<IndustrialFrame, GeneratedFrame>;
    sprites: Record<IndustrialSprite, GeneratedSprite>;
} {
    const border = opts.border_color ?? "#03091f";
    const accent = opts.accent_color ?? "#ff8300";
    const danger = opts.danger_color ?? "#ed171b";
    const confirm = opts.confirm_color ?? "#20c900";
    const highlight = opts.highlight_color ?? "#25c9ec";
    const stripeFill = opts.stripe_fill ?? "#ffb20b";
    const disabled = opts.disabled_fill ?? "#0a6d99";
    const selectedFill = opts.selected_fill ?? "#05406b";
    const rivet: GeneratedSprite = {
        kind: "button",
        width: 5,
        height: 5,
        fill: "#6f88ba",
        border_color: "#6f88ba",
        border_width: 0,
        radius: 3,
        inset_depth: 1,
        highlight_color: "#eef5ff",
        shadow_color: "#0a1433",
    };
    const lamp = (fill: string): GeneratedSprite => ({
        kind: "button",
        width: 4,
        height: 4,
        fill,
        border_color: border,
        border_width: 1,
        radius: 0,
        inset_depth: 0,
    });

    return {
        frames: {
            shell: raised("panel", opts.shell_fill ?? "#172f69", border, cool),
            panel: { ...raised("panel", opts.panel_fill ?? "#087cac", border, cool), highlight_color: highlight },
            recess: sunken("panel", opts.recess_fill ?? "#071a43", border, cool),
            slot: sunken("slot", opts.slot_fill ?? "#07577d", null, cool),
            button: { ...raised("button", opts.button_fill ?? "#0994c6", border, cool), highlight_color: highlight },
            button_selected: {
                ...sunken("button", selectedFill, border, cool),
                indicator_color: accent,
            },
            button_disabled: {
                kind: "button",
                fill: disabled,
                border_color: mix(disabled, border, 0.5),
                border_width: 1,
                radius: 0,
                inset_depth: 0,
            },
            button_accent: raised("button", accent, mix(accent, warm.dark, 0.6), warm),
            button_danger: raised("button", danger, mix(danger, warm.dark, 0.7), warm),
            button_confirm: raised("button", confirm, mix(confirm, leaf.dark, 0.6), leaf),
            hud: {
                kind: "panel",
                fill: opts.hud_fill ?? "#0a1d4a",
                border_color: mix(highlight, border, 0.6),
                border_width: 1,
                radius: 0,
                inset_depth: 0,
            },
            hazard_bar: {
                kind: "hazard_bar",
                fill: stripeFill,
                border_color: border,
                border_width: 1,
                radius: 0,
                inset_depth: 0,
                highlight_color: mix(stripeFill, warm.light, 0.5),
                shadow_color: mix(stripeFill, warm.dark, 0.5),
                stripe_color: opts.stripe_color ?? "#141a2e",
                stripe_width: 5,
            },
        },
        sprites: {
            rivet,
            lamp_on: lamp(opts.lamp_color ?? "#8dff5a"),
            lamp_off: lamp(mix(disabled, border, 0.4)),
            slot_selected: { ...sunken("slot", selectedFill, accent, cool), width: 18, height: 18 },
        },
    };
}

export const presets: { industrial: typeof industrial } = { industrial };
