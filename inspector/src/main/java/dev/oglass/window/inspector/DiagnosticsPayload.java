package dev.oglass.window.inspector;

import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.Identifier;

/** Minecraft codecs for the shared Window diagnostics protocol's bounded JSON messages. */
public final class DiagnosticsPayload {
    public static final Identifier CAPABILITY_CHANNEL =
            Identifier.fromNamespaceAndPath("window", "diagnostics/capability");
    public static final Identifier FRAME_CHANNEL =
            Identifier.fromNamespaceAndPath("window", "diagnostics/frame");

    private DiagnosticsPayload() {}

    public record Capability(String json) implements CustomPacketPayload {
        public static final Type<Capability> TYPE = new Type<>(CAPABILITY_CHANNEL);
        public static final StreamCodec<RegistryFriendlyByteBuf, Capability> CODEC =
                StreamCodec.of(
                        (buffer, payload) ->
                                buffer.writeBytes(
                                        payload.json.getBytes(
                                                java.nio.charset.StandardCharsets.UTF_8)),
                        buffer -> new Capability(readJson(buffer)));

        public Capability {
            requireBounded(json);
        }

        @Override
        public Type<Capability> type() {
            return TYPE;
        }
    }

    public record Frame(String json) implements CustomPacketPayload {
        public static final Type<Frame> TYPE = new Type<>(FRAME_CHANNEL);
        public static final StreamCodec<RegistryFriendlyByteBuf, Frame> CODEC =
                StreamCodec.of(
                        (buffer, payload) ->
                                buffer.writeBytes(
                                        payload.json.getBytes(
                                                java.nio.charset.StandardCharsets.UTF_8)),
                        buffer -> new Frame(readJson(buffer)));

        public Frame {
            requireBounded(json);
        }

        @Override
        public Type<Frame> type() {
            return TYPE;
        }
    }

    private static void requireBounded(String json) {
        if (json.getBytes(java.nio.charset.StandardCharsets.UTF_8).length
                > InspectorState.MAX_FRAME_BYTES) {
            throw new IllegalArgumentException("diagnostics payload exceeds maximum size");
        }
    }

    private static String readJson(RegistryFriendlyByteBuf buffer) {
        int length = buffer.readableBytes();
        if (length < 0 || length > InspectorState.MAX_FRAME_BYTES) {
            throw new IllegalArgumentException("diagnostics payload exceeds maximum size");
        }
        byte[] bytes = new byte[length];
        buffer.readBytes(bytes);
        try {
            return java.nio.charset.StandardCharsets.UTF_8
                    .newDecoder()
                    .onMalformedInput(java.nio.charset.CodingErrorAction.REPORT)
                    .onUnmappableCharacter(java.nio.charset.CodingErrorAction.REPORT)
                    .decode(java.nio.ByteBuffer.wrap(bytes))
                    .toString();
        } catch (java.nio.charset.CharacterCodingException error) {
            throw new IllegalArgumentException("diagnostics payload is not valid UTF-8", error);
        }
    }
}
