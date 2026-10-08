# Window

Window compiles TypeScript-authored Minecraft UIs into resource-pack fonts and typed
Kotlin bindings for a Minestom runtime. See README.md for setup and entry points.

## General guidelines

- Do not edit AGENTS.md or CLAUDE.md unless explicitly asked.
- Keep solutions simple, public APIs narrow, and comments scoped to current behavior.
- Do not hand-edit generated Kotlin or plugin/window.wasm; change their generators.
- Keep Rust, JVM runtime, and standalone inspector development independently usable.
- MC Validation is optional. Only include its checkout when explicitly configured.
- Do not revert unrelated changes or perform unrequested destructive actions.

## Tooling

Install pinned tools with `mise install` and formatter dependencies with
`pnpm install --frozen-lockfile` after tool configuration changes. Use `just`
to list recipes and `just doctor` to check prerequisites. Without mise shell
activation, prefix commands with `mise exec --`. Rust uses rust-toolchain.toml;
JVM tasks use the checked-in Gradle wrapper and Java 25.

- `just fmt` / `just fmt-check`: sources, Gradle scripts, docs, config, and justfile.
- `just lint`: Clippy with warnings denied plus formatting checks.
- `just test-rust -p window-core <filter>`: focused Rust tests.
- `just gradle :runtime:test --tests '*WindowListTest'`: focused JVM tests.
- `just check-ts` / `just test-ts`: TypeScript plugin type-check and tests.
- `just golden`: build the golden cases and verify their `expected.sha256` hashes.
- `just inspector-build`: build and test the standalone inspector.
- `just ready`: all required CI checks before publishing substantial changes.
- `just mc-validation`: opt-in real-client scenario; see docs/VALIDATION.md.

While iterating, run checks appropriate to the change. Avoid full builds and test
suites for isolated edits. Check for existing servers before starting one, and
stop any servers or utilities you start when finished.

## Code style

- Rust: rustfmt at 120 columns; imports grouped as std, external crates, then crate-local.
- Kotlin: standalone ktlint with root .editorconfig; use the version catalog and buildSrc conventions.
- Java: standalone google-java-format AOSP style at 100 columns.
- Docs/config: Oxfmt via pnpm; keep generated output and agent instructions excluded.
- Document public behavior whose contract is not obvious from its signature.
- Test observable behavior and meaningful boundaries; avoid redundant tests.

## Git

Use Conventional Commits, such as `feat(core): ...`, `fix(runtime): ...`,
`chore(setup): ...`, and `docs: ...`. CI is the source of truth; do not rely on
pre-commit hooks. Keep generated output reproducible.

## Glossary

- **rpp**: the resource-pack builder that loads Window's TypeScript/WASM plugin.
- **Compiled definition**: shared compiler/runtime data, emitted as Kotlin pack classes.
- **Surface**: the vanilla container or HUD channel that carries a UI.
- **Spacer**: a font glyph that moves the text cursor without drawing.
- **Net-zero segment**: a glyph run that returns the cursor to its starting position.
- **Slot ownership / routing**: the control writing an inventory item and the control
  receiving its clicks; a repeater cell can assign these to different controls.
- **MC Validation**: the optional external harness for real Minecraft client scenarios.
