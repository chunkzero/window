import { defineWindows, preview } from "plugin:window/ui";
import { art } from "plugin:window/theme/industrial";

import * as handles from "./handles.ts";
import search from "./search.tsx";
import shop from "./shop.tsx";
import statusHuds, { coins, hint, power, rate } from "./status_hud.tsx";

export default defineWindows({
    sprites: { searchField: art.inputField },
    windows: [shop, search],
    huds: statusHuds,
    // `rpp build` writes these to build/previews/, one PNG per preview.
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
        // The tinted hint in the tint's first palette entry, then in another.
        preview(statusHuds[5], [[hint, "Next wave in 4s"]], { scale: 3 }),
        preview(
            statusHuds[5],
            [
                [hint, "Wave incoming"],
                [handles.tone, "danger"],
            ],
            { name: "status_bottom_center_danger", scale: 3 },
        ),
    ],
});
