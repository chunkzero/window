package com.chunkzero.window.diagnostics

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** Expected render state produced by the real Window server composition pass. */
@Serializable
public data class RenderFrame(
    @SerialName("schema_version")
    public val schemaVersion: Int = DiagnosticsProtocol.SCHEMA_VERSION,
    @SerialName("frame_id") public val frameId: Long,
    public val reason: RenderFrameReason,
    public val correlation: RenderCorrelation,
    @SerialName("cursor_convention") public val cursorConvention: RenderCursorConvention,
    @SerialName("cursor_start") public val cursorStart: Int,
    @SerialName("cursor_end") public val cursorEnd: Int,
    @SerialName("net_cursor_delta") public val netCursorDelta: Int,
    @SerialName("pack_fingerprint") public val packFingerprint: PackFingerprint? = null,
    public val layers: List<RenderLayerTrace>,
) {
    internal fun validate() {
        require(schemaVersion == DiagnosticsProtocol.SCHEMA_VERSION) {
            "Unsupported render frame schema version $schemaVersion"
        }
        require(frameId > 0) { "Frame id must be positive" }
        correlation.validate()
        require(netCursorDelta == cursorEnd - cursorStart) { "Invalid frame net cursor delta" }
        packFingerprint?.validate()
        require(layers.size <= 4096) { "Render frame contains too many layers" }
        require(layers.map(RenderLayerTrace::semanticId).toSet().size == layers.size) {
            "Render frame semantic layer ids must be unique"
        }
        layers.forEach(RenderLayerTrace::validate)
    }
}

/** How semantic layer segments participate in a frame's cursor reset. */
@Serializable
public enum class RenderCursorConvention {
    /** Every layer contains its own leading and trailing reset to the surface origin. */
    @SerialName("independent_net_zero_segments")
    INDEPENDENT_NET_ZERO_SEGMENTS,

    /** Layers share a cursor and the completed frame performs one reset to its fixed width. */
    @SerialName("fixed_width_composition")
    FIXED_WIDTH_COMPOSITION,
}

@Serializable
public enum class RenderFrameReason {
    @SerialName("open")
    OPEN,

    @SerialName("reactive_update")
    REACTIVE_UPDATE,
}

@Serializable
public enum class RenderSurfaceKind {
    @SerialName("window")
    WINDOW,

    @SerialName("hud")
    HUD,
}

/** Robust key joining a frame to a concrete server session and client surface. */
@Serializable
public data class RenderCorrelation(
    @SerialName("surface_kind") public val surfaceKind: RenderSurfaceKind,
    @SerialName("semantic_id") public val semanticId: String,
    @SerialName("render_session_id") public val renderSessionId: String,
    @SerialName("container_id") public val containerId: Int? = null,
    @SerialName("hud_channel") public val hudChannel: String? = null,
) {
    internal fun validate() {
        require(semanticId.length in 1..256) { "Invalid semantic id" }
        require(renderSessionId.length in 1..128) { "Invalid render session id" }
        require(containerId == null || containerId in 0..255) { "Invalid container id" }
        require(hudChannel == null || hudChannel.length in 1..64) { "Invalid HUD channel" }
    }
}

@Serializable
public data class PackFingerprint(
    public val algorithm: String,
    public val value: String,
) {
    internal fun validate() {
        require(algorithm == "sha256") { "Unsupported pack fingerprint algorithm '$algorithm'" }
        require(value.length == 64 && value.all { it in '0'..'9' || it in 'a'..'f' }) {
            "Pack fingerprint must be 64 lowercase hexadecimal characters"
        }
    }
}
