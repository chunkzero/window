plugins {
    id("buildlogic.library")
    kotlin("plugin.serialization")
}

dependencies {
    api(libs.kotlinx.serialization.core)
    implementation(libs.kotlinx.serialization.json)

    testImplementation(libs.bundles.test)
}

publishing {
    publications.named<MavenPublication>("library") {
        pom {
            name = "Window Diagnostics Protocol"
            description = "Versioned protocol models for Window render diagnostics."
        }
    }
}
