import { defineWindows, preview } from "plugin:window/ui";
import type { PreviewOptions } from "plugin:window/ui";
import { art } from "plugin:window/theme/industrial";

import { ranks } from "./art.ts";
import clock, { runtime } from "./clock_hud.ts";
import * as handles from "./handles.ts";
import lobby from "./lobby.tsx";
import lookout from "./lookout.tsx";
import search from "./search.tsx";
import shop from "./shop.tsx";
import statusHuds, { bottomCenter, coins, hint, power, rate, topLeft, waveBar, waveCharge } from "./status_hud.tsx";
import storefront from "./storefront.tsx";
import upgrades from "./upgrades.tsx";

/** A preview of the workshop with one card in each stage. */
const upgradesPreview = (options: PreviewOptions) =>
    preview(
        upgrades,
        [
            [handles.upgradeName, ["Drill", "Furnace", "Conveyor"]],
            [handles.upgradeLevel, ["Lv 0/5", "Lv 3/5", "Lv 5/5"]],
            [handles.upgradeCost, ["900", "810", ""]],
            [handles.upgradeProgress, [0, 0.6, 1]],
            [handles.upgradeStage, ["new", "open", "maxed"]],
            [handles.amount, "five"],
            [handles.balance, "2500"],
            [handles.rank, "rank_2"],
            [handles.rankName, "Journeyman"],
            [handles.status, "8 of 15 levels"],
        ],
        options,
    );

/** The storefront browsing Gear, with a card selected and two lines in the cart. */
const storefrontState = (pane: "details" | "cart") =>
    [
        [handles.storeCategory, "gear"],
        [handles.storePane, pane],
        [handles.storeSearch, "ing_"],
        [handles.storeHasSearch, true],
        [handles.storeCartCount, "3"],
        [handles.storeResults, "Gear: 14 items"],
        [handles.storeSortLabel, "Sort: Price"],
        [handles.catalogFocused, pane === "details"],
        [
            handles.cardName,
            [
                "Gilded Ingot",
                "Tempered Iron",
                "Copper Bundle",
                "Tide Trident",
                "Sky Wings",
                "Rocket Crate",
                "Scout Spyglass",
                "Explorer Compass",
                "Chrono Dial",
            ],
        ],
        [handles.cardPrice, ["180", "95", "75", "860", "1150", "155", "165", "205", "260"]],
        [handles.cardIcon, ["gold", "iron", "copper", "trident", "elytra", "rocket", "spyglass", "compass", "clock"]],
        [handles.cardFilled, [true, true, true, true, true, true, true, true, true]],
        [handles.cardFeatured, [false, false, false, true, true]],
        [handles.cardSelected, [false, false, false, true]],
        [handles.catalogThumb, [false, false, true, true, true, true, false, false]],
        [handles.detailIcon, "trident"],
        [handles.detailName, "Tide Trident"],
        [handles.detailTier, "legendary"],
        [handles.detailKind, "Legendary gear"],
        [handles.detailPrice, "860"],
        [handles.detailBlurb, "Forged in the drowned halls. Returns to your hand and calls lightning in a storm."],
        [handles.detailInCart, "1 already in your cart"],
        [handles.quantity, "2"],
        [handles.canLess, true],
        [handles.canMore, true],
        [handles.canBuyNow, true],
        [handles.buyNowLabel, "Buy 1720"],
        [handles.cartName, ["Tide Trident", "Copper Bundle", "Rocket Crate"]],
        [handles.cartQuantity, ["x1", "x1", "x1"]],
        [handles.cartPrice, ["860", "75", "155"]],
        [handles.cartThumb, [true, true, true, true, true]],
        [handles.cartTotal, "1090"],
        [handles.canCheckout, true],
        [handles.storeBalance, "2500"],
        [handles.storeToast, "Added Tide Trident to your cart"],
    ] as const;

export default defineWindows({
    sprites: { searchField: art.inputField, ...ranks },
    windows: [shop, search, upgrades],
    huds: statusHuds,
    fullscreens: [lobby, lookout, storefront],
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
            [handles.budget, 0.42],
            [handles.status, "30 items"],
            [handles.balance, "2500"],
            [handles.canBuy, true],
            [handles.buyLabel, "Buy 210"],
        ]),
        upgradesPreview({ scale: 3 }),
        // The same window with its regions outlined.
        upgradesPreview({ name: "upgrades_regions", outlines: true }),
        preview(
            topLeft,
            [
                [coins, "2500"],
                [rate, "+10"],
                [power, "x1"],
            ],
            { scale: 3 },
        ),
        preview(clock, [[runtime, "4:07"]], { scale: 3 }),
        // The tinted hint in the tint's first palette entry, then in another.
        preview(bottomCenter, [[hint, "Next wave in 4s"]], { scale: 3 }),
        preview(
            bottomCenter,
            [
                [hint, "Wave incoming"],
                [handles.tone, "danger"],
            ],
            { name: "status_bottom_center_danger", scale: 3 },
        ),
        preview(waveBar, [[waveCharge, 0.6]], { scale: 3 }),
        preview(lobby, [
            [handles.foundryName, "Steve's Foundry"],
            [handles.lobbyStatus, "Ready for Emberly"],
            [handles.worldName, ["Ironvale", "Emberly", "Ashmoor"]],
            [handles.ready, true],
        ]),
        preview(storefront, storefrontState("details")),
        preview(storefront, storefrontState("cart"), { name: "storefront_cart" }),
    ],
});
