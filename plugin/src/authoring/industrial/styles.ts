/** Industrial's default component styles. */
import { variants } from "../styles.ts";
import type { ContainerStyle, Variants } from "../styles.ts";
import type { ArtRef } from "../types.ts";
import { art } from "./art.ts";

export interface IndustrialStyles {
    button: Variants<"base" | "disabled", ContainerStyle>;
    toggle: Variants<"base", ContainerStyle>;
    /** Choices and tabs. */
    choice: Variants<"base" | "selected", ContainerStyle>;
    repeater: Variants<"cell", ContainerStyle>;
    collection: { frame: ArtRef; selected: ArtRef };
    slots: { frame: ArtRef };
}

export const styles: IndustrialStyles = {
    button: variants({ base: { frame: art.button }, disabled: { frame: art.buttonDisabled } }),
    toggle: variants({ base: { frame: art.button } }),
    choice: variants({ base: { frame: art.button }, selected: { frame: art.buttonSelected } }),
    repeater: variants({ cell: { frame: art.button } }),
    collection: { frame: art.slot, selected: art.slotSelected },
    slots: { frame: art.slot },
};
