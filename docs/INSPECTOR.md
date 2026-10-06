# Window Render Inspector

Window Render Inspector is the distributable Fabric client mod for diagnosing a Window pack through Minecraft's real
font, GUI, and shader pipeline. It does not contain a preview renderer. The compiler describes what should render, an
optional server adapter sends the dynamic composition trace, and the inspector records what the active Minecraft client
actually rendered.

The inspector targets the same version as Window's validation harness: Minecraft 26.2 with Fabric Loader and Fabric API.

## Build and install

```bash
just inspector-build
```

Copy the remapped `window-inspector` JAR from `inspector/build/libs/` into the client's `mods/` directory alongside
compatible Fabric Loader and Fabric API versions. The mod is client-only; a vanilla or otherwise unmodified server can
be used in passive mode.

For local development, this starts a Fabric client with the inspector installed:

```bash
just inspector-run
```

## Controls

- `F7` toggles the inspector overlay.
- `Page Down` selects the next semantic layer in the current server render frame.
- `F8` explicitly exports a local report.

All keys are ordinary Minecraft key mappings and can be rebound under Controls. The overlay shows the current mode,
active-pack descriptor state, selected semantic layer, expected and observed font/geometry/cursor data, and the first
actionable mismatch.

## Passive and enhanced modes

Passive mode needs only the inspector. On resource reload it discovers `assets/<namespace>/window/debug.json` through
Minecraft's active `ResourceManager`, validates its schema and generated-resource fingerprints, and therefore checks the
pack that actually won resource resolution. A descriptor found on disk but shadowed by another active pack is not
treated as loaded truth.

Enhanced mode is negotiated after joining a server with the optional Minestom adapter. The client initiates the
versioned capability handshake; the server sends nothing to vanilla or non-debug clients. Accepted clients receive only
bounded, dynamic `RenderFrame` expectations. The static definition, resource fingerprints, and masks remain in the
loaded pack descriptor and are not retransmitted per frame.

Frames are correlated by render-session id, monotonic frame id, surface identity, and the container id or HUD channel.
The adapter can attach the descriptor's SHA-256 pack fingerprint so the client can reject expectations for a different
active pack.

## Minestom setup

Add the optional `window-minestom-diagnostics` artifact next to `window-minestom`. Read `pack_fingerprint.value` from
the compiler-emitted debug descriptor belonging to the pack you serve, then install the adapter. Installing registers it
as the global `WindowDiagnostics.observer`; closing unregisters it:

```kotlin
import com.chunkzero.window.diagnostics.PackFingerprint
import com.chunkzero.window.diagnostics.minestom.MinestomDiagnostics

val diagnostics =
    MinestomDiagnostics.install(
        PackFingerprint("sha256", descriptorPackFingerprint),
    )

Runtime.getRuntime().addShutdownHook(Thread(diagnostics::close))
```

Every window `open()`, reactive title update, and HUD `show()` then emits traces from the same composition result sent
to the player; the adapter sends them only to `MinestomHost` players. Other observers can be registered by assigning
`WindowDiagnostics.observer` directly. Observer failures are isolated from delivery. The adapter accepts at most 256 KiB
per payload, negotiates the smaller client/server limit, bounds every field and collection, ignores malformed input, and
drops oversized output.

## Reports and privacy

The inspector never lets a server force a file write or upload. Pressing the export key creates
`window-reports/<UTC timestamp>/` inside the local Minecraft game directory. Review it before sharing because rendered
UI text and environment data may be private.

An export contains:

- a versioned `report.json` summary;
- active descriptors, source pack ids, fingerprints, and validation status;
- negotiated render frames and real Minecraft font/glyph observations;
- Minecraft and inspector versions, GUI scale, language, Unicode-font setting, window/framebuffer sizes, GPU renderer,
  and active pack order;
- a real framebuffer screenshot; and
- isolated/cropped layer images when descriptor masks or observed bounds make the isolation valid.

No network destination or automatic uploader exists. The inspector retains only bounded in-memory histories, and
disconnecting clears negotiated server frames.

## Adding a regression fixture

`just mc-validation` builds this inspector with its optional validation extension. The extension is enabled by
`MC_VALIDATION_ROOT` or `-PmcValidationRoot`; standalone builds do not require the harness. See
[VALIDATION.md](VALIDATION.md) for setup. Contributors should extend
`crates/window-core/examples/generate_minecraft_test.rs`, which deterministically generates the resource pack, debug
descriptor, and canonical opaque images used by the real client test.

For a new fixture:

1. author it in the generator and add its semantic geometry to `probe.json`;
2. assert descriptor/manifest byte consistency in `window-core`;
3. make the validation server exercise the production view and diagnostics adapter;
4. assert negotiation and frame/session correlation through the inspector;
5. use exact pixels for generated opaque chrome/sprites and bounded geometry for vanilla text or GPU-sensitive output;
   and
6. export a named failure artifact. Do not check in workstation screenshots when the compiler can generate the canonical
   expectation.

Run `just ready` for normal changes. For real-client changes, also run `just mc-validation` with a configured checkout
and a display (or Xvfb).
