package dev.oglass.window.codegen.manifest

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

/**
 * Minimal legacy JSON model for compiled Window definitions — only the fields the standalone
 * codegen CLI needs. See `docs/MANIFEST.md` for the full schema.
 *
 * These DTOs are intentionally internal to the codegen module; the runtime owns its own complete
 * model. Unknown fields are ignored so the definition can grow without breaking codegen.
 */
@Serializable
internal data class Manifest(
    val version: Int,
    val windows: Map<String, ManifestWindow> = emptyMap(),
    val huds: Map<String, ManifestHud> = emptyMap(),
)

@Serializable
internal data class ManifestWindow(
    val slots: Map<String, ManifestSlot> = emptyMap(),
    @SerialName("sprite_slots") val spriteSlots: Map<String, ManifestSpriteSlot> = emptyMap(),
    val buttons: Map<String, ManifestButton> = emptyMap(),
    val items: Map<String, ManifestItem> = emptyMap(),
    val collections: Map<String, ManifestCollection> = emptyMap(),
    val inputs: Map<String, ManifestInput> = emptyMap(),
    val groups: Map<String, ManifestRepeatGroup> = emptyMap(),
)

@Serializable internal data class ManifestHud(
    val slots: Map<String, ManifestSlot> = emptyMap(),
)

@Serializable
internal data class ManifestSlot(
    /** Present ⇒ static label, rendered by the runtime; codegen skips it. */
    val text: String? = null,
)

@Serializable
internal data class ManifestButton(
    /** `"close"` ⇒ generate an open member defaulting to `close()`; otherwise abstract. */
    val default: String? = null,
    /** `false` for hover-only hotspots; codegen skips these. */
    val action: Boolean = true,
)

@Serializable internal data class ManifestItem(
    val ignored: String? = null,
)

@Serializable internal data class ManifestSpriteSlot(
    val sprite: String? = null,
)

@Serializable internal data class ManifestCollection(
    val action: Boolean = true,
)

@Serializable internal data class ManifestInput(
    val ignored: String? = null,
)

@Serializable
internal data class ManifestRepeatGroup(
    val count: Int = 0,
    val slots: Map<String, List<String>> = emptyMap(),
    @SerialName("sprite_slots") val spriteSlots: Map<String, List<String>> = emptyMap(),
    val buttons: List<String> = emptyList(),
)

/** JSON reader tolerant of unknown definition fields. */
internal val manifestJson: Json = Json { ignoreUnknownKeys = true }

/** Parse and validate a definition document, rejecting unsupported schema versions. */
internal fun parseManifest(text: String): Manifest {
    val manifest = manifestJson.decodeFromString(Manifest.serializer(), text)
    require(manifest.version == 5) {
        "unsupported manifest version ${manifest.version} (expected 5)"
    }
    return manifest
}
