package com.chunkzero.window

import net.kyori.adventure.text.Component

/**
 * The binding surface available inside [HudView.bind].
 *
 * Every dynamic slot in the HUD manifest must be bound exactly once; static labels carrying `text`
 * render automatically and must not be bound.
 *
 * Bindings are fixed once `bind()` returns: calling a binder on a retained scope afterwards throws
 * [IllegalStateException].
 */
public interface HudScope {
    /**
     * Binds a dynamic slot to a [render] lambda producing its current [Component].
     *
     * The lambda is re-invoked whenever a reactive state it reads changes. Its width is measured
     * from the plain-text content of the returned component; styling and children are preserved.
     */
    public fun slot(
        name: String,
        render: () -> Component,
    )

    /**
     * Binds a switch to a [render] lambda returning the value of the case to draw. Only the active
     * case's art and text are drawn; the lambda is re-invoked whenever a reactive state it reads
     * changes.
     */
    public fun switch(
        name: String,
        render: () -> String,
    )
}
