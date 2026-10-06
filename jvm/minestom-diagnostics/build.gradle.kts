plugins { id("buildlogic.library") }

dependencies {
    api(project(":diagnostics-protocol"))
    api(project(":minestom"))
    compileOnly(libs.minestom)
    compileOnly(libs.slf4j.api)

    testImplementation(libs.minestom)
    testImplementation(libs.bundles.test)
}

publishing {
    publications.named<MavenPublication>("library") {
        pom {
            name = "Window Minestom Diagnostics"
            description = "Optional Minestom transport for Window render diagnostics."
        }
    }
}
