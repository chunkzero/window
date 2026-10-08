import { pattern, raw } from "#plugins/window";

export default raw.ui({
    name: "list",
    container: "generic_9x3",
    children: [
        raw.panel({ frame: "shell", x: 0, y: 0, width: 176, height: 80 }),
        raw.repeater("rows", {
            frame: "recess",
            pattern: pattern.grid({ x: 0, y: 0, columns: 3, rows: 2, cell_width: 3, cell_height: 1 }),
            children: [
                raw.item("icon", { cell_slot: 1 }),
                raw.item("price", { cell_slot: 2 }),
                raw.slot("label", { x: 2, y: 2, width: 30, color: "#ffffff" }),
            ],
        }),
    ],
});
