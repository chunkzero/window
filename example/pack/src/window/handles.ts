import {
    action,
    collection,
    flag,
    fraction,
    input,
    items,
    selection,
    sprite,
    text,
    tint,
    toggle,
    value,
} from "plugin:window/bind";

// Shared by the shop, the upgrades window, and the status HUD
export const tone = tint("tone", { ok: "#3ddc84", warn: "#ffb020", danger: "#ff4d4d" });

// Shop
export const category = selection("category", ["all", "gear", "magic"]);
export const sort = selection("sort", ["featured", "price", "name"]);
export const categoryLabel = text("category_label", { shape: [3] });
export const sortLabel = text("sort_label", { shape: [3] });

export const favorites = toggle("favorites");
export const affordable = toggle("affordable");

export const products = collection("products", { selectable: true });
export const previous = action("previous");
export const next = action("next");
export const canPrevious = flag("can_previous");
export const canNext = flag("can_next");
export const previousLabel = text("previous_label");
export const nextLabel = text("next_label");
export const page = text("page");

export const selectedName = text("selected_name");
export const price = text("price");
export const hasPrice = flag("has_price");
/** How much of the selected product's price the balance covers. */
export const budget = fraction("budget");

export const search = action("search");
export const clearSearch = action("clear_search");
export const hasQuery = flag("has_query");
export const status = text("status");
export const balance = text("balance");

export const buy = action("buy");
export const canBuy = flag("can_buy");
export const buyLabel = text("buy_label");
export const openUpgrades = action("open_upgrades");

// Catalog search
export const query = input("query");
export const back = action("back");
export const confirm = action("confirm");

// Upgrades; `status` and `balance` are the shop's handles
export const UPGRADES = 3;
const cards = { shape: [UPGRADES] } as const;
export const upgrade = action("upgrade", cards);
export const upgradeIcon = items("upgrade_icon", cards);
export const upgradeName = text("upgrade_name", cards);
export const upgradeLevel = text("upgrade_level", cards);
export const upgradeCost = text("upgrade_cost", cards);
/** How far each upgrade is toward its last level. */
export const upgradeProgress = fraction("upgrade_progress", cards);
export const upgradeStage = value("upgrade_stage", ["new", "open", "maxed"], cards);

export const amount = selection("amount", ["one", "five", "max"]);
/** The workshop's rank badge, one of the catalog sprites `index.ts` registers. */
export const rank = sprite("rank", { only: ["rank_1", "rank_2", "rank_3"] });
export const rankName = text("rank_name");
export const openShop = action("open_shop");
