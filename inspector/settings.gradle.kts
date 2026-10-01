pluginManagement {
    repositories {
        maven("https://maven.fabricmc.net/")
        mavenCentral()
        gradlePluginPortal()
    }
}

plugins { id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0" }

rootProject.name = "window-inspector"

val mcValidationRoot =
    providers
        .gradleProperty("mcValidationRoot")
        .orElse(providers.environmentVariable("MC_VALIDATION_ROOT"))

if (mcValidationRoot.isPresent) {
    includeBuild(file(mcValidationRoot.get()))
}
