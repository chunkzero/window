import { raw } from "#plugins/window";

export default raw.ui({
    name: "wallet",
    container: "generic_9x3",
    children: [
        raw.label("YOUR WALLET", { x: 8, y: 6, width: 160, align: "center", bold: true }),
        raw.slot("balance", { x: 8, y: 20, width: 160, align: "center" }),
        raw.button("earn", {
            transform: { section: "container", x: 4, y: 1, width: 1, height: 1 },
            tooltip: "Earn one persistent coin",
            children: [raw.label("+1", { bold: true })],
        }),
    ],
});
