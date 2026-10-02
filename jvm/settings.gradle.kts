pluginManagement {
    repositories {
        mavenCentral()
        gradlePluginPortal()
    }
}

plugins { id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0" }

rootProject.name = "window"

val mcValidationRoot =
    providers
        .gradleProperty("mcValidationRoot")
        .orElse(providers.environmentVariable("MC_VALIDATION_ROOT"))

if (mcValidationRoot.isPresent) {
    includeBuild(file(mcValidationRoot.get()))
    include("validation-server")
}

include("runtime", "diagnostics-protocol", "minestom-diagnostics", "example")
