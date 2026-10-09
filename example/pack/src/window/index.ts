import { defineWindows } from "plugin:window/ui";
import { art } from "plugin:window/theme/industrial";

import search from "./search.tsx";
import shop from "./shop.tsx";
import statusHuds from "./status_hud.tsx";

export default defineWindows({ sprites: { searchField: art.inputField }, windows: [shop, search], huds: statusHuds });
