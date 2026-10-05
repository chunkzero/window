package com.chunkzero.window.inspector.mixin;

import com.chunkzero.window.inspector.InspectorState;

import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.util.FormattedCharSequence;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Observes the real GUI text extraction call immediately before Minecraft builds glyph quads. */
@Mixin(GuiGraphicsExtractor.class)
abstract class GuiGraphicsExtractorMixin {
    @Inject(
            method =
                    "text(Lnet/minecraft/client/gui/Font;Lnet/minecraft/util/FormattedCharSequence;IIIZ)V",
            at = @At("HEAD"))
    private void window$observeText(
            Font font,
            FormattedCharSequence content,
            int x,
            int y,
            int color,
            boolean shadow,
            CallbackInfo callback) {
        InspectorState.instance().observeText(font, content, x, y, color, shadow);
    }
}
