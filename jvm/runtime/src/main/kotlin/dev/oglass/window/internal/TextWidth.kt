package dev.oglass.window.internal

/** Cursor advance and visible ink width (pixels) of measured text. */
internal data class TextWidth(
    val advance: Int,
    val visual: Int,
) {
    /** Width of this text followed by [next], whose ink starts at this advance. */
    fun append(next: TextWidth): TextWidth = TextWidth(advance + next.advance, maxOf(visual, advance + next.visual))

    internal companion object {
        val ZERO = TextWidth(0, 0)
    }
}
