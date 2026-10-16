package com.chunkzero.window.minestom.internal

import net.kyori.adventure.key.Key
import net.kyori.adventure.nbt.CompoundBinaryTag
import net.kyori.adventure.nbt.StringBinaryTag
import net.kyori.adventure.text.Component
import net.minestom.server.dialog.Dialog
import net.minestom.server.dialog.DialogAction
import net.minestom.server.dialog.DialogActionButton
import net.minestom.server.dialog.DialogAfterAction
import net.minestom.server.dialog.DialogInput
import net.minestom.server.dialog.DialogMetadata
import net.minestom.server.entity.Player
import net.minestom.server.event.player.PlayerCustomClickEvent
import net.minestom.server.event.player.PlayerPacketEvent
import net.minestom.server.item.ItemStack
import net.minestom.server.network.packet.server.ServerPacket
import net.minestom.server.network.packet.server.common.ClearDialogPacket
import net.minestom.server.network.packet.server.common.ShowDialogPacket
import java.util.concurrent.atomic.AtomicLong

/**
 * The text prompt of [player]'s full-screen window, and the one owner of its state: [current] is the open prompt, a
 * native dialog or the anvil keyboard ([MinestomAnvilKeyboard]), and opening either kind replaces it, taking the
 * replaced kind off the client first.
 *
 * The dialog has a text field over a Confirm and a Cancel button, which report through custom click actions whose ids
 * are unique to the prompt, so responses to any other dialog are ignored. The client closes the dialog when a button
 * is pressed, and Escape does nothing. The buttons sit in the dialog's body, clear of the footer, which falls inside
 * the strip the pack's shader clips while the HUD is hidden. The keyboard ends with its text when the player clicks or
 * presses Escape.
 *
 * Either kind also ends with `null` when replaced, when a server container takes over (on the next tick), or with
 * [cancel]. Each end reports to its callback once, after the prompt has left [current], so a callback may open the
 * next prompt.
 */
internal class MinestomTextPrompt(
    private val player: Player,
    blank: ItemStack,
    sendBand: () -> Unit,
    restoreSlots: () -> Unit,
) {
    @Volatile
    private var pending: Pending? = null

    /** The dialog the writer has sent most recently and not cleared, which may no longer be [current]. */
    @Volatile
    private var shown: Pending.Dialog? = null

    private val keyboard =
        MinestomAnvilKeyboard(
            player,
            blank,
            sendBand,
            restoreSlots,
            object : MinestomAnvilKeyboard.Events {
                override fun onEdit(text: String) {
                    (pending as? Pending.Keyboard)?.onEdit(text)
                }

                override fun onFinish(text: String) {
                    val current = pending as? Pending.Keyboard ?: return
                    pending = null
                    current.onDone(text)
                }

                override fun onDisplaced() {
                    val displaced = pending as? Pending.Keyboard ?: return
                    player.scheduler().scheduleNextTick { cancel(displaced) }
                }
            },
        )

    /** The prompt awaiting an answer, if any. */
    val current: Pending? get() = pending

    /** Whether the keyboard's anvil is open, so the world age belongs in the prompt band. */
    val keyboardOpen: Boolean get() = keyboard.isOpen

    /** Shows the dialog, first reporting `null` to the prompt it replaces. */
    fun openDialog(
        title: Component,
        initial: String,
        onDone: (String?) -> Unit,
    ) {
        val id = PROMPTS.getAndIncrement()
        val confirm = Key.key("window", "prompt/$id/confirm")
        val cancel = Key.key("window", "prompt/$id/cancel")
        val show = ShowDialogPacket(dialog(title, initial.take(DIALOG_MAX_LENGTH), confirm, cancel))
        val next = Pending.Dialog(confirm, cancel, show, onDone)
        val replaced = pending
        pending = next
        keyboard.close()
        player.sendPacket(next.show)
        replaced?.onDone?.invoke(null)
    }

    /**
     * Opens the keyboard, first reporting `null` to the prompt it replaces. A keyboard replacing a keyboard keeps its
     * anvil open. When an inventory listener refuses the anvil, the new prompt reports `null` as well.
     */
    fun openKeyboard(
        initial: String,
        onEdit: (String) -> Unit,
        onDone: (String?) -> Unit,
    ) {
        val next = Pending.Keyboard(onEdit, onDone)
        val replaced = pending
        pending = next
        if (replaced is Pending.Dialog) player.sendPacket(ClearDialogPacket())
        val opened = keyboard.open(initial)
        try {
            replaced?.onDone?.invoke(null)
        } finally {
            if (!opened) cancel(next)
        }
    }

    /** Reports a button press of the open dialog, and nothing else. */
    fun onClick(event: PlayerCustomClickEvent) {
        val current = pending as? Pending.Dialog ?: return
        val text =
            when (event.key) {
                current.confirm -> {
                    ((event.payload as? CompoundBinaryTag)?.get(FIELD) as? StringBinaryTag)
                        ?.value()
                        ?.take(DIALOG_MAX_LENGTH)
                        ?: return
                }

                current.cancel -> {
                    null
                }

                else -> {
                    return
                }
            }
        pending = null
        current.onDone(text)
    }

    /** Reports the keyboard's edits and its end. */
    fun onPacket(event: PlayerPacketEvent) {
        keyboard.onPacket(event)
    }

    /** Whether the open packet being written for [windowId] is the keyboard's; see [MinestomAnvilKeyboard.ownsOpen]. */
    fun ownsOpen(windowId: Int): Boolean = keyboard.ownsOpen(windowId)

    /** Takes the open prompt off the client, if any; the prompt itself ends with [cancel]. */
    fun clear() {
        if (pending is Pending.Dialog) player.sendPacket(ClearDialogPacket())
        keyboard.close()
    }

    /** Records, from the packets as the writer sends them, which prompt's dialog the client is showing. */
    fun onWritten(packet: ServerPacket) {
        when (packet) {
            is ShowDialogPacket -> shown = (pending as? Pending.Dialog)?.takeIf { it.show == packet }
            is ClearDialogPacket -> shown = null
            else -> Unit
        }
    }

    /**
     * Cancels the open prompt on the player's next tick when a container replaces the dialog the client is showing.
     * A prompt whose dialog has not been sent yet is unaffected, as its dialog arrives after the container. The
     * keyboard's anvil reports its own replacement.
     */
    fun coverByContainer() {
        val covered = shown?.takeIf { it === pending } ?: return
        shown = null
        player.scheduler().scheduleNextTick { cancel(covered) }
    }

    /** Reports `null` to the open prompt, if any; with [only], just when that prompt is still the open one. */
    fun cancel(only: Pending? = null) {
        val current = pending ?: return
        if (only != null && only !== current) return
        pending = null
        if (current is Pending.Keyboard) keyboard.close()
        current.onDone(null)
    }

    private fun dialog(
        title: Component,
        initial: String,
        confirm: Key,
        cancel: Key,
    ): Dialog =
        Dialog.MultiAction(
            DialogMetadata(
                title,
                null,
                false,
                false,
                DialogAfterAction.CLOSE,
                emptyList(),
                listOf(
                    DialogInput.Text(FIELD, DialogInput.DEFAULT_WIDTH, title, false, initial, DIALOG_MAX_LENGTH, null),
                ),
            ),
            listOf(button("Confirm", confirm), button("Cancel", cancel)),
            null,
            2,
        )

    private fun button(
        label: String,
        id: Key,
    ) = DialogActionButton(Component.text(label), null, BUTTON_WIDTH, DialogAction.DynamicCustom(id, null))

    sealed class Pending(
        internal val onDone: (String?) -> Unit,
    ) {
        class Dialog internal constructor(
            internal val confirm: Key,
            internal val cancel: Key,
            internal val show: ShowDialogPacket,
            onDone: (String?) -> Unit,
        ) : Pending(onDone)

        class Keyboard internal constructor(
            internal val onEdit: (String) -> Unit,
            onDone: (String?) -> Unit,
        ) : Pending(onDone)
    }

    private companion object {
        const val FIELD = "text"
        const val DIALOG_MAX_LENGTH = 256
        const val BUTTON_WIDTH = 100
        val PROMPTS = AtomicLong()
    }
}
