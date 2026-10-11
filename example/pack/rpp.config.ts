import { defineConfig } from "rpp:config";
import window from "plugin:window";

export default defineConfig({
    pack: {
        name: "window-example",
        description: "Window example — custom-font inventory UI",
        format: 88,
    },
    build: {
        source: "src",
        output: "dist",
        workers: 0,
        squash: { enabled: true, engine: "builtin", json: true, png: "fast", zip: true },
    },
    plugins: [
        window({
            namespace: "window",
            coreShaders: { hud: true, hover: true },
            vanilla: { anvil: { field: "search_field" }, hud: { experience: false }, slotHighlight: false },
            size: { report: true, budget: 1_000_000 },
            previews: { output: "build/previews" },
            kotlin: {
                package: "com.chunkzero.window.example.generated",
                output: "../../jvm/example/src/main/kotlin/com/chunkzero/window/example/generated",
                target: "minestom",
            },
        }),
    ],
});
