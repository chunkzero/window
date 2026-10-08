package com.chunkzero.window.internal

import com.chunkzero.window.manifest.SwitchCaseEntry
import com.chunkzero.window.manifest.SwitchEntry

/**
 * The switches of one window or HUD: binds each switch to a lambda selecting its case, and tracks
 * which cases are active.
 *
 * A switch's selected case is the one its binding last returned. A case is
 * active while it is selected and its switch is active; a switch nested in a case is active only
 * while that case is, and so are the slots, sprite slots, regions, items, and collections a case lists.
 */
internal class Switches(
    /** The surface name used in semantic ids and messages, `window` or `hud`. */
    private val surface: String,
    private val ownerName: String,
    private val switches: Map<String, SwitchEntry>,
    /** Captures state reads per binding; `null` for surfaces that render without reactivity. */
    private val reactivity: Reactivity?,
) {
    /** Switch keys. */
    val names: Set<String> = switches.keys

    private val parents = HashMap<String, Case>()
    private val slotCases = HashMap<String, Case>()
    private val spriteSlotCases = HashMap<String, Case>()
    private val regionCases = HashMap<String, Case>()
    private val itemCases = HashMap<String, Case>()
    private val collectionCases = HashMap<String, Case>()

    /** The `debug_name` of the switch owning each slot and sprite slot of its cases. */
    private val sources = HashMap<String, String>()

    private val renders = HashMap<String, () -> String>()
    private val selected = HashMap<String, String>()

    init {
        for ((key, switch) in switches) {
            for (case in switch.cases) {
                val owner = Case(key, case.value)
                for (name in case.switches) parents[name] = owner
                for (name in case.slots) slotCases[name] = owner
                for (name in case.spriteSlots) spriteSlotCases[name] = owner
                for (name in case.regions) regionCases[name] = owner
                for (name in case.items) itemCases[name] = owner
                for (name in case.collections) collectionCases[name] = owner
                val source = switch.source ?: continue
                for (name in case.slots + case.spriteSlots) sources[name] = source
            }
        }
    }

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

    fun semanticId(key: String): String = "$surface/$ownerName/switch/$key"

    /** The `debug_name` of switch [key], if it has one. */
    fun source(key: String): String? = switches.getValue(key).source

    /** The `debug_name` of the switch whose case holds slot or sprite slot [name]. */
    fun entrySource(name: String): String? = sources[name]

    /**
     * Evaluates switch [name]'s binding under dependency capture and selects the returned case;
     * returns whether the selection changed.
     */
    fun update(name: String): Boolean {
        val render = renders.getValue(name)
        val value = if (reactivity == null) render() else reactivity.withRendering(RenderKey.Switch(name), render)
        val switch = switches.getValue(name)
        require(switch.cases.any { it.value == value }) {
            "Unknown case '$value' for switch '$name' in $surface '$ownerName'; " +
                "known cases: ${switch.cases.map { it.value }}"
        }
        return selected.put(name, value) != value
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

    fun itemActive(name: String): Boolean = itemCases[name]?.let(::isActive) ?: true

    fun collectionActive(name: String): Boolean = collectionCases[name]?.let(::isActive) ?: true

    /**
     * Reads the case each switch selects now rather than at its last render, for routing clicks: each binding is
     * evaluated without dependency capture. A binding that throws keeps the selected case. Each switch is read at
     * most once per lookup.
     */
    fun current(): (String) -> String? {
        val values = HashMap<String, String?>()
        return { key -> values.getOrPut(key) { renders[key]?.let { render -> runCatching(render).getOrNull() } } }
    }

    /** Whether region [name] is active when each switch selects the case [current] returns for its key. */
    fun regionActive(
        name: String,
        current: (String) -> String?,
    ): Boolean = regionCases[name]?.let { isActive(it, current) } ?: true

    /** Whether collection [name] is active when each switch selects the case [current] returns for its key. */
    fun collectionActive(
        name: String,
        current: (String) -> String?,
    ): Boolean = collectionCases[name]?.let { isActive(it, current) } ?: true

    private fun isActive(key: String): Boolean = parents[key]?.let(::isActive) ?: true

    private fun isActive(case: Case): Boolean = selected[case.switch] == case.value && isActive(case.switch)

    private fun isActive(
        case: Case,
        current: (String) -> String?,
    ): Boolean {
        if (parents[case.switch]?.let { isActive(it, current) } == false) return false
        val value = current(case.switch) ?: selected[case.switch]
        return value == case.value
    }

    private data class Case(
        val switch: String,
        val value: String,
    )
}
