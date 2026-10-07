import { defineWindows } from "#plugins/window";

import search from "./search.tsx";
import shop from "./shop.tsx";
import statusHuds from "./status_hud.tsx";
import theme from "./theme.ts";

export default defineWindows({ themes: [theme], windows: [shop, search], huds: statusHuds });
