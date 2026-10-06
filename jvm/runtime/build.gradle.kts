plugins {
    id("buildlogic.library")
    kotlin("plugin.serialization")
}

dependencies {
    api(project(":diagnostics-protocol"))
    compileOnly(libs.minestom)
    compileOnly(libs.slf4j.api)
    api(libs.kotlinx.serialization.core)
    implementation(libs.adventure.minimessage)
    implementation(libs.kotlinx.serialization.json)

    testImplementation(project(":minestom"))
    testImplementation(libs.minestom)
    testImplementation(libs.adventure.minimessage)
    testImplementation(libs.bundles.test)
}

publishing {
    publications.named<MavenPublication>("library") {
        pom {
            name = "Window Runtime"
            description =
                "Runtime for rendering Window UIs from generated pack definitions. Pair it with " +
                "window-minestom or window-multistom."
        }
    }
}
