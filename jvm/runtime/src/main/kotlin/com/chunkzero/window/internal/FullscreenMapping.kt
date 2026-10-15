package com.chunkzero.window.internal

import com.chunkzero.window.manifest.ControlEntry
import com.chunkzero.window.manifest.FullscreenCanvasEntry
import kotlin.math.abs
import kotlin.math.floor
import kotlin.math.sign

/** A point on the full-screen canvas, in canvas pixels from its top-left. */
internal data class CanvasPoint(
    val x: Int,
    val y: Int,
)

/** A relative rotation in degrees. */
internal data class Turn(
    val yaw: Float,
    val pitch: Float,
)

/** The rotation-to-canvas mapping of [canvas], which the generated text shader computes the same way. */
internal class FullscreenMapping(
    private val canvas: FullscreenCanvasEntry,
) {
    /** The cursor's canvas point for a rotation from yaw and pitch 0, with yaw wrapped into `[-180, 180)`. */
    fun cursor(
        yaw: Float,
        pitch: Float,
    ): CanvasPoint = CanvasPoint(axis(canvas.width, wrap(yaw)), axis(canvas.height, pitch.toDouble()))

    /**
     * The relative rotation that returns a player turned more than [PULL_BACK_BAND] degrees past a canvas edge to that
     * edge, or `null` within it. Yaw past the seam behind the player returns to the nearer edge.
     */
    fun pullBack(
        yaw: Float,
        pitch: Float,
    ): Turn? {
        val turn = Turn(overshoot(wrap(yaw), canvas.edgeYaw), overshoot(pitch.toDouble(), canvas.edgePitch))
        return turn.takeIf { it.yaw != 0f || it.pitch != 0f }
    }

    /** Whether [point] lies within [control] grown by [margin] canvas pixels on every side. */
    fun hits(
        control: ControlEntry,
        point: CanvasPoint,
        margin: Int = 0,
    ): Boolean =
        point.x >= control.x - margin &&
            point.x < control.x + control.width + margin &&
            point.y >= control.y - margin &&
            point.y < control.y + control.height + margin

    private fun wrap(yaw: Float): Double = yaw - 360.0 * floor((yaw + 180.0) / 360.0)

    private fun axis(
        size: Int,
        angle: Double,
    ): Int {
        val pixel = floor(size / 2.0 + angle / canvas.degreesPerPixel + canvas.snapEpsilon)
        return pixel.coerceIn(0.0, size - 1.0).toInt()
    }

    private fun overshoot(
        angle: Double,
        edge: Double,
    ): Float = if (abs(angle) > edge + PULL_BACK_BAND) (sign(angle) * edge - angle).toFloat() else 0f

    companion object {
        /** Degrees past a canvas edge a player may turn before [pullBack] turns them back. */
        const val PULL_BACK_BAND = 1.0
    }
}
