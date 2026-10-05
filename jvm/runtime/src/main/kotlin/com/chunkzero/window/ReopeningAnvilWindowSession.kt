package com.chunkzero.window

import com.chunkzero.window.internal.AnvilReopenGate
import com.chunkzero.window.internal.InventoryHandle
import com.chunkzero.window.internal.RenderScheduler
import com.chunkzero.window.internal.requireEntry
import net.minestom.server.entity.Player

/**
 * An anvil input window whose title changes, which experimental anvil updates allow. Each change
 * reopens the anvil, so title sends and edits pass through an [AnvilReopenGate].
 */
internal class ReopeningAnvilWindowSession(
    definition: WindowDefinition,
    view: WindowView,
    player: Player,
    scheduler: RenderScheduler,
    handle: InventoryHandle,
    diagnosticsObserver: RenderDiagnosticsObserver,
) : AnvilWindowSession(definition, view, player, scheduler, handle, diagnosticsObserver) {
    private val gate = AnvilReopenGate(scheduler, handle, input.initial, ::deliver) { sendTitle(reopen = true) }

    override fun send(action: () -> Unit): Boolean = gate.send(action)

    override fun onInput(value: String) = gate.input(value)

    override fun onPong(id: Int) = gate.pong(id)

    override fun onClientPacket() = gate.release()

    override fun setInput(
        name: String,
        value: String,
    ) {
        definition.requireEntry(entry.inputs, name, "anvil input", known = "inputs")
        if (!closed) gate.replace(value)
    }
}
