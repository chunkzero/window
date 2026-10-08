import { shape } from "#plugins/window";

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

const lamp = (fill: `#${string}`) =>
    shape({
        kind: "button",
        width: 4,
        height: 4,
        fill,
        border_color: "#03091f",
        border_width: 1,
        radius: 0,
        inset_depth: 0,
    });

/** The runtime sprites Kotlin selects for sprite handles. */
export default { sprites: { coin, lamp_on: lamp("#8dff5a"), lamp_off: lamp("#074568") } };
