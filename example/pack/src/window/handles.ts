import {
    action,
    collection,
    flag,
    fraction,
    input,
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

// Lobby and lookout, the full-screen windows; `tone` is shared
export const WORLDS = 3;
const worlds = { shape: [WORLDS] } as const;
export const worldName = text("world_name", worlds);
export const pickWorld = action("pick_world", worlds);
export const previousWorld = action("previous_world");
export const nextWorld = action("next_world");
export const foundryName = text("foundry_name");
export const rename = action("rename");
export const lobbyStatus = text("lobby_status");
export const ready = flag("ready");
export const play = action("play");

/** How far the lookout's signal has charged. */
export const signal = fraction("signal");
export const charge = action("charge");

// Storefront, a full-screen window; `tone` is shared
/** The product art a card or the detail pane shows; `none` shows nothing. */
export const STORE_ICONS = [
    "none",
    "diamond",
    "emerald",
    "gold",
    "iron",
    "copper",
    "redstone",
    "lapis",
    "coal",
    "quartz",
    "amethyst",
    "ender_pearl",
    "blaze_rod",
    "ghast_tear",
    "prismarine",
    "heart",
    "nether_star",
    "totem",
    "trident",
    "elytra",
    "rocket",
    "golden_apple",
    "experience",
    "book",
    "spyglass",
    "compass",
    "clock",
    "name_tag",
    "saddle",
    "shell",
    "echo",
] as const;

export const storeCategory = selection("store_category", ["all", "gear", "magic", "deals"]);
export const storePane = selection("store_pane", ["details", "cart"]);
export const storeSearch = text("store_search");
export const storeHasSearch = flag("store_has_search");
export const storeFind = action("store_find");
export const storeClearSearch = action("store_clear_search");
export const storeCartCount = text("store_cart_count");
export const openDetails = action("open_details");
export const openCart = action("open_cart");

export const STORE_CARDS = 9;
const storeCards = { shape: [STORE_CARDS] } as const;
export const cardName = text("card_name", storeCards);
export const cardPrice = text("card_price", storeCards);
export const cardIcon = value("card_icon", STORE_ICONS, storeCards);
export const cardFilled = flag("card_filled", storeCards);
export const cardFeatured = flag("card_featured", storeCards);
export const cardSelected = flag("card_selected", storeCards);
export const pickCard = action("pick_card", storeCards);
export const storeResults = text("store_results");
export const storeSortLabel = text("store_sort_label");
export const storeSort = action("store_sort");
/** Which pane the scroll wheel moves. */
export const catalogFocused = flag("catalog_focused");

/** Segments of a scrollbar track; each is lit while its stretch of the list is in view. */
export const CATALOG_TRACK = 8;
export const catalogThumb = flag("catalog_thumb", { shape: [CATALOG_TRACK] });
export const catalogUp = action("catalog_up");
export const catalogDown = action("catalog_down");

export const detailIcon = value("detail_icon", STORE_ICONS);
export const detailName = text("detail_name");
export const detailTier = tint("detail_tier", {
    common: "#ffffff",
    rare: "#55ffff",
    epic: "#ff77ff",
    legendary: "#ffb20b",
});
export const detailKind = text("detail_kind");
export const detailPrice = text("detail_price");
export const detailBlurb = text("detail_blurb");
export const detailInCart = text("detail_in_cart");
export const quantity = text("quantity");
export const canLess = flag("can_less");
export const canMore = flag("can_more");
export const less = action("less");
export const more = action("more");
export const addToCart = action("add_to_cart");
export const buyNow = action("buy_now");
export const canBuyNow = flag("can_buy_now");
export const buyNowLabel = text("buy_now_label");

export const CART_ROWS = 5;
const cartRows = { shape: [CART_ROWS] } as const;
export const cartName = text("cart_name", cartRows);
export const cartQuantity = text("cart_quantity", cartRows);
export const cartPrice = text("cart_price", cartRows);
export const removeLine = action("remove_line", cartRows);
export const cartThumb = flag("cart_thumb", { shape: [CART_ROWS] });
export const cartUp = action("cart_up");
export const cartDown = action("cart_down");
export const cartEmpty = flag("cart_empty");
export const cartTotal = text("cart_total");
export const checkout = action("checkout");
export const canCheckout = flag("can_checkout");
export const clearCart = action("clear_cart");

export const storeBalance = text("store_balance");
export const storeToast = text("store_toast");
