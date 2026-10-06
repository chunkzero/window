plugins {
    id("buildlogic.library")
    kotlin("plugin.serialization")
}

dependencies {
    api(project(":diagnostics-protocol"))
    compileOnly(libs.adventure.api)
    compileOnly(libs.slf4j.api)
    api(libs.kotlinx.serialization.core)
    implementation(libs.adventure.minimessage)
    implementation(libs.kotlinx.serialization.json)

    testImplementation(libs.adventure.api)
    testImplementation(libs.adventure.text.serializer.plain)
    testImplementation(libs.bundles.test)
    testRuntimeOnly(libs.slf4j.api)
}

publishing {
    publications.named<MavenPublication>("library") {
        pom {
            name = "Window Runtime"
            description =
                "Server-agnostic runtime for rendering Window UIs from generated pack definitions."
        }
    }
}
