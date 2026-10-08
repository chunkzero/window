import { defineConfig } from "rpp:config";
import window from "plugin:window";

export default defineConfig({
    pack: { name: "golden", format: 84 },
    build: { workers: 1, squash: { enabled: false } },
    plugins: [
        // Shares the jsx project's Kotlin package so both styles must produce identical output.
        window({ hudShaders: true, kotlin: { package: "golden.jsx", output: "kotlin-out", target: "agnostic" } }),
    ],
});
