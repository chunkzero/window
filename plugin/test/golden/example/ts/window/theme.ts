import { presets, theme } from "#plugins/window";
import type { GeneratedFrame } from "#plugins/window";

const industrial = presets.industrial();
const frames = industrial.frames as Record<string, GeneratedFrame>;

/** State sprites draw at a fixed size, so each button width gets its own copy of the frame style. */
function sized(frame: string, width: number, height: number) {
    return { ...frames[frame], width, height };
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
        vent_slot: { ...frames.recess, border_width: 0, inset_depth: 0, width: 6, height: 2 },
    },
});
