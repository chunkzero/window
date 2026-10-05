package com.chunkzero.window.inspector;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import net.minecraft.resources.Identifier;

import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.Test;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.util.List;

final class InspectorStateTest {
    @AfterEach
    void reset() {
        InspectorState.instance().disconnected();
        InspectorState.instance().descriptors(List.of());
    }

    @Test
    void acceptsVersionedMonotonicFramesAndRejectsReplay() {
        InspectorState state = InspectorState.instance();
        assertTrue(
                state.acceptCapability(
                        "{\"schema_version\":1,\"accepted\":true,\"client_nonce\":\"nonce\","
                                + "\"session_id\":\"session\",\"max_payload_bytes\":262144,"
                                + "\"features\":[\"render_frame_v1\"]}",
                        "nonce"));
        String frame = frame(1, 4, "render-a", null);
        assertTrue(state.acceptFrame(frame));
        assertFalse(state.acceptFrame(frame));
        assertFalse(state.acceptFrame(frame(2, 5, "render-a", null)));
        assertTrue(state.acceptFrame(frame(1, 1, "render-b", null)));
        assertEquals(1, state.latestFrame().orElseThrow().frameId());
    }

    @Test
    void descriptorReadsAreBounded() throws IOException {
        byte[] allowed = new byte[32];
        assertEquals(
                32,
                DescriptorCatalog.readBounded(new ByteArrayInputStream(allowed), allowed.length)
                        .length);
        byte[] oversized = new byte[33];
        org.junit.jupiter.api.Assertions.assertThrows(
                IOException.class,
                () ->
                        DescriptorCatalog.readBounded(
                                new ByteArrayInputStream(oversized), allowed.length));
    }

    @Test
    void capabilityHelloMatchesSharedProtocolWireContract() {
        JsonObject hello =
                JsonParser.parseString(WindowInspector.capabilityHello("golden-nonce"))
                        .getAsJsonObject();
        assertEquals(1, hello.get("schema_version").getAsInt());
        assertEquals(1, hello.get("minimum_schema_version").getAsInt());
        assertEquals(1, hello.get("maximum_schema_version").getAsInt());
        assertEquals("golden-nonce", hello.get("client_nonce").getAsString());
        assertEquals(262144, hello.get("max_payload_bytes").getAsInt());
        assertEquals("render_frame_v1", hello.getAsJsonArray("features").get(0).getAsString());
        assertEquals(
                "window:diagnostics/capability", DiagnosticsPayload.CAPABILITY_CHANNEL.toString());
        assertEquals("window:diagnostics/frame", DiagnosticsPayload.FRAME_CHANNEL.toString());
    }

    @Test
    void rejectsAckWithWrongNonceAndSurfacesPackFingerprintMismatch() {
        InspectorState state = InspectorState.instance();
        String ack =
                "{\"schema_version\":1,\"accepted\":true,\"client_nonce\":\"right\","
                        + "\"session_id\":\"session\",\"max_payload_bytes\":262144,"
                        + "\"features\":[\"render_frame_v1\"]}";
        assertFalse(state.acceptCapability(ack, "wrong"));
        assertTrue(state.acceptCapability(ack, "right"));

        JsonObject descriptor =
                JsonParser.parseString(
                                "{\"schema_version\":1,\"pack_fingerprint\":"
                                        + "{\"algorithm\":\"sha256\",\"value\":\"active\"}}")
                        .getAsJsonObject();
        state.descriptors(
                List.of(
                        new DescriptorCatalog.Descriptor(
                                Identifier.parse("window:window/debug.json"),
                                "test-pack",
                                1,
                                "descriptor-hash",
                                new byte[0],
                                descriptor,
                                null)));
        assertTrue(
                state.acceptFrame(
                        frame(1, 1, "render", "{\"algorithm\":\"sha256\",\"value\":\"server\"}")));
        assertTrue(state.firstMismatch().orElseThrow().contains("pack fingerprint"));
    }

    private static String frame(
            int schemaVersion, long frameId, String renderSession, String fingerprint) {
        return "{\"schema_version\":"
                + schemaVersion
                + ",\"frame_id\":"
                + frameId
                + ",\"reason\":\"open\","
                + "\"cursor_convention\":\"independent_net_zero_segments\","
                + "\"cursor_start\":0,\"cursor_end\":0,\"net_cursor_delta\":0,"
                + "\"correlation\":{\"surface_kind\":\"window\","
                + "\"semantic_id\":\"probe\",\"render_session_id\":\""
                + renderSession
                + "\"}"
                + (fingerprint == null ? "" : ",\"pack_fingerprint\":" + fingerprint)
                + ",\"layers\":[]}";
    }
}
