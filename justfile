# Run `just` to list the repository's development commands.
set shell := ["bash", "-euo", "pipefail", "-c"]

gradlew := justfile_directory() / "jvm" / "gradlew"
mc_validation := absolute_path(env_var_or_default("MC_VALIDATION_ROOT", justfile_directory() / "../mc-validation"))
rpp := env_var_or_default("RPP", "rpp")

_default:
    @just --list

# Check local prerequisites; rpp and MC Validation are needed only for integration tasks.
doctor:
    @bash scripts/doctor.sh

# Type-check the Rust workspace.
check:
    cargo check --locked --workspace --all-targets

# Run Rust tests; accepts package selectors and test filters.
test-rust *ARGS:
    cargo test --locked --workspace {{ ARGS }}

fmt-rust:
    cargo fmt --all

fmt-check-rust:
    cargo fmt --all -- --check

lint-rust:
    cargo clippy --locked --workspace --all-targets -- -D warnings

# Build the compiler component for the local rpp plugin package.
plugin:
    cargo build --locked -p window-rpp-plugin --release --target wasm32-wasip2
    cp target/wasm32-wasip2/release/window_rpp_plugin.wasm plugin/window.wasm

check-wasm:
    cargo check --locked -p window-rpp-plugin --target wasm32-wasip2

# Build the example pack and regenerate its Kotlin bindings.
example-pack: plugin
    cd example/pack && {{ rpp }} codegen && {{ rpp }} build

# Run the example server after building its pack.
example-run:
    "{{ gradlew }}" -p jvm :example:run

# Run a Gradle task, e.g. `just gradle :runtime:test --tests '*WindowPagerTest'`.
gradle *ARGS:
    "{{ gradlew }}" -p jvm {{ ARGS }}

test-jvm *ARGS:
    "{{ gradlew }}" -p jvm test {{ ARGS }}

fmt-jvm:
    ktlint --format 'jvm/**/*.kt' 'jvm/**/*.kts' '!**/build/**' '!**/generated/**'

fmt-check-jvm:
    ktlint 'jvm/**/*.kt' 'jvm/**/*.kts' '!**/build/**' '!**/generated/**'

# Build and test the standalone Fabric inspector.
inspector-build *ARGS:
    "{{ gradlew }}" -p inspector build {{ ARGS }}

inspector-run:
    "{{ gradlew }}" -p inspector runClient

fmt-inspector:
    ktlint --format 'inspector/*.kts'
    git ls-files -z --cached --others --exclude-standard -- 'inspector/src/**/*.java' | xargs -0 google-java-format --aosp --replace

fmt-check-inspector:
    ktlint 'inspector/*.kts'
    git ls-files -z --cached --others --exclude-standard -- 'inspector/src/**/*.java' | xargs -0 google-java-format --aosp --dry-run --set-exit-if-changed

# Generate the rpp SDK and component typings the TypeScript plugin type-checks against.
codegen: plugin
    cd plugin && {{ rpp }} codegen

# Type-check the TypeScript plugin sources and tests.
check-ts: codegen
    pnpm check:ts

test-ts:
    pnpm test:ts

# Build every golden case with the TypeScript plugin and verify expected.sha256 Pass `--update` to rewrite the hashes.
golden *ARGS: plugin
    pnpm golden {{ ARGS }}

# Benchmark JSX and function-style authoring builds, e.g. `just bench --runs 5 --scenarios noop --cold-wasm`.
bench *ARGS: plugin
    RPP="{{ rpp }}" node bench/authoring/bench.ts {{ ARGS }}

fmt-docs:
    pnpm fmt

fmt-check-docs:
    pnpm fmt:check

fmt-just:
    just --unstable --fmt

fmt-check-just:
    just --unstable --fmt --check

# Generate the deterministic pack consumed by real-client validation.
validation-resources:
    cargo run --locked --quiet -p window-core --example generate_minecraft_test -- "{{ justfile_directory() }}/build/generated/windowTestResources"

# Build the same inspector with its optional MC Validation extension.
inspector-validation-build:
    MC_VALIDATION_ROOT="{{ mc_validation }}" "{{ gradlew }}" -p inspector build

# Run the opt-in real Minecraft scenario (requires MC Validation and a display).
mc-validation: validation-resources inspector-validation-build
    MC_VALIDATION_ROOT="{{ mc_validation }}" "{{ gradlew }}" -p jvm :validation-server:shadowJar
    "{{ mc_validation }}/mc-validation" agent "{{ justfile_directory() }}/validation/scenarios/window-smoke.json" --server-jar "{{ justfile_directory() }}/jvm/validation-server/build/libs/window-validation-server.jar" --mod "{{ justfile_directory() }}/inspector/build/libs/window-inspector-0.1.0-alpha.0-validation.jar" --resource-pack "{{ justfile_directory() }}/build/generated/windowTestResources" --define "MANIFEST={{ justfile_directory() }}/build/generated/windowTestResources/window-test/manifest.json" --define "DESCRIPTOR={{ justfile_directory() }}/build/generated/windowTestResources/assets/window/window/debug.json" --output "{{ justfile_directory() }}/build/reports/mc-validation"

# Format sources, configuration, and documentation.
fmt: fmt-rust fmt-jvm fmt-inspector fmt-docs fmt-just

fmt-check: fmt-check-rust fmt-check-jvm fmt-check-inspector fmt-check-docs fmt-check-just

lint: lint-rust fmt-check

# Run deterministic tests. Use ecosystem recipes to scope individual changes.
test: test-rust test-jvm test-ts

# These recipes are also the GitHub Actions jobs.
ci-rust: lint-rust test-rust check-wasm

ci-jvm:
    "{{ gradlew }}" -p jvm build

ci-ts: check-ts test-ts

ci-inspector:
    "{{ gradlew }}" -p inspector build

# All required CI checks. Run before publishing substantial changes.
ready: fmt-check ci-rust ci-jvm ci-ts ci-inspector

ci: ready

# Full validation, including the opt-in real Minecraft scenario.
ci-full: ready mc-validation
