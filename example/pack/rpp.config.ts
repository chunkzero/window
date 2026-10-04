import { defineConfig } from "#rpp/config";
import window from "#plugins/window";

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
            hudShaders: true,
            anvilFieldSprite: "search_field",
            kotlin: {
                package: "dev.oglass.window.example.generated",
                output: "../../jvm/example/src/main/kotlin/dev/oglass/window/example/generated",
            },
        }),
    ],
});
