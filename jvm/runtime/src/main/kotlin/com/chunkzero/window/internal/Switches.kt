package com.chunkzero.window.internal

import com.chunkzero.window.manifest.SwitchCaseEntry
import com.chunkzero.window.manifest.SwitchEntry

/**
 * The switches of one window or HUD: binds each binding name to a lambda selecting the case of
 * every switch copy that shares it, and tracks which cases are active.
 *
 * A switch's selected case is the one its binding last returned, else its `initial` case. A case is
 * active while it is selected and its switch is active; a switch nested in a case is active only
 * while that case is, and so are the slots, sprite slots, and regions a case lists.
 */
internal class Switches(
    /** The surface name used in semantic ids and messages, `window` or `hud`. */
    private val surface: String,
    private val ownerName: String,
    private val switches: Map<String, SwitchEntry>,
    /** Captures state reads per binding; `null` for surfaces that render without reactivity. */
    private val reactivity: Reactivity?,
) {
    /** The switch keys of each binding name. State switches are selected by key through [select] instead. */
    private val bindings: Map<String, List<String>> =
        switches.filterValues { !it.states }.keys.groupBy { switches.getValue(it).binding ?: it }

    /** Binding names. */
    val names: Set<String> = bindings.keys

    private val parents = HashMap<String, Case>()
    private val slotCases = HashMap<String, Case>()
    private val spriteSlotCases = HashMap<String, Case>()
    private val regionCases = HashMap<String, Case>()

    /** The authored element owning each slot and sprite slot of a derived switch's cases. */
    private val sources = HashMap<String, String>()

    private val renders = HashMap<String, () -> String>()
    private val selected = HashMap<String, String>()

    init {
        for ((key, switch) in switches) {
            switch.initial?.let { selected[key] = it }
            for (case in switch.cases) {
                val owner = Case(key, case.value)
                for (name in case.switches) parents[name] = owner
                for (name in case.slots) slotCases[name] = owner
                for (name in case.spriteSlots) spriteSlotCases[name] = owner
                for (name in case.regions) regionCases[name] = owner
                val source = switch.source ?: continue
                for (name in case.slots + case.spriteSlots) sources[name] = source
            }
        }
    }

    fun bind(
        name: String,
        render: () -> String,
    ) {
        require(name in bindings) {
            "Unknown switch '$name' in $surface '$ownerName'; known switches: ${names.sorted()}"
        }
        require(renders.put(name, render) == null) { "Switch '$name' bound more than once" }
    }

    fun validate() {
        val unbound = names - renders.keys
        check(unbound.isEmpty()) { "Unbound switches in $surface '$ownerName': ${unbound.sorted()}" }
    }

    fun semanticId(key: String): String = "$surface/$ownerName/switch/$key"

    /** The authored element switch [key] comes from, if it is derived. */
    fun source(key: String): String? = switches.getValue(key).source

    /** The authored element a slot or sprite slot of a derived switch's case comes from. */
    fun entrySource(name: String): String? = sources[name]

    /**
     * Evaluates binding [name] under dependency capture and selects the returned case in every switch
     * sharing it; returns whether any selection changed.
     */
    fun update(name: String): Boolean {
        val render = renders.getValue(name)
        val value = if (reactivity == null) render() else reactivity.withRendering(RenderKey.Switch(name), render)
        var changed = false
        for (key in bindings.getValue(name)) changed = select(key, value) || changed
        return changed
    }

    /** Selects case [value] of switch [key]; returns whether the selection changed. */
    fun select(
        key: String,
        value: String,
    ): Boolean {
        val switch = switches.getValue(key)
        require(switch.cases.any { it.value == value }) {
            val known = switch.cases.map { it.value }
            if (switch.states) {
                "Unknown state '$value' for button '$key' in $surface '$ownerName'; known states: ${known.sorted()}"
            } else {
                "Unknown case '$value' for switch '${switch.binding ?: key}' in $surface '$ownerName'; " +
                    "known cases: $known"
            }
        }
        return selected.put(key, value) != value
    }

    /** The active case of switch [key], or `null` while the switch is inactive or has no case selected. */
    fun activeCase(key: String): SwitchCaseEntry? {
        if (!isActive(key)) return null
        val value = selected[key] ?: return null
        return switches.getValue(key).cases.first { it.value == value }
    }

    fun slotActive(name: String): Boolean = slotCases[name]?.let(::isActive) ?: true

    fun spriteSlotActive(name: String): Boolean = spriteSlotCases[name]?.let(::isActive) ?: true

    fun regionActive(name: String): Boolean = regionCases[name]?.let(::isActive) ?: true

    /**
     * Whether region [name] is active when each state switch selects the case [current] returns for its key,
     * instead of the case last rendered; a `null` result keeps the selected case.
     */
    fun regionActive(
        name: String,
        current: (String) -> String?,
    ): Boolean = regionCases[name]?.let { isActive(it, current) } ?: true

    private fun isActive(key: String): Boolean = parents[key]?.let(::isActive) ?: true

    private fun isActive(case: Case): Boolean = selected[case.switch] == case.value && isActive(case.switch)

    private fun isActive(
        case: Case,
        current: (String) -> String?,
    ): Boolean {
        if (parents[case.switch]?.let { isActive(it, current) } == false) return false
        val value = (if (switches.getValue(case.switch).states) current(case.switch) else null) ?: selected[case.switch]
        return value == case.value
    }

    private data class Case(
        val switch: String,
        val value: String,
    )
}
