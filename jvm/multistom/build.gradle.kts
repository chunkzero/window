plugins { id("buildlogic.library") }

dependencies {
    api(project(":runtime"))
    compileOnly(libs.multistom)

    testImplementation(project(":host-testkit"))
    testImplementation(libs.multistom)
    testImplementation(libs.bundles.test)
}

publishing {
    publications.named<MavenPublication>("library") {
        pom {
            name = "Window Multistom"
            description = "Multistom host for Window UIs."
        }
    }
}
