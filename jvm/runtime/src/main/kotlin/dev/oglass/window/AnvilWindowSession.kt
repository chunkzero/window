package dev.oglass.window

import dev.oglass.window.internal.AnvilReopenGate
import dev.oglass.window.internal.InventoryHandle
import dev.oglass.window.internal.RenderScheduler
import dev.oglass.window.internal.requireEntry
import net.minestom.server.entity.Player

/**
 * A window with a static title and one anvil input. The anvil never reopens, so the player's edits
 * go straight to the input's handler.
 */
internal open class AnvilWindowSession(
    definition: WindowDefinition,
    view: WindowView,
    player: Player,
    scheduler: RenderScheduler,
    handle: InventoryHandle,
    diagnosticsObserver: RenderDiagnosticsObserver,
) : WindowSession(definition, view, player, scheduler, handle, diagnosticsObserver) {
    private val name = entry.inputs.keys.single()
    protected val input = entry.inputs.getValue(name)
    private lateinit var handler: (String) -> Unit

    /** The input's text as of the latest edit the handler received. */
    private var value = input.initial

    override fun bindInput() {
        handler = bindings.inputHandlers.getValue(name)
    }

    /**
     * Stages [value] as the seed's name, since reopens and resyncs reset the edit box to it, then
     * passes it to the handler.
     */
    protected fun deliver(value: String) {
        if (closed) return
        writer.stageInput(input, value)
        handler(value)
    }

    override fun onInput(value: String) {
        // Drops the client's echo of a seed name it was sent.
        if (value == this.value) return
        this.value = value
        deliver(value)
    }

    override fun setInput(
        name: String,
        value: String,
    ) {
        definition.requireEntry(entry.inputs, name, "anvil input", known = "inputs")
        if (closed) return
        val text = value.take(AnvilReopenGate.MAX_NAME_LENGTH)
        if (text == this.value) return
        this.value = text
        // Sending the renamed seed sets the client's edit box in place.
        writer.applyInput(input, text)
        deliver(text)
    }
}
