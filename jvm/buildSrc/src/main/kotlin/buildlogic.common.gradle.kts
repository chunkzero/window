plugins { kotlin("jvm") }

group = "dev.oglass.window"

version = "0.1.0-alpha.0"

repositories { mavenCentral() }

java { toolchain { languageVersion.set(JavaLanguageVersion.of(25)) } }

kotlin { jvmToolchain(25) }

tasks.test { useJUnitPlatform() }
