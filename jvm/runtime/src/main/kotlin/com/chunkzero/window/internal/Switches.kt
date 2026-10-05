package com.chunkzero.window.internal

import com.chunkzero.window.manifest.SwitchCaseEntry
import com.chunkzero.window.manifest.SwitchEntry

/**
 * The switches of one window or HUD: binds each to a lambda selecting its active case, validates
 * that each is bound, and reports the slots that inactive cases hide.
 */
internal class Switches(
    /** The surface name used in semantic ids and messages, `window` or `hud`. */
    private val surface: String,
    private val ownerName: String,
    private val switches: Map<String, SwitchEntry>,
    private val reactivity: Reactivity,
) {
    val names: Set<String> = switches.keys

    private val renders = HashMap<String, () -> String>()
    private val active = HashMap<String, String>()

    fun bind(
        name: String,
        render: () -> String,
    ) {
        require(name in switches) {
            "Unknown switch '$name' in $surface '$ownerName'; known switches: ${names.sorted()}"
        }
        require(renders.put(name, render) == null) { "Switch '$name' bound more than once" }
    }

    fun validate() {
        val unbound = names - renders.keys
        check(unbound.isEmpty()) { "Unbound switches in $surface '$ownerName': ${unbound.sorted()}" }
    }

    fun semanticId(name: String): String = "$surface/$ownerName/switch/$name"

    /** Selects switch [name]'s case under dependency capture and returns it. */
    fun render(name: String): SwitchCaseEntry {
        val render = renders.getValue(name)
        val value = reactivity.withRendering(RenderKey.Switch(name)) { render() }
        val cases = switches.getValue(name).cases
        val selected =
            cases.firstOrNull { it.value == value }
                ?: throw IllegalArgumentException(
                    "Unknown case '$value' for switch '$name' in $surface '$ownerName'; " +
                        "known cases: ${cases.map { it.value }}",
                )
        active[name] = value
        return selected
    }

    /** Text slots of every case that is not active. */
    fun hiddenSlots(): Set<String> = hidden { it.slots }

    /** Sprite slots of every case that is not active. */
    fun hiddenSpriteSlots(): Set<String> = hidden { it.spriteSlots }

    private fun hidden(members: (SwitchCaseEntry) -> List<String>): Set<String> =
        buildSet {
            for ((name, switch) in switches) {
                for (case in switch.cases) {
                    if (case.value != active[name]) addAll(members(case))
                }
            }
        }
}
