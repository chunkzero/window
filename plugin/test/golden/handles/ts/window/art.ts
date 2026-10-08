import { derive, industrial, shape, texture } from "#plugins/window";

export const coin = shape(
    {
        kind: "button",
        width: 8,
        height: 8,
        radius: 4,
        border_width: 1,
        inset_depth: 1,
        fill: "#ffb20b",
        border_color: "#8d3c03",
        highlight_color: "#ffd731",
        shadow_color: "#c67707",
    },
    { name: "coin" },
);

/** A flat recess strip, drawn as a vent between the container and the inventory. */
export const ventSlot = derive((get) => ({
    ...get(industrial.art.recess),
    border_width: 0,
    inset_depth: 0,
    width: 6,
    height: 2,
}));

/** The anvil's text field, restyled pack-wide through `anvilFieldSprite`. */
export const searchField = derive((get) => ({ ...get(industrial.art.recess), width: 110, height: 16 }));

export const icons = {
    back: texture("window/icons/back.png"),
    clear: texture("window/icons/clear.png"),
    check: texture("window/icons/check.png"),
};
