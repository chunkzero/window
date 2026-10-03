import groovy.json.JsonOutput
import groovy.json.JsonSlurper

plugins {
    java
    id("net.fabricmc.fabric-loom") version "1.17.13"
}

group = "dev.oglass.window"

version = "0.1.0-alpha.0"

val withValidation =
    providers
        .gradleProperty("mcValidationRoot")
        .orElse(providers.environmentVariable("MC_VALIDATION_ROOT"))
        .isPresent

repositories {
    maven("https://maven.fabricmc.net/")
    mavenCentral()
}

dependencies {
    minecraft("com.mojang:minecraft:26.2")
    implementation("net.fabricmc:fabric-loader:0.19.5")
    implementation("net.fabricmc.fabric-api:fabric-api:0.161.0+26.2")
    if (withValidation) {
        compileOnly("dev.rpp.mcvalidation:core:0.1.0-SNAPSHOT")
        compileOnly("dev.rpp.mcvalidation:fabric-client:0.1.0-SNAPSHOT")
    }

    testImplementation("org.junit.jupiter:junit-jupiter:5.13.4")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher:1.13.4")
}

java {
    toolchain.languageVersion = JavaLanguageVersion.of(25)
    withSourcesJar()
}

if (withValidation) {
    sourceSets.main { java.srcDir("src/validation/java") }
    tasks.jar { archiveClassifier.set("validation") }
}

tasks.processResources {
    inputs.property("version", project.version)
    inputs.property("withValidation", withValidation)
    filesMatching("fabric.mod.json") { expand("version" to project.version) }
    if (withValidation) {
        doLast {
            val metadata = destinationDir.resolve("fabric.mod.json")
            val descriptor = JsonSlurper().parse(metadata) as Map<*, *>
            val entrypoints = descriptor["entrypoints"] as Map<*, *>
            val validation = listOf("dev.oglass.window.inspector.WindowValidationExtension")
            val updated =
                descriptor + ("entrypoints" to (entrypoints + ("mc-validation" to validation)))
            metadata.writeText(JsonOutput.prettyPrint(JsonOutput.toJson(updated)) + "\n")
        }
    }
}

tasks.test { useJUnitPlatform() }
