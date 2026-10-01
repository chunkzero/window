plugins {
    id("buildlogic.common")
    id("com.gradleup.shadow")
    application
}

application { mainClass = "dev.oglass.window.validationserver.MainKt" }

dependencies {
    implementation(libs.minestom)
    implementation(libs.kotlinx.serialization.json)
    implementation(project(":runtime"))
    implementation(project(":minestom-diagnostics"))
    implementation("dev.rpp.mcvalidation:minestom:0.1.0-SNAPSHOT")
    runtimeOnly("org.slf4j:slf4j-simple:2.0.18")
}

tasks.shadowJar {
    archiveFileName.set("window-validation-server.jar")
    mergeServiceFiles()
    manifest { attributes["Main-Class"] = application.mainClass.get() }
}
