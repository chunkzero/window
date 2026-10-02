#!/usr/bin/env bash
# Consume the packed plugin and Maven repository without project substitutions.
set -euo pipefail
: "${RELEASE_VERSION:?Set the Window version}"
repository=$(pwd)
consumer=$(mktemp -d)
trap 'rm -rf "$consumer"' EXIT
mkdir -p "$consumer/plugin" "$consumer/pack/src/window" "$consumer/jvm"
tar -xzf "dist/window-$RELEASE_VERSION.rpp.tgz" -C "$consumer/plugin"
tar -xzf "dist/window-$RELEASE_VERSION-maven.tar.gz" -C "$consumer"
cp scripts/fixtures/release-ui.ts "$consumer/pack/src/window/wallet.ts"
cat > "$consumer/pack/rpp.json" <<'JSON'
{"dependencies":{"window":"path:../plugin"}}
JSON
cat > "$consumer/pack/rpp.config.ts" <<'TS'
import { defineConfig } from "#rpp/config";
import window from "#plugins/window";
export default defineConfig({
    pack: { name: "release-consumer", description: "Window release consumer", packFormat: 84 },
    build: { source: "src", output: "dist" },
    plugins: [window({ namespace: "window", kotlin: { package: "consumer.ui", output: "../jvm/src/main/kotlin/consumer/ui" } })],
});
TS
(cd "$consumer/pack" && rpp codegen && rpp check && rpp build)
test -f "$consumer/pack/dist/assets/window/font/default.json" || find "$consumer/pack/dist/assets/window/font" -name '*.json' | grep -q .
test -f "$consumer/jvm/src/main/kotlin/consumer/ui/WindowPack.kt"
cp jvm/gradle/libs.versions.toml "$consumer/libs.versions.toml"
cat > "$consumer/jvm/settings.gradle.kts" <<'KTS'
pluginManagement { repositories { gradlePluginPortal(); mavenCentral() } }
plugins { id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0" }
dependencyResolutionManagement {
    versionCatalogs { create("libs") { from(files("../libs.versions.toml")) } }
}
rootProject.name = "window-consumer"
KTS
cat > "$consumer/jvm/build.gradle.kts" <<KTS
plugins { kotlin("jvm") version "2.3.10" }
repositories { maven { url = uri("../window-$RELEASE_VERSION-maven") }; mavenCentral() }
kotlin { jvmToolchain(25) }
dependencies {
    implementation("dev.oglass.window:window-runtime:$RELEASE_VERSION")
    implementation(libs.minestom)
}
KTS
"$repository/jvm/gradlew" -p "$consumer/jvm" --no-daemon compileKotlin
