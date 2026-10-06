package com.chunkzero.window.host

import com.chunkzero.window.ButtonTooltip
import net.kyori.adventure.key.Key

/** Items Window renders itself, described by meaning; hosts translate them to native items. */
public sealed interface WindowItem {
    /**
     * Invisible click target (item model [model]) with an optional tooltip, max stack 1.
     *
     * [tooltip] is final: its title is the item name and its lines the lore, used as given. `null` hides the tooltip.
     */
    public data class Hitbox(
        val model: Key,
        val tooltip: ButtonTooltip?,
    ) : WindowItem

    /**
     * Anvil input seed: its name is the text box contents; no enchantments component; tooltip hidden; max stack 1.
     *
     * Seeds with different non-zero [revision]s must be different items to the client, even with equal [text], because
     * the client only resets its text box when the seed changes. `0` adds no revision.
     */
    public data class AnvilSeed(
        val model: Key,
        val text: String,
        val revision: Int = 0,
    ) : WindowItem
}
