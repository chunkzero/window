import { defineWindows } from "#plugins/window";

import { searchField } from "./art.ts";
import search from "./search.tsx";
import shop from "./shop.tsx";
import statusHuds from "./status_hud.tsx";

export default defineWindows({ sprites: { searchField }, windows: [shop, search], huds: statusHuds });
