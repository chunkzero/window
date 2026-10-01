package dev.oglass.window.inspector;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import net.minecraft.client.gui.Font;
import net.minecraft.util.FormattedCharSequence;

import java.util.ArrayDeque;
import java.util.Deque;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicLong;

/** Narrow thread-safe observation surface shared by the renderer hook, overlay, and tests. */
public final class InspectorState {
    public static final int MAX_FRAME_BYTES = 256 * 1024;
    private static final int MAX_OBSERVATIONS = 512;
    private static final int MAX_FRAMES = 32;
    private static final InspectorState INSTANCE = new InspectorState();

    private final AtomicLong observationSequence = new AtomicLong();
    private final Deque<RenderObservation> observations = new ArrayDeque<>();
    private final Deque<ReceivedFrame> frames = new ArrayDeque<>();
    private final Map<String, Long> latestFrameBySession = new HashMap<>();
    private volatile List<DescriptorCatalog.Descriptor> descriptors = List.of();
    private volatile boolean enhanced;
    private volatile String sessionId;
    private volatile boolean overlayVisible;
    private volatile int selectedLayer;

    private InspectorState() {}

    public static InspectorState instance() {
        return INSTANCE;
    }

    public synchronized void observeText(
            Font font, FormattedCharSequence content, int x, int y, int color, boolean shadow) {
        if (!overlayVisible && descriptors.isEmpty() && frames.isEmpty()) return;
        observations.addLast(
                RenderObservation.text(
                        observationSequence.incrementAndGet(), font, content, x, y, color, shadow));
        while (observations.size() > MAX_OBSERVATIONS) observations.removeFirst();
    }

    public synchronized boolean acceptFrame(String json) {
        if (!enhanced) return false;
        byte[] encoded = json.getBytes(java.nio.charset.StandardCharsets.UTF_8);
        if (encoded.length > MAX_FRAME_BYTES) return false;
        try {
            JsonObject object = JsonParser.parseString(json).getAsJsonObject();
            if (!object.has("schema_version") || object.get("schema_version").getAsInt() != 1) {
                return false;
            }
            if (!object.has("layers")
                    || !object.get("layers").isJsonArray()
                    || object.getAsJsonArray("layers").size() > 4096) return false;
            if (!object.has("reason")
                    || !object.has("cursor_convention")
                    || !object.has("cursor_start")
                    || !object.has("cursor_end")
                    || !object.has("net_cursor_delta")) return false;
            String reason = object.get("reason").getAsString();
            if (!reason.equals("open") && !reason.equals("reactive_update")) return false;
            long frameId = object.has("frame_id") ? object.get("frame_id").getAsLong() : -1;
            if (frameId <= 0
                    || !object.has("correlation")
                    || !object.get("correlation").isJsonObject()) return false;
            String renderSession =
                    string(object.getAsJsonObject("correlation"), "render_session_id", "");
            if (renderSession.isBlank() || renderSession.length() > 128) return false;
            JsonObject correlation = object.getAsJsonObject("correlation");
            String surfaceKind = string(correlation, "surface_kind", "");
            String semanticId = string(correlation, "semantic_id", "");
            if ((!surfaceKind.equals("window") && !surfaceKind.equals("hud"))
                    || semanticId.isBlank()
                    || semanticId.length() > 256) return false;
            if (object.get("cursor_end").getAsInt() - object.get("cursor_start").getAsInt()
                    != object.get("net_cursor_delta").getAsInt()) return false;
            java.util.Set<String> semanticLayers = new java.util.HashSet<>();
            for (var element : object.getAsJsonArray("layers")) {
                if (!element.isJsonObject()) return false;
                JsonObject layer = element.getAsJsonObject();
                String layerId = string(layer, "semantic_id", "");
                if (layerId.isBlank()
                        || layerId.length() > 256
                        || !semanticLayers.add(layerId)
                        || !layer.has("kind")
                        || !layer.has("content")
                        || layer.get("content").getAsString().length() > 32767
                        || !layer.has("font")
                        || layer.get("font").getAsString().length() > 256
                        || !layer.has("expected_bounds")
                        || !layer.get("expected_bounds").isJsonObject()) return false;
            }
            long latest = latestFrameBySession.getOrDefault(renderSession, 0L);
            if (frameId <= latest) return false;
            latestFrameBySession.put(renderSession, frameId);
            frames.addLast(new ReceivedFrame(frameId, renderSession, json, object));
            while (frames.size() > MAX_FRAMES) frames.removeFirst();
            selectedLayer = Math.min(selectedLayer, Math.max(0, layerCount() - 1));
            return true;
        } catch (RuntimeException error) {
            return false;
        }
    }

    public boolean acceptCapability(String json, String clientNonce) {
        if (json.getBytes(java.nio.charset.StandardCharsets.UTF_8).length > MAX_FRAME_BYTES)
            return false;
        try {
            JsonObject object = JsonParser.parseString(json).getAsJsonObject();
            if (object.get("schema_version").getAsInt() != 1
                    || !object.get("accepted").getAsBoolean()
                    || !clientNonce.equals(object.get("client_nonce").getAsString())
                    || object.get("max_payload_bytes").getAsInt() <= 0
                    || object.get("max_payload_bytes").getAsInt() > MAX_FRAME_BYTES
                    || !hasRenderFrameFeature(object.getAsJsonArray("features"))) {
                return false;
            }
            sessionId = object.get("session_id").getAsString();
            if (sessionId.isBlank() || sessionId.length() > 128) return false;
            enhanced = true;
            return true;
        } catch (RuntimeException error) {
            return false;
        }
    }

    private static boolean hasRenderFrameFeature(JsonArray features) {
        for (var feature : features) {
            if (feature.isJsonPrimitive() && feature.getAsString().equals("render_frame_v1"))
                return true;
        }
        return false;
    }

    public synchronized List<RenderObservation> observations() {
        return List.copyOf(observations);
    }

    public synchronized Optional<ReceivedFrame> latestFrame() {
        return frames.isEmpty() ? Optional.empty() : Optional.of(frames.getLast());
    }

    public synchronized List<ReceivedFrame> frames() {
        return List.copyOf(frames);
    }

    public List<DescriptorCatalog.Descriptor> descriptors() {
        return descriptors;
    }

    public void descriptors(List<DescriptorCatalog.Descriptor> value) {
        descriptors = List.copyOf(value);
    }

    public boolean enhanced() {
        return enhanced;
    }

    public void disconnected() {
        enhanced = false;
        sessionId = null;
        synchronized (this) {
            frames.clear();
            latestFrameBySession.clear();
        }
    }

    public boolean overlayVisible() {
        return overlayVisible;
    }

    public void toggleOverlay() {
        overlayVisible = !overlayVisible;
    }

    public synchronized void selectNextLayer() {
        int count = layerCount();
        if (count > 0) selectedLayer = (selectedLayer + 1) % count;
    }

    public synchronized Optional<JsonObject> selectedLayer() {
        if (frames.isEmpty()) return Optional.empty();
        JsonArray layers = layers(frames.getLast().json());
        if (layers.isEmpty()) return Optional.empty();
        selectedLayer = Math.min(selectedLayer, layers.size() - 1);
        return Optional.of(layers.get(selectedLayer).getAsJsonObject());
    }

    public synchronized Optional<String> firstMismatch() {
        if (frames.isEmpty()) return Optional.empty();
        JsonObject frame = frames.getLast().json();
        if (frame.has("pack_fingerprint") && frame.get("pack_fingerprint").isJsonObject()) {
            JsonObject expected = frame.getAsJsonObject("pack_fingerprint");
            boolean matched =
                    descriptors.stream()
                            .filter(DescriptorCatalog.Descriptor::valid)
                            .map(DescriptorCatalog.Descriptor::json)
                            .filter(json -> json.has("pack_fingerprint"))
                            .map(json -> json.getAsJsonObject("pack_fingerprint"))
                            .anyMatch(actual -> fingerprintEquals(expected, actual));
            if (!matched) {
                return Optional.of(
                        "server frame pack fingerprint does not match an active descriptor");
            }
        }
        String convention = string(frame, "cursor_convention", "");
        if (convention.equals("independent_net_zero_segments")
                && frame.has("net_cursor_delta")
                && frame.get("net_cursor_delta").getAsInt() != 0) {
            return Optional.of("window frame violates its net-zero cursor contract");
        }
        if (convention.equals("fixed_width_composition")
                && frame.has("cursor_start")
                && frame.has("cursor_end")
                && frame.has("net_cursor_delta")
                && frame.get("cursor_end").getAsInt() - frame.get("cursor_start").getAsInt()
                        != frame.get("net_cursor_delta").getAsInt()) {
            return Optional.of("HUD frame cursor delta does not equal its fixed width");
        }
        return Optional.empty();
    }

    private static boolean fingerprintEquals(JsonObject first, JsonObject second) {
        return string(first, "algorithm", "").equals(string(second, "algorithm", ""))
                && string(first, "value", "").equals(string(second, "value", ""));
    }

    public synchronized Optional<RenderObservation> observationFor(JsonObject layer) {
        String content = string(layer, "content", "");
        if (content.isEmpty())
            return observations.isEmpty() ? Optional.empty() : Optional.of(observations.getLast());
        return observations.stream()
                .filter(value -> value.content().contains(content))
                .reduce((first, second) -> second);
    }

    private int layerCount() {
        return frames.isEmpty() ? 0 : layers(frames.getLast().json()).size();
    }

    private static JsonArray layers(JsonObject frame) {
        return frame.has("layers") && frame.get("layers").isJsonArray()
                ? frame.getAsJsonArray("layers")
                : new JsonArray();
    }

    static String string(JsonObject object, String name, String fallback) {
        return object.has(name) && object.get(name).isJsonPrimitive()
                ? object.get(name).getAsString()
                : fallback;
    }

    public record ReceivedFrame(
            long frameId, String renderSessionId, String rawJson, JsonObject json) {}
}
