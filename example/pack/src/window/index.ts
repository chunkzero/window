import { defineWindows, preview } from "plugin:window/ui";
import { art } from "plugin:window/theme/industrial";

import * as handles from "./handles.ts";
import search from "./search.tsx";
import shop from "./shop.tsx";
import statusHuds, { coins, power, rate } from "./status_hud.tsx";

export default defineWindows({
    sprites: { searchField: art.inputField },
    windows: [shop, search],
    huds: statusHuds,
    // `rpp build` writes these to build/previews/shop.png and build/previews/status_top_left.png.
    previews: [
        preview(shop, [
            [handles.categoryLabel, ["All", "Gear", "Magic"]],
            [handles.sortLabel, ["Top", "Price", "Name"]],
            [handles.products, { selected: 0 }],
            [handles.previousLabel, "Prev"],
            [handles.nextLabel, "Next"],
            [handles.canNext, true],
            [handles.page, "1 of 2"],
            [handles.selectedName, "Amethyst Song"],
            [handles.price, "210"],
            [handles.hasPrice, true],
            [handles.status, "30 items"],
            [handles.balance, "2500"],
            [handles.canBuy, true],
            [handles.buyLabel, "Buy 210"],
        ]),
        preview(
            statusHuds[0],
            [
                [coins, "2500"],
                [rate, "+10"],
                [power, "x1"],
            ],
            { scale: 3 },
        ),
    ],
});
