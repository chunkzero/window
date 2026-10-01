import { defineConfig } from "#rpp/config";
import window from "#plugins/window";

export default defineConfig({
    pack: {
        name: "window-example",
        description: "Example resource pack driven by the Window rpp plugin.",
        packFormat: 84,
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
            kotlin: {
                package: "dev.oglass.window.example.generated",
                output: "../../jvm/example/src/main/kotlin/dev/oglass/window/example/generated",
            },
        }),
    ],
});
