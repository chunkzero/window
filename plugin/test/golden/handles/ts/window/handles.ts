import { action, collection, flag, input, selection, text, toggle } from "plugin:window/bind";

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

export const search = action("search");
export const clearSearch = action("clear_search");
export const hasQuery = flag("has_query");
export const status = text("status");
export const balance = text("balance");

export const buy = action("buy");
export const canBuy = flag("can_buy");
export const buyLabel = text("buy_label");

// Catalog search
export const query = input("query");
export const back = action("back");
export const confirm = action("confirm");
