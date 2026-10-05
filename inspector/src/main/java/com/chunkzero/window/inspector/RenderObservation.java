package com.chunkzero.window.inspector;

import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.font.TextRenderable;
import net.minecraft.network.chat.Style;
import net.minecraft.util.FormattedCharSequence;

import java.util.ArrayList;
import java.util.List;

/** A text call observed before Minecraft converts it into GUI render-state glyph quads. */
public record RenderObservation(
        long sequence,
        String content,
        String font,
        int x,
        int y,
        int width,
        int height,
        int cursorStart,
        int cursorEnd,
        boolean shadow,
        List<GlyphQuad> glyphs) {
    static RenderObservation text(
            long sequence,
            Font renderer,
            FormattedCharSequence content,
            int x,
            int y,
            int color,
            boolean shadow) {
        StringBuilder plain = new StringBuilder();
        Style[] firstStyle = new Style[] {Style.EMPTY};
        content.accept(
                (index, style, codepoint) -> {
                    if (plain.isEmpty()) firstStyle[0] = style;
                    plain.appendCodePoint(codepoint);
                    return true;
                });
        String font =
                firstStyle[0].getFont() == null
                        ? "minecraft:default"
                        : firstStyle[0].getFont().toString();
        Font.PreparedText prepared = renderer.prepareText(content, x, y, color, shadow, false, 0);
        List<GlyphQuad> glyphs = new ArrayList<>();
        prepared.visit(
                new Font.GlyphVisitor() {
                    @Override
                    public void acceptGlyph(TextRenderable.Styled glyph) {
                        Style style = glyph.style();
                        glyphs.add(
                                new GlyphQuad(
                                        glyph.left(),
                                        glyph.top(),
                                        glyph.right(),
                                        glyph.bottom(),
                                        glyph.activeLeft(),
                                        glyph.activeTop(),
                                        glyph.activeRight(),
                                        glyph.activeBottom(),
                                        String.valueOf(glyph.textureView()),
                                        style.getFont() == null
                                                ? "minecraft:default"
                                                : style.getFont().toString(),
                                        style.getColor() == null
                                                ? null
                                                : style.getColor().toString(),
                                        style.isBold(),
                                        style.isItalic(),
                                        style.isUnderlined(),
                                        style.isStrikethrough(),
                                        style.isObfuscated()));
                    }
                });
        var bounds = prepared.bounds();
        int left = bounds == null ? x : bounds.left();
        int top = bounds == null ? y : bounds.top();
        int width = bounds == null ? 0 : bounds.width();
        int height = bounds == null ? 0 : bounds.height();
        return new RenderObservation(
                sequence,
                plain.toString(),
                font,
                left,
                top,
                width,
                height,
                x,
                x + renderer.width(content),
                shadow,
                List.copyOf(glyphs));
    }

    /** Actual glyph quad and active ink geometry emitted by Minecraft's font pipeline. */
    public record GlyphQuad(
            float left,
            float top,
            float right,
            float bottom,
            float activeLeft,
            float activeTop,
            float activeRight,
            float activeBottom,
            String texture,
            String font,
            String color,
            boolean bold,
            boolean italic,
            boolean underlined,
            boolean strikethrough,
            boolean obfuscated) {}
}
