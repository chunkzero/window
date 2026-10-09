import { Image } from "plugin:window/ui";
import { Prompt } from "plugin:window/theme/industrial";

import { icons } from "./art.ts";
import { back, confirm, query } from "./handles.ts";

/**
 * A static anvil: nothing on it changes after open, so typing never reopens the screen. Vanilla's anvil art is hidden,
 * and without the inventory the shell ends at the hazard bar below the anvil's slots.
 */
export default (
    <Prompt
        name="catalog_search"
        title="Catalog Search"
        label="Name"
        bind={query}
        onConfirm={confirm}
        confirm={<Image art={icons.check} translate={[-1, 0]} />}
        confirmTooltip="Search"
        onBack={back}
        back={<Image art={icons.back} />}
    />
);
