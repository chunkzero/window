package com.chunkzero.window.internal

import com.chunkzero.window.diagnostics.RenderLayerKind
import com.chunkzero.window.diagnostics.RenderStyleTrace
import com.chunkzero.window.manifest.HudEntry
import com.chunkzero.window.manifest.LayerKind
import net.kyori.adventure.text.Component

/**
 * The slots, switches, tints, and bars of a HUD-composed surface, and its layers composed through [composer].
 *
 * With [reactivity], every binding is evaluated under dependency capture. A surface without it re-evaluates every
 * binding on each [update].
 */
internal class HudLayers(
    /** The surface name used in semantic ids and messages, such as `hud`. */
    private val surface: String,
    private val ownerName: String,
    private val entry: HudEntry,
    private val composer: HudComposer,
    reactivity: Reactivity?,
) {
    val switches = Switches(surface, ownerName, entry.switches, reactivity)
    val tints = Tints(surface, ownerName, entry.tints, reactivity)
    val bars = Bars(surface, ownerName, entry.art, reactivity)
    val slots =
        DynamicSlots(
            surface,
            ownerName,
            entry.slots,
            reactivity,
            { id, slot, content -> composer.renderSlot(id, slot, content, tints.value(slot.tint)) },
        ) { id, slot ->
            emptySegment(
                id,
                RenderLayerKind.HUD_TEXT,
                slot.x,
                slot.y,
                slot.font,
                RenderStyleTrace(slot.color, slot.shadow),
                cursor = 0,
            )
        }

    private val contents = HashMap<String, Component>()
    private val segments = LinkedHashMap<String, RenderedSegment>()
    private var changed = false

    fun validate() {
        slots.validate()
        switches.validate()
        tints.validate()
        bars.validate()
    }

    /** Every binding's render key, for a full refresh. */
    fun renderKeys(): Set<RenderKey> =
        buildSet {
            slots.names.mapTo(this, RenderKey::Slot)
            switches.names.mapTo(this, RenderKey::Switch)
            tints.names.mapTo(this, RenderKey::Tint)
            bars.names.mapTo(this, RenderKey::Bar)
        }

    /** Evaluates every binding and composes the first render. */
    fun seed(): ComposedRender {
        for (name in switches.names) switches.update(name)
        for (name in tints.names) tints.update(name)
        for (name in bars.names) bars.update(name)
        slots.seed(::seedSlot) { name, segment -> segments[name] = segment }
        return compose()
    }

    /**
     * Re-evaluates the bindings [dirty] names, or every binding when it is `null`, and returns whether the layers
     * changed since the last [compose]. Slots whose provider returns an equal component, switches and tints that keep
     * their case or palette entry, and bars that keep their columns reuse their previous layout.
     */
    fun update(dirty: Set<RenderKey>?): Boolean {
        fun stale(key: RenderKey) = dirty == null || key in dirty
        for (name in switches.names) {
            if (stale(RenderKey.Switch(name))) changed = switches.update(name) || changed
        }
        for (name in tints.names) {
            if (!stale(RenderKey.Tint(name)) || !tints.update(name)) continue
            changed = true
            for (slot in slots.tinted(name)) {
                contents.remove(slot)
                slots.label(slot)?.let { segments[slot] = it }
            }
        }
        for (name in bars.names) {
            if (stale(RenderKey.Bar(name))) changed = bars.update(name) || changed
        }
        for (name in slots.names) {
            if (!stale(RenderKey.Slot(name)) && name in contents) continue
            val content = slots.content(name)
            if (contents[name] == content) continue
            segments[name] = slots.segment(name, content)
            contents[name] = content
            changed = true
        }
        return changed
    }

    private fun seedSlot(name: String): RenderedSegment {
        val content = slots.content(name)
        val segment = slots.segment(name, content)
        contents[name] = content
        return segment
    }

    /** Composes the layers in order, leaving out the entries of inactive switch cases. */
    fun compose(): ComposedRender {
        changed = false
        val layers =
            entry.layers.mapNotNull { layer ->
                val name = layer.name
                when (layer.kind) {
                    LayerKind.SLOT -> {
                        val segment = segments[name]?.takeIf { switches.slotActive(name) }
                        segment?.let {
                            HudLayer.Slot(
                                it.sourced(entry.slots[name]?.source ?: switches.entrySource(name)),
                            )
                        }
                    }

                    LayerKind.SWITCH -> {
                        val art = switches.activeCase(name)?.static?.takeIf { it.isNotEmpty() }
                        art?.let { composer.caseArt(switches.semanticId(name), name, it, switches.source(name)) }
                    }

                    LayerKind.ART -> {
                        val art = entry.art.getValue(name).takeIf { switches.artActive(name) }
                        art?.let {
                            val id = "$surface/$ownerName/$name"
                            composer.art(id, name, it, tints.value(it.tint), bars.columns(it.bar))
                        }
                    }

                    LayerKind.SPRITE_SLOT, LayerKind.COLLECTION -> {
                        null
                    }
                }
            }
        return composer.compose(ownerName, layers)
    }
}
