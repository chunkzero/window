import { action, raw, text } from "#plugins/window";

export default raw.ui({
    name: "wallet",
    container: "generic_9x3",
    children: [
        raw.text("YOUR WALLET", { x: 8, y: 6, width: 160, align: "center", bold: true }),
        raw.text(text("balance"), { x: 8, y: 20, width: 160, align: "center" }),
        raw.section("container", {
            claim: "none",
            children: [
                raw.box({
                    layout: { column: 5, row: 2 },
                    style: { justify: "center", align: "center" },
                    children: [
                        raw.text("+1", { bold: true }),
                        raw.region({ on_click: action("earn"), tooltip: "Earn one persistent coin" }),
                    ],
                }),
            ],
        }),
    ],
});
