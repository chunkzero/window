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
 * another prompt replacing it, or with [close]. Each of those reports to its callback once. The buttons sit in the
 * dialog's body, clear of the footer, which falls inside the strip the pack's shader clips while the HUD is hidden.
 */
internal class MinestomTextPrompt(
    private val player: Player,
) {
    private var pending: Pending? = null

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

    /** Reports `null` to the open prompt, if any, and takes its dialog off the client's screen when [clear]. */
    fun close(clear: Boolean) {
        val current = pending ?: return
        pending = null
        if (clear) player.sendPacket(ClearDialogPacket())
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

    private class Pending(
        val confirm: Key,
        val cancel: Key,
        val onDone: (String?) -> Unit,
    )

    private companion object {
        const val FIELD = "text"
        const val MAX_LENGTH = 256
        const val BUTTON_WIDTH = 100
        val PROMPTS = AtomicLong()
    }
}
