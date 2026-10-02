package dev.oglass.window.internal

private const val SECTION_SIGN = '§'
private const val REPLACEMENT_CHARACTER = 0xFFFD

/**
 * Visits the code points the client renders for one literal text run, with each one's bold state.
 *
 * Mirrors the client's `StringDecomposer.iterateFormatted`: `§` and the following char are never
 * rendered, and a trailing lone `§` is dropped. Codes are case-insensitive: `l` turns bold on, color
 * codes (`0`-`9`, `a`-`f`) turn it off, `r` restores [baseBold], and other codes keep it. Unpaired
 * surrogates render as U+FFFD.
 */
internal fun forEachRenderedCodePoint(
    text: String,
    baseBold: Boolean,
    action: (codePoint: Int, bold: Boolean) -> Unit,
) {
    var bold = baseBold
    var i = 0
    while (i < text.length) {
        val char = text[i]
        when {
            char == SECTION_SIGN -> {
                if (i + 1 >= text.length) return
                bold = legacyBold(text[i + 1], bold, baseBold)
                i += 2
            }

            char.isHighSurrogate() && i + 1 < text.length && text[i + 1].isLowSurrogate() -> {
                action(Character.toCodePoint(char, text[i + 1]), bold)
                i += 2
            }

            else -> {
                action(if (char.isSurrogate()) REPLACEMENT_CHARACTER else char.code, bold)
                i++
            }
        }
    }
}

private fun legacyBold(
    code: Char,
    bold: Boolean,
    baseBold: Boolean,
): Boolean =
    when (code.lowercaseChar()) {
        in '0'..'9', in 'a'..'f' -> false
        'l' -> true
        'r' -> baseBold
        else -> bold
    }
