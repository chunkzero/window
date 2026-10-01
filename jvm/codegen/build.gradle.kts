plugins {
    id("buildlogic.common")
    kotlin("plugin.serialization")
    application
}

application { mainClass = "dev.oglass.window.codegen.MainKt" }

dependencies {
    implementation(libs.kotlinx.serialization.json)

    testImplementation(libs.bundles.test)
}
