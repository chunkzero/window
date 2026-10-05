package com.chunkzero.window.inspector;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import net.minecraft.resources.Identifier;
import net.minecraft.server.packs.resources.Resource;
import net.minecraft.server.packs.resources.ResourceManager;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;

/** Discovers Window debug descriptors from the resource packs Minecraft actually activated. */
public final class DescriptorCatalog {
    public static final int SUPPORTED_SCHEMA_VERSION = 1;
    public static final int MAX_DESCRIPTOR_BYTES = 2 * 1024 * 1024;

    private DescriptorCatalog() {}

    public static List<Descriptor> load(ResourceManager resources) {
        Map<Identifier, Resource> candidates =
                resources.listResources(
                        "window", identifier -> identifier.getPath().equals("window/debug.json"));
        List<Descriptor> descriptors = new ArrayList<>();
        candidates.entrySet().stream()
                .sorted(Map.Entry.comparingByKey())
                .forEach(
                        entry -> {
                            Resource resource = entry.getValue();
                            try (InputStream input = resource.open()) {
                                byte[] bytes = readBounded(input, MAX_DESCRIPTOR_BYTES);
                                JsonObject json =
                                        JsonParser.parseString(
                                                        new String(
                                                                bytes,
                                                                java.nio.charset.StandardCharsets
                                                                        .UTF_8))
                                                .getAsJsonObject();
                                int schema = requiredSchema(json);
                                String validationError = validateResources(resources, json);
                                descriptors.add(
                                        new Descriptor(
                                                entry.getKey(),
                                                resource.sourcePackId(),
                                                schema,
                                                sha256(bytes),
                                                bytes,
                                                json,
                                                validationError));
                            } catch (IOException | RuntimeException error) {
                                descriptors.add(
                                        Descriptor.invalid(
                                                entry.getKey(),
                                                resource.sourcePackId(),
                                                error.toString()));
                            }
                        });
        return List.copyOf(descriptors);
    }

    static int requiredSchema(JsonObject json) {
        if (!json.has("schema_version") || !json.get("schema_version").isJsonPrimitive()) {
            throw new IllegalArgumentException("debug descriptor has no integer schema_version");
        }
        int schema = json.get("schema_version").getAsInt();
        if (schema <= 0)
            throw new IllegalArgumentException("debug descriptor schema_version must be positive");
        return schema;
    }

    private static String validateResources(ResourceManager manager, JsonObject descriptor) {
        if (!descriptor.has("resources") || !descriptor.get("resources").isJsonObject()) {
            return "debug descriptor has no resources object";
        }
        for (Map.Entry<String, com.google.gson.JsonElement> entry :
                descriptor.getAsJsonObject("resources").entrySet()) {
            if (!entry.getValue().isJsonObject())
                return "resource entry is not an object: " + entry.getKey();
            JsonObject expected = entry.getValue().getAsJsonObject();
            if (!expected.has("fingerprint")) continue;
            Identifier identifier = identifierFor(expected);
            if (identifier == null) continue;
            Resource actual = manager.getResource(identifier).orElse(null);
            if (actual == null) return "active resource is missing: " + identifier;
            try (InputStream input = actual.open()) {
                byte[] bytes = readBounded(input, MAX_DESCRIPTOR_BYTES * 8);
                if (expected.has("byte_length")
                        && expected.get("byte_length").getAsLong() != bytes.length) {
                    return "active resource length mismatch: " + identifier;
                }
                JsonObject fingerprint = expected.getAsJsonObject("fingerprint");
                if (!"sha256".equals(fingerprint.get("algorithm").getAsString())
                        || !sha256(bytes).equals(fingerprint.get("value").getAsString())) {
                    return "active resource fingerprint mismatch: " + identifier;
                }
            } catch (IOException | RuntimeException error) {
                return "could not validate active resource "
                        + identifier
                        + ": "
                        + error.getMessage();
            }
        }
        return null;
    }

    private static Identifier identifierFor(JsonObject resource) {
        if (resource.has("path")) {
            String path = resource.get("path").getAsString();
            if (path.startsWith("assets/")) {
                int namespaceEnd = path.indexOf('/', "assets/".length());
                if (namespaceEnd > 0 && namespaceEnd + 1 < path.length()) {
                    return Identifier.tryBuild(
                            path.substring("assets/".length(), namespaceEnd),
                            path.substring(namespaceEnd + 1));
                }
            }
        }
        if (resource.has("resource_id")) {
            String id = resource.get("resource_id").getAsString();
            // Font definition resource ids name the logical font and omit the file suffix.
            if (id.contains(":font/") && !id.endsWith(".json")) id += ".json";
            return Identifier.tryParse(id);
        }
        return null;
    }

    static byte[] readBounded(InputStream input, int maximum) throws IOException {
        ByteArrayOutputStream output = new ByteArrayOutputStream(Math.min(maximum, 8192));
        byte[] buffer = new byte[8192];
        int total = 0;
        for (int count; (count = input.read(buffer)) >= 0; ) {
            total += count;
            if (total > maximum) {
                throw new IOException("debug descriptor exceeds " + maximum + " bytes");
            }
            output.write(buffer, 0, count);
        }
        return output.toByteArray();
    }

    private static String sha256(byte[] bytes) {
        try {
            return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
        } catch (NoSuchAlgorithmException impossible) {
            throw new AssertionError(impossible);
        }
    }

    /** A descriptor together with the active pack that supplied its winning resource. */
    public record Descriptor(
            Identifier resource,
            String packId,
            int schemaVersion,
            String sha256,
            byte[] bytes,
            JsonObject json,
            String error) {
        public Descriptor {
            bytes = bytes.clone();
        }

        static Descriptor invalid(Identifier resource, String packId, String error) {
            return new Descriptor(resource, packId, 0, "", new byte[0], null, error);
        }

        public boolean valid() {
            return error == null && schemaVersion == SUPPORTED_SCHEMA_VERSION;
        }

        @Override
        public byte[] bytes() {
            return bytes.clone();
        }
    }
}
