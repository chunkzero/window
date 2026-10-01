# Releasing

Window's rpp plugin is published to [chunkzero/rpp-registry](https://github.com/chunkzero/rpp-registry) by CI.
`plugin/window.wasm` is built during the release and is not committed.

## Prerequisites

- An rpp release matching `RPP_VERSION` in `.github/workflows/release.yml`. Bump it when a newer rpp is required.
- The `RPP_APP_ID` and `RPP_APP_PRIVATE_KEY` organization secrets for chunkzero's GitHub App, which is installed on
  `chunkzero/rpp-registry`. The workflow exchanges them for a short-lived token, and the app's bot opens the registry
  pull request.

## Cut a release

1. Set `version` in `plugin/rpp.json` and merge the change to `main`.
2. Tag the merge commit `v<version>` and push the tag, for example `git tag v0.1.0 && git push origin v0.1.0`.
3. The `Release` workflow builds the plugin, packs it, uploads the archive to the tag's GitHub release, and opens a pull
   request against `chunkzero/rpp-registry`.
4. A maintainer reviews and merges the registry pull request. The version is installable once it merges.

The workflow fails if the tag does not match the plugin version. Re-running it for a published version succeeds only
when the packed bytes are identical; otherwise bump the version.
