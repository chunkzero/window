# Validation

Window validates the same contract at three boundaries: pure compiler tests, headless runtime and host tests, and a real
Minecraft client. A test at only one boundary is insufficient because resource activation, protocol input, glyph
geometry, and shaders exist only in the client.

## Deterministic checks

```bash
just test                 # Rust, JVM, and TypeScript tests
just inspector-build      # inspector build and tests
just ready                # all required CI checks
```

`window_core::validation::validate_compile_output` validates an in-memory compiler result as a complete artifact set. It
checks deterministic paths, schema-v1 descriptor regeneration and fingerprints, font providers and bitmap metrics,
spacer agreement, cursor contracts, shifted-font coverage, sprites, inventory ownership, and collection references.

```rust
let output = window_core::pipeline::compile_project_json(project, &input)?;
window_core::validation::validate_compile_output(&output).assert_valid();
```

Rust tests additionally cover layout, rasterization, alpha compositing, codepoint allocation, authoring validation,
codegen, HUD shaders, and the end-to-end shop fixture. Kotlin tests cover title/HUD composition, measurement,
reactivity, bindings, click routing, and paging against an in-memory host. Each host runs `window-host-testkit`'s
conformance suite (slot normalization, player-slot leases, anvil reopens, ping ordering, HUD channels) against its real
server through a recording connection.

## Real Minecraft validation

```bash
just mc-validation
```

This is the only real-client runner. The recipe:

1. generates the deterministic fixture under `build/generated/windowTestResources/`;
2. builds the disposable Minestom validation server;
3. builds `window-inspector` with its optional MC Validation extension;
4. mounts the generated pack assets and inspector into MC Validation; and
5. runs `validation/scenarios/window-smoke.json` at an exact framebuffer size.

The scenario uses shared built-ins for lifecycle, screen transitions, typed client/inventory state, real container
clicks, Unicode typing, GUI scale, checkpoints, screenshots, and correlated server events. The inspector supplies these
namespaced steps:

- `window:assert_descriptor` validates the descriptor and every referenced active resource fingerprint;
- `window:wait_frame` selects an unconsumed diagnostics frame by surface, semantic id, reason, session, and monotonic
  frame id;
- `window:assert_layer` checks the authored font/bounds against Minecraft's prepared glyph geometry; and
- `window:capture_layer` exports real framebuffer pixels owned by the observed glyph rectangles.

Structured descriptor, frame, and glyph records are retained as hashed JSON artifacts in the shared report. PNGs,
semantic client state, text observations, render-pass boundaries, server events, and process logs live under
`build/reports/mc-validation/`.

The scenario currently covers production container opening, active pack fingerprints, static chrome, dynamic text,
sprites, container/hotbar item sync, repeater cells that render a real item in one of their own slots while every cell
slot still routes clicks to the cell button, left/right/shift/hotbar clicks, reactive title invalidation, anvil input,
HUD diagnostics, GUI scale 2, and clean disconnect.

The `just mc-validation` recipe defaults to the sibling `../mc-validation` checkout and passes its absolute path to both
Gradle builds. Override it with:

```bash
MC_VALIDATION_ROOT=/path/to/mc-validation just mc-validation
```

For direct Gradle commands, set the same environment variable or pass `-PmcValidationRoot=/path/to/mc-validation` (the
property takes precedence). Without either setting, the JVM build omits `validation-server` and the inspector omits the
extension and its entry point. Normal checks need no external checkout.

`just inspector-validation-build` produces the optional extension build as
`inspector/build/libs/window-inspector-0.1.0-alpha.0-validation.jar`. The standalone inspector JAR remains the artifact
to install for normal debugging.

On headless Linux, run the recipe under `xvfb-run --auto-servernum`.

## Adding a regression fixture

Extend `crates/window-core/examples/generate_minecraft_test.rs`, add the production server behavior in
`jvm/validation-server`, and express the client flow in `validation/scenarios/window-smoke.json`. Prefer shared
built-ins; extend `WindowValidationExtension` only for Window-owned renderer or protocol semantics that the generic
contract cannot represent.

Do not check in workstation screenshots when the compiler can emit a canonical asset or the scenario can retain
deterministic semantic evidence.
