plugins { kotlin("jvm") }

group = "dev.oglass.window"

version = providers.gradleProperty("windowVersion").orElse("0.1.0-alpha.0").get()

repositories { mavenCentral() }

java { toolchain { languageVersion.set(JavaLanguageVersion.of(25)) } }

kotlin { jvmToolchain(25) }

tasks.test { useJUnitPlatform() }

tasks.withType<AbstractArchiveTask>().configureEach {
    isPreserveFileTimestamps = false
    isReproducibleFileOrder = true
}

tasks.withType<Jar>().configureEach {
    manifest.attributes("Implementation-Version" to project.version)
}
