plugins {
    id("buildlogic.common")
    `java-library`
    id("org.jetbrains.dokka")
    `maven-publish`
}

java {
    withJavadocJar()
    withSourcesJar()
}

publishing {
    publications {
        create<MavenPublication>("library") {
            from(components["java"])
            artifactId = "window-${project.name}"
            pom {
                url = "https://github.com/chunkzero/window"
                licenses {
                    license {
                        name = "MIT License"
                        url = "https://opensource.org/license/mit/"
                    }
                    license {
                        name = "Apache License, Version 2.0"
                        url = "https://www.apache.org/licenses/LICENSE-2.0"
                    }
                }
                developers {
                    developer {
                        id = "chunkzero"
                        name = "chunkzero"
                    }
                }
                issueManagement {
                    system = "GitHub"
                    url = "https://github.com/chunkzero/window/issues"
                }
                scm {
                    connection = "scm:git:https://github.com/chunkzero/window.git"
                    developerConnection = "scm:git:ssh://git@github.com/chunkzero/window.git"
                    url = "https://github.com/chunkzero/window"
                }
            }
        }
    }
    repositories {
        maven {
            url =
                rootProject.layout.buildDirectory
                    .dir("staging-deploy")
                    .get()
                    .asFile
                    .toURI()
        }
    }
}
