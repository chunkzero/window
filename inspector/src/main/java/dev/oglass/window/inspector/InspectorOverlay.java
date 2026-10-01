package dev.oglass.window.inspector;

import com.google.gson.JsonObject;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;

import java.util.ArrayList;
import java.util.List;

final class InspectorOverlay {
    private static final int BACKGROUND = 0xd0101010;
    private static final int OK = 0xff55ff55;
    private static final int WARN = 0xffffaa00;
    private static final int TEXT = 0xffeeeeee;

    void render(GuiGraphicsExtractor graphics) {
        InspectorState state = InspectorState.instance();
        if (!state.overlayVisible()) return;
        Minecraft client = Minecraft.getInstance();
        List<String> lines = new ArrayList<>();
        lines.add("Window Inspector  " + (state.enhanced() ? "enhanced" : "passive"));
        long valid =
                state.descriptors().stream().filter(DescriptorCatalog.Descriptor::valid).count();
        lines.add("active descriptors: " + valid + "/" + state.descriptors().size());
        if (state.descriptors().isEmpty())
            lines.add("! no assets/<namespace>/window/debug.json in active packs");
        state.descriptors().stream()
                .filter(value -> !value.valid())
                .findFirst()
                .ifPresent(value -> lines.add("! descriptor: " + value.error()));
        state.firstMismatch().ifPresent(value -> lines.add("! " + value));

        state.selectedLayer()
                .ifPresentOrElse(
                        layer -> appendLayer(lines, state, layer),
                        () ->
                                lines.add(
                                        state.enhanced()
                                                ? "no frame layers"
                                                : "server trace unavailable"));
        lines.add("F7 toggle  PageDown layer  F8 export report");

        int width = lines.stream().mapToInt(client.font::width).max().orElse(220) + 12;
        int height = lines.size() * 10 + 10;
        graphics.fill(4, 4, 4 + width, 4 + height, BACKGROUND);
        int y = 10;
        for (String line : lines) {
            int color = line.startsWith("!") ? WARN : TEXT;
            graphics.text(client.font, line, 10, y, color, true);
            y += 10;
        }
    }

    private static void appendLayer(List<String> lines, InspectorState state, JsonObject layer) {
        String semantic =
                InspectorState.string(
                        layer, "semantic_id", InspectorState.string(layer, "id", "unnamed"));
        lines.add("layer: " + semantic);
        String font = InspectorState.string(layer, "font", "unspecified");
        String content = InspectorState.string(layer, "content", "");
        lines.add("expected font=" + font + " content=" + abbreviate(content));
        if (layer.has("expected_bounds")) {
            lines.add("expected bounds=" + layer.get("expected_bounds"));
        }
        String ascent = layer.has("ascent") ? layer.get("ascent").getAsString() : "provider";
        String advance = layer.has("advance") ? layer.get("advance").getAsString() : "?";
        String cursor =
                layer.has("cursor_start") && layer.has("cursor_end")
                        ? layer.get("cursor_start").getAsString()
                                + "->"
                                + layer.get("cursor_end").getAsString()
                        : "?";
        String delta =
                layer.has("net_cursor_delta") ? layer.get("net_cursor_delta").getAsString() : "?";
        lines.add(
                "expected ascent/advance/cursor/net="
                        + ascent
                        + "/"
                        + advance
                        + "/"
                        + cursor
                        + "/"
                        + delta);
        state.observationFor(layer)
                .ifPresentOrElse(
                        actual -> {
                            lines.add(
                                    "actual bounds="
                                            + actual.x()
                                            + ","
                                            + actual.y()
                                            + " "
                                            + actual.width()
                                            + "x"
                                            + actual.height());
                            lines.add(
                                    "glyph advance/cursor="
                                            + actual.width()
                                            + " / "
                                            + actual.cursorStart()
                                            + "->"
                                            + actual.cursorEnd());
                            String actualFonts =
                                    actual.glyphs().stream()
                                            .map(RenderObservation.GlyphQuad::font)
                                            .distinct()
                                            .limit(4)
                                            .collect(java.util.stream.Collectors.joining(","));
                            lines.add("actual glyph fonts=" + actualFonts);
                            String mismatch = firstMismatch(layer, actual);
                            lines.add(
                                    (mismatch == null ? "ok: " : "! ")
                                            + (mismatch == null ? "geometry observed" : mismatch));
                        },
                        () -> lines.add("! no matching Minecraft text render call observed"));
    }

    private static String firstMismatch(JsonObject layer, RenderObservation actual) {
        if (layer.has("font")) {
            String expectedFont = layer.get("font").getAsString();
            boolean matchedFont =
                    actual.glyphs().stream().anyMatch(glyph -> expectedFont.equals(glyph.font()));
            if (!matchedFont) return "expected font was absent from actual Minecraft glyph quads";
        }
        String expectedContent = InspectorState.string(layer, "content", "");
        if (!expectedContent.equals(actual.content())) {
            // The observed call contains several semantic title/HUD segments. Per-glyph geometry is
            // still exported, but aggregate call bounds cannot be compared to one layer's bounds.
            return null;
        }
        JsonObject bounds = null;
        if (layer.has("expected_bounds") && layer.get("expected_bounds").isJsonObject()) {
            bounds = layer.getAsJsonObject("expected_bounds");
        } else if (layer.has("bounds") && layer.get("bounds").isJsonObject()) {
            bounds = layer.getAsJsonObject("bounds");
        }
        if (bounds != null) {
            if (bounds.has("x") && bounds.get("x").getAsInt() != actual.x()) return "x mismatch";
            if (bounds.has("y") && bounds.get("y").getAsInt() != actual.y()) return "y mismatch";
            if (bounds.has("width") && bounds.get("width").getAsInt() != actual.width())
                return "width mismatch";
        }
        if (layer.has("advance") && layer.get("advance").getAsInt() != actual.width())
            return "advance mismatch";
        return null;
    }

    private static String abbreviate(String value) {
        return value.length() <= 42 ? value : value.substring(0, 39) + "...";
    }
}
