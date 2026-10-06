plugins { id("buildlogic.library") }

dependencies {
    api(project(":runtime"))
    api(libs.bundles.test)
    compileOnly(libs.adventure.api)
}

publishing {
    publications.named<MavenPublication>("library") {
        pom {
            name = "Window Host Test Kit"
            description = "Shared conformance tests for Window server hosts."
        }
    }
}
