plugins {
    id("buildlogic.common")
    application
}

application { mainClass = "dev.oglass.window.example.MainKt" }

dependencies {
    implementation(libs.minestom)
    implementation(libs.adventure.minimessage)
    implementation(libs.kotlinx.coroutines)
    implementation(libs.kotlinx.serialization.json)

    implementation(project(":runtime"))
}
