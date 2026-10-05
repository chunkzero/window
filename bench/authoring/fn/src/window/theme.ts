import { presets, theme } from "#plugins/window";

const industrial = presets.industrial();

/** State sprites draw at a fixed size, so each button width gets its own copy of the frame style. */
function sized(frame: keyof typeof industrial.frames, width: number, height: number) {
    return { ...industrial.frames[frame], width, height };
}

export default theme({
    ...industrial,
    sprites: {
        ...industrial.sprites,
        tab: sized("button", 52, 16),
        tab_selected: sized("button_selected", 52, 16),
        action: sized("button", 52, 16),
        action_disabled: sized("button_disabled", 52, 16),
        buy: sized("button_accent", 106, 16),
        buy_disabled: sized("button_disabled", 106, 16),
        clear_search: sized("button", 16, 16),
        clear_search_disabled: sized("button_disabled", 16, 16),
        search_field: sized("recess", 110, 16),
        icon_back: { texture: "window/icons/back.png" },
        icon_clear: { texture: "window/icons/clear.png" },
        icon_check: { texture: "window/icons/check.png" },
        vent_slot: { ...industrial.frames.recess, border_width: 0, inset_depth: 0, width: 6, height: 2 },
        coin: {
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
    },
});
