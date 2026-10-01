package dev.oglass.window.inspector;

import com.mojang.blaze3d.platform.InputConstants;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keymapping.v1.KeyMappingHelper;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayNetworking;
import net.fabricmc.fabric.api.client.rendering.v1.hud.HudElementRegistry;
import net.fabricmc.fabric.api.client.rendering.v1.hud.VanillaHudElements;
import net.fabricmc.fabric.api.networking.v1.PayloadTypeRegistry;
import net.fabricmc.fabric.api.resource.v1.ResourceLoader;
import net.fabricmc.fabric.api.resource.v1.reloader.SimpleReloadListener;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.resources.Identifier;
import net.minecraft.server.packs.PackType;
import net.minecraft.server.packs.resources.PreparableReloadListener;

import org.lwjgl.glfw.GLFW;

import java.util.UUID;

/** Fabric entry point for the distributable Window Render Inspector. */
public final class WindowInspector implements ClientModInitializer {
    private static final Identifier OVERLAY =
            Identifier.fromNamespaceAndPath("window-inspector", "overlay");
    private static final KeyMapping.Category CATEGORY =
            KeyMapping.Category.register(
                    Identifier.fromNamespaceAndPath("window-inspector", "controls"));
    private static final KeyMapping TOGGLE =
            new KeyMapping(
                    "key.window-inspector.toggle",
                    InputConstants.Type.KEYSYM,
                    GLFW.GLFW_KEY_F7,
                    CATEGORY);
    private static final KeyMapping NEXT_LAYER =
            new KeyMapping(
                    "key.window-inspector.next_layer",
                    InputConstants.Type.KEYSYM,
                    GLFW.GLFW_KEY_PAGE_DOWN,
                    CATEGORY);
    private static final KeyMapping EXPORT =
            new KeyMapping(
                    "key.window-inspector.export",
                    InputConstants.Type.KEYSYM,
                    GLFW.GLFW_KEY_F8,
                    CATEGORY);
    private static String clientNonce;
    private static boolean helloSent;
    private static int handshakeRetry;

    @Override
    public void onInitializeClient() {
        PayloadTypeRegistry.clientboundPlay()
                .register(DiagnosticsPayload.Capability.TYPE, DiagnosticsPayload.Capability.CODEC);
        PayloadTypeRegistry.serverboundPlay()
                .register(DiagnosticsPayload.Capability.TYPE, DiagnosticsPayload.Capability.CODEC);
        PayloadTypeRegistry.clientboundPlay()
                .register(DiagnosticsPayload.Frame.TYPE, DiagnosticsPayload.Frame.CODEC);
        ClientPlayNetworking.registerGlobalReceiver(
                DiagnosticsPayload.Capability.TYPE,
                (payload, context) ->
                        context.client()
                                .execute(
                                        () ->
                                                InspectorState.instance()
                                                        .acceptCapability(
                                                                payload.json(), clientNonce)));
        ClientPlayNetworking.registerGlobalReceiver(
                DiagnosticsPayload.Frame.TYPE,
                (payload, context) ->
                        context.client()
                                .execute(
                                        () ->
                                                InspectorState.instance()
                                                        .acceptFrame(payload.json())));
        ClientPlayConnectionEvents.JOIN.register(
                (handler, sender, client) -> {
                    reloadDescriptors(client);
                    clientNonce = UUID.randomUUID().toString();
                    helloSent = false;
                    handshakeRetry = 0;
                    attemptHandshake();
                });
        ClientPlayConnectionEvents.DISCONNECT.register(
                (handler, client) -> {
                    clientNonce = null;
                    helloSent = false;
                    InspectorState.instance().disconnected();
                });
        ResourceLoader.get(PackType.CLIENT_RESOURCES)
                .registerReloadListener(
                        Identifier.fromNamespaceAndPath("window-inspector", "descriptors"),
                        new SimpleReloadListener<java.util.List<DescriptorCatalog.Descriptor>>() {
                            @Override
                            protected java.util.List<DescriptorCatalog.Descriptor> prepare(
                                    PreparableReloadListener.SharedState sharedState) {
                                return DescriptorCatalog.load(sharedState.resourceManager());
                            }

                            @Override
                            protected void apply(
                                    java.util.List<DescriptorCatalog.Descriptor> descriptors,
                                    PreparableReloadListener.SharedState sharedState) {
                                InspectorState.instance().descriptors(descriptors);
                            }
                        });

        KeyMappingHelper.registerKeyMapping(TOGGLE);
        KeyMappingHelper.registerKeyMapping(NEXT_LAYER);
        KeyMappingHelper.registerKeyMapping(EXPORT);
        ClientTickEvents.END_CLIENT_TICK.register(WindowInspector::tick);
        HudElementRegistry.attachElementAfter(
                VanillaHudElements.SUBTITLES,
                OVERLAY,
                (graphics, tick) -> new InspectorOverlay().render(graphics));
    }

    /** Re-reads descriptors from the active ResourceManager; exposed for the validation harness. */
    public static void reloadDescriptors(Minecraft client) {
        InspectorState.instance().descriptors(DescriptorCatalog.load(client.getResourceManager()));
    }

    static String capabilityHello(String nonce) {
        if (nonce.isBlank() || nonce.length() > 128 || nonce.indexOf('"') >= 0) {
            throw new IllegalArgumentException("client nonce must be 1..128 JSON-safe characters");
        }
        return "{\"schema_version\":1,\"minimum_schema_version\":1,"
                + "\"maximum_schema_version\":1,\"client_nonce\":\""
                + nonce
                + "\",\"max_payload_bytes\":"
                + InspectorState.MAX_FRAME_BYTES
                + ",\"features\":[\"render_frame_v1\"]}";
    }

    private static void tick(Minecraft client) {
        while (TOGGLE.consumeClick()) InspectorState.instance().toggleOverlay();
        while (NEXT_LAYER.consumeClick()) InspectorState.instance().selectNextLayer();
        while (EXPORT.consumeClick()) ReportExporter.export(client);
        if (!helloSent && clientNonce != null && ++handshakeRetry >= 20) {
            handshakeRetry = 0;
            attemptHandshake();
        }
    }

    private static void attemptHandshake() {
        if (clientNonce == null
                || !ClientPlayNetworking.canSend(DiagnosticsPayload.Capability.TYPE)) {
            return;
        }
        ClientPlayNetworking.send(new DiagnosticsPayload.Capability(capabilityHello(clientNonce)));
        helloSent = true;
    }
}
