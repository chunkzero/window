import { item, panel, pattern, repeater, slot, ui } from "#plugins/window";

export default ui({
    name: "list",
    container: "generic_9x3",
    children: [
        panel({ frame: "shell", x: 0, y: 0, width: 176, height: 80 }),
        repeater("rows", {
            frame: "slot_cell",
            pattern: pattern.grid({ x: 0, y: 0, columns: 3, rows: 2, cell_width: 3, cell_height: 1 }),
            children: [
                item("icon", { cell_slot: 1 }),
                item("price", { cell_slot: 2 }),
                slot("label", { x: 2, y: 2, width: 30, color: "#ffffff" }),
            ],
        }),
    ],
});
