package dev.oglass.window.inspector;

import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.mojang.blaze3d.platform.NativeImage;
import com.mojang.blaze3d.systems.DeviceInfo;
import com.mojang.blaze3d.systems.RenderSystem;

import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.client.Minecraft;
import net.minecraft.client.Screenshot;
import net.minecraft.network.chat.Component;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;

/** Creates a local report only after an explicit client key press. It never uploads data. */
final class ReportExporter {
    private static final DateTimeFormatter NAME =
            DateTimeFormatter.ofPattern("uuuuMMdd-HHmmss").withZone(ZoneOffset.UTC);

    private ReportExporter() {}

    static void export(Minecraft client) {
        Path directory =
                client.gameDirectory
                        .toPath()
                        .resolve("window-reports")
                        .resolve(NAME.format(Instant.now()));
        try {
            Files.createDirectories(directory);
            InspectorState state = InspectorState.instance();
            JsonObject report = environment(client);
            report.addProperty("report_schema_version", 1);
            report.addProperty("mode", state.enhanced() ? "enhanced" : "passive");
            state.firstMismatch()
                    .ifPresent(value -> report.addProperty("first_actionable_mismatch", value));
            JsonArray descriptors = new JsonArray();
            for (DescriptorCatalog.Descriptor descriptor : state.descriptors()) {
                JsonObject item = new JsonObject();
                item.addProperty("resource", descriptor.resource().toString());
                item.addProperty("pack", descriptor.packId());
                item.addProperty("schema_version", descriptor.schemaVersion());
                item.addProperty("sha256", descriptor.sha256());
                if (descriptor.json() != null && descriptor.json().has("pack_fingerprint")) {
                    item.add(
                            "pack_fingerprint",
                            descriptor.json().get("pack_fingerprint").deepCopy());
                }
                if (descriptor.error() != null) item.addProperty("error", descriptor.error());
                descriptors.add(item);
                if (descriptor.valid()) {
                    Files.write(
                            directory.resolve(
                                    safeName(descriptor.resource().getNamespace()) + "-debug.json"),
                            descriptor.bytes());
                }
            }
            report.add("descriptors", descriptors);
            JsonArray frames = new JsonArray();
            for (InspectorState.ReceivedFrame frame : state.frames()) frames.add(frame.json());
            report.add("render_frames", frames);
            JsonArray observations =
                    new GsonBuilder().create().toJsonTree(state.observations()).getAsJsonArray();
            report.add("minecraft_text_observations", observations);
            Files.writeString(
                    directory.resolve("report.json"),
                    new GsonBuilder().setPrettyPrinting().create().toJson(report));
            Files.writeString(
                    directory.resolve("README.txt"),
                    "Window Inspector report. Generated only by the local user's export key.\n"
                        + "It may contain server-provided rendered text and environment details."
                        + " Review before sharing.\n");
            Screenshot.takeScreenshot(
                    client.gameRenderer.mainRenderTarget(),
                    image -> {
                        try (image) {
                            image.writeToFile(directory.resolve("screenshot.png"));
                            writeIsolatedLayers(
                                    image,
                                    state,
                                    directory.resolve("layers"),
                                    client.getWindow().getGuiScale());
                        } catch (IOException ignored) {
                            // report.json remains useful if framebuffer readback fails.
                        }
                    });
            notify(client, "Window report exported to " + directory);
        } catch (IOException error) {
            notify(client, "Window report export failed: " + error.getMessage());
        }
    }

    private static void notify(Minecraft client, String message) {
        if (client.player != null) client.player.sendSystemMessage(Component.literal(message));
    }

    private static JsonObject environment(Minecraft client) {
        JsonObject value = new JsonObject();
        value.addProperty("created_at", Instant.now().toString());
        value.addProperty("minecraft", client.getLaunchedVersion());
        value.addProperty(
                "inspector",
                FabricLoader.getInstance()
                        .getModContainer("window-inspector")
                        .map(container -> container.getMetadata().getVersion().getFriendlyString())
                        .orElse("unknown"));
        value.addProperty("gui_scale", client.getWindow().getGuiScale());
        value.addProperty("language", client.options.languageCode);
        value.addProperty("unicode", client.options.forceUnicodeFont().get());
        value.addProperty("window_width", client.getWindow().getWidth());
        value.addProperty("window_height", client.getWindow().getHeight());
        value.addProperty("framebuffer_width", client.gameRenderer.mainRenderTarget().width);
        value.addProperty("framebuffer_height", client.gameRenderer.mainRenderTarget().height);
        DeviceInfo device = RenderSystem.getDevice().getDeviceInfo();
        value.addProperty("gpu_renderer", device.name());
        value.addProperty("gpu_backend", device.backendName());
        JsonArray packs = new JsonArray();
        client.getResourceManager().listPacks().forEach(pack -> packs.add(pack.packId()));
        value.add("active_pack_order", packs);
        return value;
    }

    private static void writeIsolatedLayers(
            NativeImage framebuffer, InspectorState state, Path directory, int scale)
            throws IOException {
        var frame = state.latestFrame();
        if (frame.isEmpty() || !frame.get().json().has("layers")) return;
        Files.createDirectories(directory);
        for (var element : frame.get().json().getAsJsonArray("layers")) {
            if (!element.isJsonObject()) continue;
            JsonObject layer = element.getAsJsonObject();
            var observation = state.observationFor(layer);
            if (observation.isEmpty() || observation.get().glyphs().isEmpty()) continue;
            RenderObservation actual = observation.get();
            String content = InspectorState.string(layer, "content", "");
            if (!content.equals(actual.content())) continue;
            int left = Math.max(0, actual.x() * scale);
            int top = Math.max(0, actual.y() * scale);
            int right = Math.min(framebuffer.getWidth(), (actual.x() + actual.width()) * scale);
            int bottom = Math.min(framebuffer.getHeight(), (actual.y() + actual.height()) * scale);
            if (right <= left || bottom <= top) continue;
            try (NativeImage isolated = new NativeImage(right - left, bottom - top, true)) {
                isolated.fillRect(0, 0, isolated.getWidth(), isolated.getHeight(), 0);
                for (RenderObservation.GlyphQuad glyph : actual.glyphs()) {
                    int glyphLeft = Math.max(left, (int) Math.floor(glyph.left() * scale));
                    int glyphTop = Math.max(top, (int) Math.floor(glyph.top() * scale));
                    int glyphRight = Math.min(right, (int) Math.ceil(glyph.right() * scale));
                    int glyphBottom = Math.min(bottom, (int) Math.ceil(glyph.bottom() * scale));
                    for (int y = glyphTop; y < glyphBottom; y++) {
                        for (int x = glyphLeft; x < glyphRight; x++) {
                            isolated.setPixel(x - left, y - top, framebuffer.getPixel(x, y));
                        }
                    }
                }
                String id =
                        InspectorState.string(
                                layer, "semantic_id", InspectorState.string(layer, "id", "layer"));
                isolated.writeToFile(directory.resolve(safeName(id) + ".png"));
            }
        }
    }

    private static String safeName(String value) {
        return value.replaceAll("[^a-zA-Z0-9._-]", "_");
    }
}
