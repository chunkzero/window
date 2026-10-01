plugins { `kotlin-dsl` }

repositories {
    gradlePluginPortal()
    mavenCentral()
}

dependencies {
    implementation(libs.plugin.kotlin)
    implementation(libs.plugin.shadow)
    implementation(libs.plugin.dokka)
    implementation(libs.plugin.serialization)
}

kotlin { jvmToolchain(25) }
