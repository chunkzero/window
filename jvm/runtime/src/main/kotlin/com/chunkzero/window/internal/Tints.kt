package com.chunkzero.window.internal

import net.kyori.adventure.text.format.TextColor

/**
 * The tints of one window or HUD: binds each tint to a lambda picking its palette entry, and tracks the
 * entry each last picked.
 */
internal class Tints(
    /** The surface name used in messages, `window` or `hud`. */
    private val surface: String,
    private val ownerName: String,
    /** Each tint's palette: its entries' names and `#rrggbb` colors. */
    private val palettes: Map<String, Map<String, String>>,
    /** Captures state reads per binding; `null` for surfaces that render without reactivity. */
    private val reactivity: Reactivity?,
) {
    /** Tint ids. */
    val names: Set<String> = palettes.keys

    private val renders = HashMap<String, () -> String>()
    private val picked = HashMap<String, String>()

    fun bind(
        name: String,
        render: () -> String,
    ) {
        require(name in palettes) {
            "Unknown tint '$name' in $surface '$ownerName'; known tints: ${names.sorted()}"
        }
        require(renders.put(name, render) == null) { "Tint '$name' bound more than once" }
    }

    fun validate() {
        val unbound = names - renders.keys
        check(unbound.isEmpty()) { "Unbound tints in $surface '$ownerName': ${unbound.sorted()}" }
    }

    /**
     * Evaluates tint [name]'s binding under dependency capture and picks the returned palette entry;
     * returns whether the entry changed.
     */
    fun update(name: String): Boolean {
        val render = renders.getValue(name)
        val value = if (reactivity == null) render() else reactivity.withRendering(RenderKey.Tint(name), render)
        val palette = palettes.getValue(name)
        require(value in palette) {
            "Unknown palette entry '$value' for tint '$name' in $surface '$ownerName'; " +
                "known entries: ${palette.keys.sorted()}"
        }
        return picked.put(name, value) != value
    }

    /** The palette entry tint [name] last picked; `null` without a tint. */
    fun value(name: String?): String? = name?.let(picked::getValue)
}

/** The color of entry [value] of tint [tint] in these palettes, or `null` when [tint] or [value] is. */
internal fun Map<String, Map<String, String>>.tintColor(
    tint: String?,
    value: String?,
): TextColor? {
    if (tint == null || value == null) return null
    return requireColor(getValue(tint).getValue(value))
}
