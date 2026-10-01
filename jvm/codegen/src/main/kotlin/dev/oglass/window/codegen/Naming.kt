package dev.oglass.window.codegen

/**
 * Name mangling for generated members and classes.
 *
 * Manifest names match `^[a-z][a-z0-9_]*$` (per `docs/MANIFEST.md`), so the only token separator is
 * `_`. We split on it and recombine into Kotlin identifiers.
 */
internal object Naming {
    /** Members of [dev.oglass.window.WindowView] that generated members must not shadow. */
    val reservedMembers: Set<String> =
        setOf(
            "player",
            "state",
            "bind",
            "onOpen",
            "onClose",
            "close",
            "refresh",
            "windowName",
            "hudName",
            "onShow",
            "onHide",
            "hide",
        )

    /** `shop_menu` → `ShopMenuView`. */
    fun className(window: String): String = pascal(window) + "View"

    /** `status_bar` → `StatusBarHud`. */
    fun hudClassName(hud: String): String = pascal(hud) + "Hud"

    /** `buy_now` → `buyNow` (a dynamic slot member name). */
    fun slotMember(slot: String): String = camel(slot)

    /** `buy_now` → `onBuyNow` (a button handler member name). */
    fun buttonMember(button: String): String = "on" + pascal(button)

    /** `shop_menu` → `ShopMenu`. */
    private fun pascal(name: String): String =
        name
            .split('_')
            .filter { it.isNotEmpty() }
            .joinToString("") { word -> word.replaceFirstChar { it.uppercaseChar() } }

    /** `buy_now` → `buyNow`. */
    private fun camel(name: String): String {
        val parts = name.split('_').filter { it.isNotEmpty() }
        if (parts.isEmpty()) return ""
        return parts.first() +
            parts.drop(1).joinToString("") { word -> word.replaceFirstChar { it.uppercaseChar() } }
    }

    /** Whether [name] is a syntactically valid (non-keyword) Kotlin identifier. */
    fun isValidIdentifier(name: String): Boolean {
        if (name.isEmpty()) return false
        if (!(name[0].isLetter() || name[0] == '_')) return false
        if (!name.all { it.isLetterOrDigit() || it == '_' }) return false
        return name !in HARD_KEYWORDS
    }

    /**
     * Kotlin hard keywords — these can never be used as plain identifiers. Soft/modifier keywords
     * are legal identifiers and intentionally omitted.
     */
    private val HARD_KEYWORDS: Set<String> =
        setOf(
            "as",
            "break",
            "class",
            "continue",
            "do",
            "else",
            "false",
            "for",
            "fun",
            "if",
            "in",
            "interface",
            "is",
            "null",
            "object",
            "package",
            "return",
            "super",
            "this",
            "throw",
            "true",
            "try",
            "typealias",
            "typeof",
            "val",
            "var",
            "when",
            "while",
        )
}
