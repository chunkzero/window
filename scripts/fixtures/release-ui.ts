import { button, label, slot, ui } from "#plugins/window";

export default ui({
    name: "wallet",
    container: "generic_9x3",
    children: [
        label("YOUR WALLET", { x: 8, y: 6, width: 160, align: "center", bold: true }),
        slot("balance", { x: 8, y: 20, width: 160, align: "center" }),
        button("earn", {
            transform: { section: "container", x: 4, y: 1, width: 1, height: 1 },
            tooltip: "Earn one persistent coin",
            children: [label("+1", { bold: true })],
        }),
    ],
});
