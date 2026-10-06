plugins { id("buildlogic.library") }

dependencies {
    api(project(":runtime"))
    compileOnly(libs.minestom)
}

publishing {
    publications.named<MavenPublication>("library") {
        pom {
            name = "Window Minestom"
            description = "Window platform for a single Minestom server."
        }
    }
}
