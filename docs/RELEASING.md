# Releasing

Window publishes its TypeScript/WASIp2 rpp plugin and a Maven repository archive of its JVM libraries (runtime,
diagnostics protocol and Minestom diagnostics, with POMs, sources and Javadoc). Every artifact of a build uses the same
version and has a SHA-256 sidecar; `release.json` records the full source commit.

## Versions

Versions follow the [chunkzero release scheme](https://github.com/chunkzero/release-tools). `Cargo.toml`'s workspace
version is the upcoming release; bump it after each release.

| Channel | Version                                               | Maven repository                        |
| ------- | ----------------------------------------------------- | --------------------------------------- |
| Release | the workspace version, e.g. `0.1.0-alpha.0`           | `https://maven.chunkzero.com`           |
| Nightly | `0.1.0-nightly.<UTC commit time>.g<12-character sha>` | `https://maven.chunkzero.com/nightlies` |

A commit always gets the same nightly version, published releases never change and none are deleted, so pinned versions
and lockfiles keep working.

## Publish

The `Release` workflow publishes a nightly from `main` every day, skipping a commit that is already published. To
publish manually, run it on `main` with `channel=nightly` or `channel=release`; on other branches it only builds and
checks the artifacts. It publishes the Maven libraries first, then the GitHub release, then opens a registry PR in
`chunkzero/rpp-registry`. Prereleases are never marked latest.

It uses the organization secrets `MAVEN_R2_TOKEN` for Maven and `REGISTRY_APP_ID`/`REGISTRY_APP_PRIVATE_KEY` for the
registry. The workflow packs and checks the plugin with the rpp pinned in `mise.toml`; its `RPP_VERSION` must match.

Before publishing, the workflow unpacks the packed plugin, builds an authored UI with it, and compiles the generated
Kotlin against the packaged Maven repository.

## Install a pinned version

After its registry PR merges:

```sh
rpp add window@0.1.0-nightly.20261005021334.g0123456789ab
rpp build
```

Commit `rpp.json` and `rpp.lock`. Depend on `com.chunkzero.window:window-minestom` or `window-multistom` at the same
version, from the repository for its channel:

```kotlin
repositories {
    maven("https://maven.chunkzero.com/nightlies") // nightlies only
    maven("https://maven.chunkzero.com")
    mavenCentral()
}
```

For an offline mirror, download `window-<version>-maven.tar.gz` and its `.sha256` from the release, verify and extract
it, and point a Gradle Maven repository at `window-<version>-maven`.
