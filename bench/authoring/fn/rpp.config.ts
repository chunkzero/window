import { defineConfig } from "#rpp/config";
import window from "#plugins/window";

export default defineConfig({
    pack: { name: "golden", format: 84 },
    build: { workers: 1, squash: { enabled: false } },
    plugins: [window({ hudShaders: true, kotlin: { package: "golden.jsx", output: "kotlin-out" } })],
});
