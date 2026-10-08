import { action, industrial, items, raw, text } from "#plugins/window";

const rows = action("rows", { shape: [6] });
const icon = items("icon", { shape: [6] });
const price = items("price", { shape: [6] });
const label = text("label", { shape: [6] });

/** Six recessed cells of three slots under a label, showing an icon and a price stack; any click on a cell runs `rows[i]`. */
const cells = Array.from({ length: 6 }, (_, i) => {
    const [column, row] = [(i % 3) * 3 + 1, Math.floor(i / 3) + 1];
    return [
        raw.box({
            layout: { column: { start: column, span: 3 }, row },
            style: { padding: 1 },
            children: [
                raw.box({
                    frame: industrial.art.recess,
                    style: { grow: 1 },
                    children: [raw.text(label.at(i), { x: 2, y: 2, width: 30, color: "#ffffff" })],
                }),
            ],
        }),
        raw.items(icon.at(i), { layout: { column, row } }),
        raw.items(price.at(i), { layout: { column: column + 1, row } }),
        raw.region({ on_click: rows.at(i), layout: { column: { start: column, span: 3 }, row } }),
    ];
}).flat();

export default raw.ui({
    name: "list",
    container: "generic_9x3",
    children: [
        raw.box({ frame: industrial.art.shell, x: 0, y: 0, style: { width: 176, height: 80 } }),
        raw.section("container", { claim: "none", children: cells }),
    ],
});
