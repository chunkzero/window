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
import net.minestom.server.network.packet.server.common.ClearDialogPacket
import net.minestom.server.network.packet.server.common.ShowDialogPacket
import java.util.concurrent.atomic.AtomicLong

/**
 * The dialog a full-screen window shows [player] to type into: a text field over a Confirm and a Cancel button, which
 * report through custom click actions whose ids are unique to the prompt, so responses to any other dialog are ignored.
 *
 * The client closes the dialog when a button is pressed, and Escape does nothing, so a prompt ends with a button, with
 * another prompt replacing it, or with [cancel]. Each of those reports to its callback once, after the prompt has left
 * [current], so a callback may open the next prompt. The buttons sit in the dialog's body, clear of the footer, which
 * falls inside the strip the pack's shader clips while the HUD is hidden.
 */
internal class MinestomTextPrompt(
    private val player: Player,
) {
    @Volatile
    private var pending: Pending? = null

    /** The prompt awaiting an answer, if any. */
    val current: Pending? get() = pending

    /** Shows the dialog, first reporting `null` to the prompt it replaces. */
    fun open(
        title: Component,
        initial: String,
        onDone: (String?) -> Unit,
    ) {
        val id = PROMPTS.getAndIncrement()
        val next = Pending(Key.key("window", "prompt/$id/confirm"), Key.key("window", "prompt/$id/cancel"), onDone)
        val replaced = pending
        pending = next
        player.sendPacket(ShowDialogPacket(dialog(title, initial.take(MAX_LENGTH), next)))
        replaced?.onDone?.invoke(null)
    }

    /** Reports a button press of the open prompt, and nothing else. */
    fun onClick(event: PlayerCustomClickEvent) {
        val current = pending ?: return
        val text =
            when (event.key) {
                current.confirm -> {
                    ((event.payload as? CompoundBinaryTag)?.get(FIELD) as? StringBinaryTag)?.value()?.take(MAX_LENGTH)
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

    /** Takes the open prompt's dialog, if any, off the client's screen; the prompt itself ends with [cancel]. */
    fun clear() {
        if (pending != null) player.sendPacket(ClearDialogPacket())
    }

    /** Reports `null` to the open prompt, if any; with [only], just when that prompt is still the open one. */
    fun cancel(only: Pending? = null) {
        val current = pending ?: return
        if (only != null && only !== current) return
        pending = null
        current.onDone(null)
    }

    private fun dialog(
        title: Component,
        initial: String,
        prompt: Pending,
    ): Dialog =
        Dialog.MultiAction(
            DialogMetadata(
                title,
                null,
                false,
                false,
                DialogAfterAction.CLOSE,
                emptyList(),
                listOf(DialogInput.Text(FIELD, DialogInput.DEFAULT_WIDTH, title, false, initial, MAX_LENGTH, null)),
            ),
            listOf(button("Confirm", prompt.confirm), button("Cancel", prompt.cancel)),
            null,
            2,
        )

    private fun button(
        label: String,
        id: Key,
    ) = DialogActionButton(Component.text(label), null, BUTTON_WIDTH, DialogAction.DynamicCustom(id, null))

    class Pending internal constructor(
        internal val confirm: Key,
        internal val cancel: Key,
        internal val onDone: (String?) -> Unit,
    )

    private companion object {
        const val FIELD = "text"
        const val MAX_LENGTH = 256
        const val BUTTON_WIDTH = 100
        val PROMPTS = AtomicLong()
    }
}
