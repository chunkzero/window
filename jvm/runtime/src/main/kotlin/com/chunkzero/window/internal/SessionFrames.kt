package com.chunkzero.window.internal

import com.chunkzero.window.RenderDiagnosticsObserver
import com.chunkzero.window.diagnostics.RenderBounds
import com.chunkzero.window.diagnostics.RenderCorrelation
import com.chunkzero.window.diagnostics.RenderCursorConvention
import com.chunkzero.window.diagnostics.RenderFrame
import com.chunkzero.window.diagnostics.RenderFrameReason
import com.chunkzero.window.diagnostics.RenderLayerKind
import com.chunkzero.window.diagnostics.RenderLayerTrace
import com.chunkzero.window.diagnostics.RenderStyleTrace
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import org.slf4j.Logger
import java.util.UUID
import java.util.concurrent.atomic.AtomicLong

/** Frame-level cursor framing shared by every frame of one session. */
internal data class FrameCursor(
    val convention: RenderCursorConvention,
    val start: Int,
    val end: Int,
)

/**
 * Numbers a session's composed frames and reports them to its diagnostics observer, isolating
 * observer failures from normal rendering.
 */
internal class SessionFrames(
    private val player: Player,
    private val observer: RenderDiagnosticsObserver,
    private val cursor: FrameCursor,
    private val logger: Logger,
    private val failureMessage: String,
    private val correlation: (renderSessionId: String) -> RenderCorrelation,
) {
    private val renderSessionId = UUID.randomUUID().toString()
    private val nextFrameId = AtomicLong(1)

    fun observe(
        render: ComposedRender,
        reason: RenderFrameReason,
    ) {
        val frame =
            RenderFrame(
                frameId = nextFrameId.getAndIncrement(),
                reason = reason,
                correlation = correlation(renderSessionId),
                cursorConvention = cursor.convention,
                cursorStart = cursor.start,
                cursorEnd = cursor.end,
                netCursorDelta = cursor.end - cursor.start,
                layers = render.layers,
            )
        try {
            observer.observe(player, frame)
        } catch (error: RuntimeException) {
            logger.warn(failureMessage, error)
        }
    }
}

/** A segment that draws nothing, traced as a zero-size layer at ([x], [y]). */
internal fun emptySegment(
    semanticId: String,
    kind: RenderLayerKind,
    x: Int,
    y: Int,
    font: String,
    style: RenderStyleTrace,
    cursor: Int,
): RenderedSegment =
    RenderedSegment(
        Component.empty(),
        RenderLayerTrace(
            semanticId = semanticId,
            kind = kind,
            content = "",
            font = font,
            style = style,
            expectedBounds = RenderBounds(x, y, 0, 0),
            cursorStart = cursor,
            contentCursorStart = x,
            contentCursorEnd = x,
            cursorEnd = cursor,
            advance = 0,
            visualWidth = 0,
            netCursorDelta = 0,
        ),
    )
