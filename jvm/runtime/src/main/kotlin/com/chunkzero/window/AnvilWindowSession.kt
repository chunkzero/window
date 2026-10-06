package com.chunkzero.window

import com.chunkzero.window.host.WindowHost
import com.chunkzero.window.internal.AnvilReopenGate
import com.chunkzero.window.internal.requireEntry

/**
 * A window with a static title and one anvil input. The anvil never reopens, so the player's edits
 * go straight to the input's handler.
 */
internal open class AnvilWindowSession<I : Any>(
    definition: WindowDefinition,
    view: WindowView<I>,
    host: WindowHost<I>,
) : ContainerWindowSession<I>(definition, view, host) {
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
    protected fun submit(value: String) {
        if (closed) return
        deliver(renderer.stageInput(input, value))
        handler(value)
    }

    override fun onInput(value: String) {
        // Drops the client's echo of a seed name it was sent.
        if (value == this.value) return
        this.value = value
        submit(value)
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
        deliver(renderer.applyInput(input, text))
        submit(text)
    }
}
