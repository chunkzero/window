package com.chunkzero.window

import com.chunkzero.window.host.FullscreenHost
import com.chunkzero.window.internal.LazyState
import com.chunkzero.window.internal.Reactivity
import com.chunkzero.window.internal.dynamicSlot
import net.kyori.adventure.text.Component
import kotlin.properties.ReadWriteProperty

/**
 * A user-defined view over a full-screen window: declares slot renders and button handlers, holds reactive state, and
 * reacts to lifecycle events.
 *
 * A view shows [definition] to the player of [host] and is single-use: [open] it once. The player aims a cursor by
 * turning, clicks with the attack and use keys, scrolls, types into a [promptText] or [promptTextDialog] prompt, and
 * leaves by sneaking.
 * Reactive [state] cells read inside render lambdas re-render the window when written.
 */
public abstract class FullscreenView(
    private val definition: FullscreenDefinition,
    /** The host showing this view to its player. */
    protected val host: FullscreenHost,
) {
    private var opened = false
    private var reactivity: Reactivity? = null
    private var session: FullscreenWindowSession? = null

    /**
     * Opens this view: runs [bind], shows the window, and calls [onOpen].
     *
     * A failed open still consumes the view: [onOpen] and [onClose] never run.
     *
     * @throws IllegalStateException if this view was already opened, a dynamic slot, switch, tint, or bar is unbound, an
     *   action lacks both a handler and a default, or the host could not show it.
     * @throws IllegalArgumentException for unknown names referenced in [bind].
     */
    public fun open(): FullscreenSession {
        check(!opened) { "Fullscreen view '${definition.name}' was already opened; views are single-use" }
        opened = true
        return FullscreenWindowSession(definition, this, host).also { it.open() }
    }

    /**
     * Creates a reactive state delegate with the given [initial] value.
     *
     * Reads during a render register that target as a dependent; writes schedule a single batched re-render on the
     * next tick. Write state only on the thread that serves the host's player.
     */
    protected fun <T> state(initial: T): ReadWriteProperty<Any?, T> = LazyState(initial) { reactivity }

    /**
     * Creates a [WindowList] showing [source] through [cells] cells, keyed by [key], as [WindowView.list] does: read
     * [WindowList.at] in a repeater's indexed renders, and back previous and next buttons with [WindowList.canPrevious]
     * and [WindowList.canNext]. [WindowList.items] adapts a list to item collections, which full-screen windows do not
     * have.
     *
     * @throws IllegalArgumentException if [cells] or [step] is not positive.
     */
    protected fun <T, K> list(
        cells: Int,
        key: (T) -> K,
        step: Int = cells,
        select: WindowList.Select = WindowList.Select.FIRST,
        source: () -> List<T>,
    ): WindowList<T, K> = WindowList(cells, key, step, select, source) { reactivity }

    /** Declares the slot renders and button handlers for this view. */
    protected abstract fun FullscreenScope.bind()

    /** Invoked after the window is shown. */
    protected open fun onOpen() {}

    /** Invoked when the window closes: by [close], a `window:close` control, the player sneaking, or disconnecting. */
    protected open fun onClose() {}

    /**
     * Closes this window and restores the player.
     *
     * @throws IllegalStateException if the view is not open.
     */
    protected fun close() {
        requireSession().close()
    }

    /**
     * Forces every render target to re-render on the next tick.
     *
     * @throws IllegalStateException if the view is not open.
     */
    protected fun refresh() {
        requireSession().refreshAll()
    }

    /**
     * Lets the player type on the canvas, starting from [initial]: the keyboard prompt. Nothing of the prompt is drawn,
     * so draw the text from [onEdit], which receives the whole text after each edit and re-renders at once rather than
     * on the next tick. [onDone] then receives the text once, when the player presses Escape or clicks anywhere. There
     * is no cancel; use [promptTextDialog] when the player must be able to back out. While the prompt is open the HUD
     * stays hidden, canvas input is ignored, so the cursor stays where it was, and the hovered control's tooltip is not
     * drawn.
     *
     * The prompt is an anvil the pack's shaders hide, so:
     * - the server never learns the client's caret, so a drawn caret belongs at the end of the text;
     * - the anvil holds 50 characters, so the text moves on in 50-character pieces: the earlier pieces cannot be edited,
     *   and characters typed while the next piece opens can be lost;
     * - GUI text and fills in the anvil's rename field and label bands, such as chat lines crossing them, are hidden
     *   while it is open;
     * - the system mouse pointer shows;
     * - each edit costs a round trip before [onEdit] runs;
     * - without the pack's shaders, the player sees a plain vanilla anvil.
     *
     * Opening a prompt of either kind while one is open replaces it, and the replaced prompt's [onDone] receives `null`.
     * When a server container opens over the prompt, its [onDone] receives `null` as well, on the next tick, and the
     * window stays open. When the window closes, it receives `null` once the player is restored. [onDone] runs after its
     * prompt has ended, so it may open another one; on a closed window it receives `null` at once.
     *
     * ```
     * override fun onRename(click: FullscreenClick) {
     *     draft = name
     *     promptText(name, onEdit = { draft = it }) { text ->
     *         draft = null
     *         text?.let { name = it }
     *     }
     * }
     * ```
     *
     * @throws IllegalStateException if the view was never opened.
     */
    @ExperimentalFullscreenApi
    protected fun promptText(
        initial: String = "",
        onEdit: (String) -> Unit,
        onDone: (String?) -> Unit,
    ) {
        requireSession().promptText(initial, onEdit, onDone)
    }

    /**
     * Opens a native dialog titled [title] over the canvas for the player to type into, starting from [initial], and
     * calls [onDone] once with the text when they confirm, or `null` when they cancel. The dialog closes with the button
     * and the canvas is back. The text holds at most 256 characters, and [initial] is cut to fit. Escape does not
     * dismiss the dialog. While it is open the HUD stays hidden, canvas input is ignored, and the hovered control's
     * tooltip is not drawn.
     *
     * Unlike [promptText], the dialog covers the canvas, but it offers a cancel, a movable caret, and selection, and
     * needs nothing from the pack. Replacement and [onDone] timing are as for [promptText].
     *
     * ```
     * override fun onRename(click: FullscreenClick) {
     *     promptTextDialog(Component.text("Rename"), name) { it?.let { name = it } }
     * }
     * ```
     *
     * @throws IllegalStateException if the view was never opened.
     */
    protected fun promptTextDialog(
        title: Component = Component.empty(),
        initial: String = "",
        onDone: (String?) -> Unit,
    ) {
        requireSession().promptTextDialog(title, initial, onDone)
    }

    /**
     * [value] shortened with an ellipsis so that it, followed by [suffix], fits the width of slot [slot] when drawn in
     * its font and style; followed by [suffix], which is kept even when it alone is wider. Content that already fits is
     * returned unchanged.
     *
     * @throws IllegalArgumentException if [slot] is not a dynamic text slot of this window.
     */
    protected fun fit(
        slot: String,
        value: Component,
        suffix: Component = Component.empty(),
    ): Component =
        definition.composer.fit(
            definition.layers.slots.dynamicSlot(slot, "fullscreen", definition.name),
            value,
            suffix,
        )

    internal fun attach(
        session: FullscreenWindowSession,
        reactivity: Reactivity,
    ) {
        this.session = session
        this.reactivity = reactivity
    }

    internal fun invokeBind(scope: FullscreenScope) {
        with(scope) { bind() }
    }

    internal fun invokeOnOpen() {
        onOpen()
    }

    internal fun invokeOnClose() {
        onClose()
    }

    private fun requireSession(): FullscreenWindowSession = checkNotNull(session) { "Fullscreen view is not open" }
}
