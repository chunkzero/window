# Releasing

Window publishes its TypeScript/WASIp2 rpp plugin, a versioned Maven repository archive (runtime, diagnostics protocol
and Minestom diagnostics, including POMs, sources and Javadoc). Window has no native CLI and is not an aqua package.
TypeScript package files replace the former Lua package; all artifacts use the same version and include SHA-256
sidecars. `release.json` records the full source commit.

## Publish

RPP must already have the release pinned by `RPP_VERSION` in the release workflow. The organization secrets
`REGISTRY_APP_ID` and `REGISTRY_APP_PRIVATE_KEY` grant the publishing job access to `chunkzero/rpp-registry`; no PR job
has release write credentials. The repository secret `MAVEN_R2_TOKEN` authorizes publication of the matching JVM
libraries to `https://maven.chunkzero.com` through Maven R2.

Push `v<version>` matching `Cargo.toml` for a stable/beta release, or manually run `Release` on `main` with
`mode=nightly` and `publish=true`. Dispatch with `publish=false` builds and checks artifacts without publishing. Only
`main` and version tags are supported publishing refs. The plugin manifest and JVM version are set in the build
workspace to the coordinated release version.

Daily builds skip when the most recently published nightly has the same source SHA. Drafts do not count as published.
Nightlies use `v0.1.0-nightly.<UTC date>.g<12-character commit>`, never a rolling tag. All releases first upload to a
draft, download and compare every asset, then become public. Prereleases use `latest=false`. Publication is serialized;
retries can resume drafts or verify already-published assets and recover the registry PR.

The consumer job unpacks the actual plugin, loads its WASIp2 compiler, type-checks and builds an authored UI, then
compiles its generated Kotlin against the packaged Maven repository with no composite build or source substitutions.

## Install a pinned nightly

After its registry PR has been reviewed and merged:

```sh
rpp add window@0.1.0-nightly.20261001.g0123456789ab
rpp build
```

Commit `rpp.json` and `rpp.lock`. Use the exact published version, including the commit suffix. The JVM runtime must use
the same version. Configure Gradle with `maven("https://maven.chunkzero.com")` and Maven Central, then depend on
`com.chunkzero.window:window-runtime:<version>`. Each release publishes the runtime and its Window dependencies
together.

For an offline mirror, download `window-<version>-maven.tar.gz` and its `.sha256` from that tag, verify the checksum,
and extract it. Point a Gradle Maven repository at `window-<version>-maven`; Maven Central supplies third-party
dependencies.

Only the latest 30 published nightlies are retained. Stable and beta releases are never pruned. Deleted nightly versions
cannot be freshly installed, even with an old lockfile; use a stable/beta release for long-lived deployments or retain
the verified artifacts.
