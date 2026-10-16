import { derive, shape, texture } from "plugin:window/ui";
import { art } from "plugin:window/theme/industrial";

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
    ...get(art.recess),
    border_width: 0,
    inset_depth: 0,
    width: 6,
    height: 2,
}));

/** Grey art for a tint to color: the dark fill takes a dim shade of the tint, the lighter rim a brighter one. */
export const tonePill = shape({
    kind: "panel",
    fill: "#404040",
    border_color: "#b0b0b0",
    border_width: 1,
    radius: 4,
    inset_depth: 0,
});

/** A white bar, which a tint colors exactly. */
export const toneLamp = shape({
    kind: "button",
    width: 2,
    height: 10,
    fill: "#ffffff",
    border_width: 0,
    radius: 0,
    inset_depth: 0,
});

/** The empty part of a bar. */
export const barTrack = shape({ kind: "panel", fill: "#0a1433", border_width: 0, radius: 0, inset_depth: 0 });

/** A white bar fill, which a tint colors exactly. */
export const toneFill = shape({ kind: "panel", fill: "#ffffff", border_width: 0, radius: 0, inset_depth: 0 });

/** A bar fill with a darker rim, so its right edge shows only once the bar is full. */
export const budgetFill = shape({
    kind: "panel",
    fill: "#ffb20b",
    border_color: "#8d3c03",
    border_width: 1,
    radius: 0,
    inset_depth: 0,
});

/** The plate a maxed upgrade shows in place of its level bar. */
export const maxedPlate = shape({
    kind: "panel",
    fill: "#ffb20b",
    border_color: "#8d3c03",
    border_width: 1,
    radius: 2,
    inset_depth: 0,
});

/** White art for the `rank` sprite slot, which a tint colors. */
export const ranks = {
    rank_1: texture("window/icons/rank_1.png"),
    rank_2: texture("window/icons/rank_2.png"),
    rank_3: texture("window/icons/rank_3.png"),
};

export const icons = {
    back: texture("window/icons/back.png"),
    clear: texture("window/icons/clear.png"),
    check: texture("window/icons/check.png"),
};

/** White translucent glass, nine-sliced over its box; a `tint` colors it and the world shows through. */
export const glass = texture("window/icons/store/glass.png", { insets: 2 });

/** A storefront product's 16x16 art by `STORE_ICONS` id; as a frame, it stretches over its box. */
export const productIcon = (id: string) => texture(`window/icons/store/products/${id}.png`, { insets: 0 });

export const storeIcons = {
    arrowUp: texture("window/icons/store/arrow_up.png"),
    arrowDown: texture("window/icons/store/arrow_down.png"),
    magnifier: texture("window/icons/store/magnifier.png"),
    cart: texture("window/icons/store/cart.png"),
};
