package dev.oglass.window.inspector;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import dev.rpp.mcvalidation.client.ValidationArtifactSink.ArtifactBounds;
import dev.rpp.mcvalidation.client.ValidationExtension;
import dev.rpp.mcvalidation.client.ValidationStepContext;
import dev.rpp.mcvalidation.core.CustomStep;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

/**
 * MC Validation extension for Window descriptor, diagnostics-frame, and semantic-layer evidence.
 */
public final class WindowValidationExtension implements ValidationExtension {
    private static final Gson JSON =
            new GsonBuilder().setPrettyPrinting().disableHtmlEscaping().create();
    private static final Set<String> TYPES =
            Set.of(
                    "window:assert_descriptor",
                    "window:wait_frame",
                    "window:assert_layer",
                    "window:capture_layer");

    private final Set<String> consumedFrames = new HashSet<>();
    private InspectorState.ReceivedFrame currentFrame;

    @Override
    public boolean supports(String stepType) {
        return TYPES.contains(stepType);
    }

    @Override
    public void execute(ValidationStepContext context) {
        JsonObject step = payload(context);
        switch (context.step().getType()) {
            case "window:assert_descriptor" -> assertDescriptor(context, step);
            case "window:wait_frame" -> waitForFrame(context, step);
            case "window:assert_layer" -> assertLayer(context, step);
            case "window:capture_layer" -> captureLayer(context, step);
            default ->
                    throw new AssertionError(
                            "unsupported Window validation step: " + context.step().getType());
        }
    }

    private void assertDescriptor(ValidationStepContext context, JsonObject step) {
        String namespace = requiredString(step, "namespace");
        String id = requiredString(step, "id");
        context.client().runOnClient(WindowInspector::reloadDescriptors);
        DescriptorCatalog.Descriptor descriptor =
                InspectorState.instance().descriptors().stream()
                        .filter(value -> value.resource().getNamespace().equals(namespace))
                        .findFirst()
                        .orElseThrow(
                                () ->
                                        new AssertionError(
                                                "active ResourceManager has no Window descriptor"
                                                        + " for "
                                                        + namespace));
        if (!descriptor.valid()) {
            throw new AssertionError("active Window descriptor is invalid: " + descriptor.error());
        }

        JsonObject evidence = new JsonObject();
        evidence.addProperty("namespace", namespace);
        evidence.addProperty("resource", descriptor.resource().toString());
        evidence.addProperty("source_pack", descriptor.packId());
        evidence.addProperty("descriptor_sha256", descriptor.sha256());
        evidence.add("descriptor", descriptor.json().deepCopy());
        context.artifacts().retainText(id, "window_descriptor", "json", JSON.toJson(evidence));
        context.artifacts()
                .pass(
                        "window.descriptor." + namespace,
                        "validated active descriptor "
                                + descriptor.resource()
                                + " from "
                                + descriptor.packId());
    }

    private void waitForFrame(ValidationStepContext context, JsonObject step) {
        String surfaceKind = requiredString(step, "surface_kind");
        String semanticId = requiredString(step, "semantic_id");
        String reason = requiredString(step, "reason");
        String id = requiredString(step, "id");
        int timeoutTicks = optionalInt(step, "timeout_ticks", 200);
        long afterFrameId = optionalLong(step, "after_frame_id", 0);
        if (!Set.of("window", "hud").contains(surfaceKind)) {
            throw new AssertionError("surface_kind must be window or hud");
        }
        if (!Set.of("open", "reactive_update").contains(reason)) {
            throw new AssertionError("reason must be open or reactive_update");
        }
        if (timeoutTicks <= 0 || afterFrameId < 0) {
            throw new AssertionError(
                    "timeout_ticks must be positive and after_frame_id must not be negative");
        }

        context.client()
                .waitFor(
                        client ->
                                matchingFrame(surfaceKind, semanticId, reason, afterFrameId)
                                        != null,
                        timeoutTicks);
        currentFrame = matchingFrame(surfaceKind, semanticId, reason, afterFrameId);
        if (currentFrame == null) throw new AssertionError("matching Window frame disappeared");
        consumedFrames.add(frameKey(currentFrame));
        InspectorState.instance()
                .firstMismatch()
                .ifPresent(
                        value -> {
                            throw new AssertionError(value);
                        });
        context.artifacts()
                .retainText(id, "window_frame", "json", JSON.toJson(currentFrame.json()));
        context.artifacts()
                .pass(
                        "window.frame." + semanticId + "." + reason,
                        "matched render session "
                                + currentFrame.renderSessionId()
                                + " frame "
                                + currentFrame.frameId());
    }

    private InspectorState.ReceivedFrame matchingFrame(
            String surfaceKind, String semanticId, String reason, long afterFrameId) {
        return InspectorState.instance().frames().stream()
                .filter(frame -> frame.frameId() > afterFrameId)
                .filter(frame -> !consumedFrames.contains(frameKey(frame)))
                .filter(frame -> reason.equals(frame.json().get("reason").getAsString()))
                .filter(
                        frame -> {
                            JsonObject correlation = frame.json().getAsJsonObject("correlation");
                            return surfaceKind.equals(correlation.get("surface_kind").getAsString())
                                    && semanticId.equals(
                                            correlation.get("semantic_id").getAsString());
                        })
                .reduce((first, second) -> second)
                .orElse(null);
    }

    private void assertLayer(ValidationStepContext context, JsonObject step) {
        String layerId = requiredString(step, "layer");
        String expectedFont = optionalString(step, "font");
        String id = requiredString(step, "id");
        int minimumGlyphs = optionalInt(step, "min_glyphs", 1);
        int tolerance = optionalInt(step, "tolerance", 2);
        int timeoutTicks = optionalInt(step, "timeout_ticks", 200);
        if (minimumGlyphs <= 0 || tolerance < 0) {
            throw new AssertionError(
                    "min_glyphs must be positive and tolerance must not be negative");
        }

        LayerObservation observed =
                waitForLayerObservation(context, layerId, expectedFont, timeoutTicks);
        if (observed.glyphs().size() < minimumGlyphs) {
            throw new AssertionError(
                    "layer "
                            + layerId
                            + " expected at least "
                            + minimumGlyphs
                            + " glyph quads in font "
                            + expectedFont
                            + ", observed "
                            + observed.glyphs().size());
        }
        int expectedHeight =
                observed.layer().getAsJsonObject("expected_bounds").get("height").getAsInt();
        double tallestGlyph =
                observed.glyphs().stream()
                        .mapToDouble(glyph -> glyph.bottom() - glyph.top())
                        .max()
                        .orElse(0);
        if (tallestGlyph > expectedHeight + tolerance) {
            throw new AssertionError(
                    "layer "
                            + layerId
                            + " glyph height exceeds authored bounds: "
                            + tallestGlyph
                            + " vs "
                            + expectedHeight
                            + " + "
                            + tolerance);
        }

        JsonObject evidence = new JsonObject();
        evidence.add("expected", observed.layer().deepCopy());
        evidence.add("minecraft_glyph_observation", JSON.toJsonTree(observed.observation()));
        evidence.add("selected_glyph_quads", JSON.toJsonTree(observed.glyphs()));
        context.artifacts()
                .retainText(id, "window_layer_observation", "json", JSON.toJson(evidence));
        context.artifacts()
                .pass(
                        "window.layer." + layerId.replace('/', '.'),
                        "matched "
                                + observed.glyphs().size()
                                + " glyph quads within authored height "
                                + expectedHeight);
    }

    private void captureLayer(ValidationStepContext context, JsonObject step) {
        String layerId = requiredString(step, "layer");
        String expectedFont = optionalString(step, "font");
        String id = requiredString(step, "id");
        int timeoutTicks = optionalInt(step, "timeout_ticks", 200);
        LayerObservation observed =
                waitForLayerObservation(context, layerId, expectedFont, timeoutTicks);
        if (observed.glyphs().isEmpty()) {
            throw new AssertionError("no matching glyph quads for " + layerId);
        }

        FramebufferMetrics framebuffer =
                context.client()
                        .computeOnClient(
                                client ->
                                        new FramebufferMetrics(
                                                client.getWindow().getWidth()
                                                        / client.getWindow().getGuiScaledWidth(),
                                                client.getWindow().getHeight()
                                                        / client.getWindow().getGuiScaledHeight(),
                                                client.getWindow().getWidth(),
                                                client.getWindow().getHeight()));
        List<ArtifactBounds> geometry = new ArrayList<>();
        for (RenderObservation.GlyphQuad glyph : observed.glyphs()) {
            int left = Math.max(0, (int) Math.floor(glyph.left() * framebuffer.scaleX()));
            int top = Math.max(0, (int) Math.floor(glyph.top() * framebuffer.scaleY()));
            int right =
                    Math.min(
                            framebuffer.width(),
                            (int) Math.ceil(glyph.right() * framebuffer.scaleX()));
            int bottom =
                    Math.min(
                            framebuffer.height(),
                            (int) Math.ceil(glyph.bottom() * framebuffer.scaleY()));
            if (right > left && bottom > top) {
                geometry.add(new ArtifactBounds(left, top, right - left, bottom - top));
            }
        }
        if (geometry.isEmpty())
            throw new AssertionError("layer " + layerId + " is entirely outside the framebuffer");
        ArtifactBounds union = union(geometry);
        context.artifacts().captureGeometryLayer(id, union, geometry);
        context.artifacts()
                .pass(
                        "window.layer-capture." + layerId.replace('/', '.'),
                        "captured "
                                + geometry.size()
                                + " glyph rectangles from the real framebuffer");
    }

    private LayerObservation waitForLayerObservation(
            ValidationStepContext context, String layerId, String expectedFont, int timeoutTicks) {
        if (timeoutTicks <= 0) throw new AssertionError("timeout_ticks must be positive");
        context.client()
                .waitFor(client -> observeLayer(layerId, expectedFont) != null, timeoutTicks);
        LayerObservation observation = observeLayer(layerId, expectedFont);
        if (observation == null)
            throw new AssertionError("matching Window layer observation disappeared");
        return observation;
    }

    private LayerObservation observeLayer(String layerId, String expectedFont) {
        JsonObject layer = requiredLayer(layerId);
        RenderObservation observation =
                InspectorState.instance().observationFor(layer).orElse(null);
        if (observation == null) return null;
        List<RenderObservation.GlyphQuad> glyphs =
                observation.glyphs().stream()
                        .filter(glyph -> fontMatches(expectedFont, glyph.font()))
                        .toList();
        return glyphs.isEmpty() ? null : new LayerObservation(layer, observation, glyphs);
    }

    private JsonObject requiredLayer(String layerId) {
        if (currentFrame == null) throw new AssertionError("no correlated Window frame selected");
        for (var element : currentFrame.json().getAsJsonArray("layers")) {
            JsonObject layer = element.getAsJsonObject();
            if (layerId.equals(layer.get("semantic_id").getAsString())) return layer;
        }
        throw new AssertionError("frame has no semantic layer " + layerId);
    }

    private static ArtifactBounds union(List<ArtifactBounds> geometry) {
        int left = geometry.stream().mapToInt(ArtifactBounds::x).min().orElseThrow();
        int top = geometry.stream().mapToInt(ArtifactBounds::y).min().orElseThrow();
        int right =
                geometry.stream().mapToInt(value -> value.x() + value.width()).max().orElseThrow();
        int bottom =
                geometry.stream().mapToInt(value -> value.y() + value.height()).max().orElseThrow();
        return new ArtifactBounds(left, top, right - left, bottom - top);
    }

    private static boolean fontMatches(String expected, String observed) {
        return expected == null
                || expected.equals(observed)
                || observed.equals("Resource[id=" + expected + "]");
    }

    private static String frameKey(InspectorState.ReceivedFrame frame) {
        return frame.renderSessionId() + ":" + frame.frameId();
    }

    private static JsonObject payload(ValidationStepContext context) {
        if (!(context.step() instanceof CustomStep custom)) {
            throw new AssertionError("Window extension requires a custom scenario step");
        }
        return JsonParser.parseString(custom.getPayload().toString()).getAsJsonObject();
    }

    private static String requiredString(JsonObject object, String name) {
        String value = optionalString(object, name);
        if (value == null || value.isBlank()) throw new AssertionError("missing required " + name);
        return value;
    }

    private static String optionalString(JsonObject object, String name) {
        return object.has(name) && object.get(name).isJsonPrimitive()
                ? object.get(name).getAsString()
                : null;
    }

    private static int optionalInt(JsonObject object, String name, int fallback) {
        return object.has(name) ? object.get(name).getAsInt() : fallback;
    }

    private static long optionalLong(JsonObject object, String name, long fallback) {
        return object.has(name) ? object.get(name).getAsLong() : fallback;
    }

    private record LayerObservation(
            JsonObject layer,
            RenderObservation observation,
            List<RenderObservation.GlyphQuad> glyphs) {}

    private record FramebufferMetrics(int scaleX, int scaleY, int width, int height) {}
}
