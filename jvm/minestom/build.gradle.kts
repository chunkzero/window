plugins { id("buildlogic.library") }

dependencies {
    api(project(":runtime"))
    compileOnly(libs.minestom)

    testImplementation(project(":host-testkit"))
    testImplementation(libs.minestom)
    testImplementation(libs.bundles.test)
}

publishing {
    publications.named<MavenPublication>("library") {
        pom {
            name = "Window Minestom"
            description = "Minestom host for Window UIs."
        }
    }
}
