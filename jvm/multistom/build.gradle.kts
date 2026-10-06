plugins { id("buildlogic.library") }

dependencies {
    api(project(":runtime"))
    compileOnly(libs.multistom)

    testImplementation(libs.multistom)
    testImplementation(libs.bundles.test)
}

publishing {
    publications.named<MavenPublication>("library") {
        pom {
            name = "Window Multistom"
            description = "Window platform for Multistom, where each player belongs to a server process."
        }
    }
}
