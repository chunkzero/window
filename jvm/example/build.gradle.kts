plugins {
    id("buildlogic.common")
    application
}

application { mainClass = "com.chunkzero.window.example.MainKt" }

dependencies {
    implementation(libs.minestom)
    implementation(libs.adventure.minimessage)
    implementation(libs.kotlinx.coroutines)
    implementation(libs.kotlinx.serialization.json)

    implementation(project(":runtime"))
}
