package com.chunkzero.window.example

import com.chunkzero.window.ExperimentalFullscreenApi
import com.chunkzero.window.FullscreenClick
import com.chunkzero.window.example.generated.LobbyFullscreen
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import kotlin.math.abs

/**
 * The full-screen lobby: a paged world list moved by the arrows or the scroll wheel, a foundry name edited in place on
 * its plate, and a ready flag flipped by right clicking anywhere.
 */
class MyLobby(
    player: Player,
) : LobbyFullscreen(player) {
    private var name by state("${player.username}'s Foundry")
    private var isReady by state(false)

    /** The name as typed so far while the rename prompt is open. */
    private var draft: String? by state(null)
    private val worlds = list(cells = WORLD_NAME_SIZE, key = { it: String -> it }, step = 1) { WORLDS }

    override fun foundryName(): Component = draft?.let(::editing) ?: fit("foundry_name", Component.text(name))

    override fun lobbyStatus(): Component =
        Component.text(
            when {
                draft != null -> "Type a name, then click or press Esc"
                isReady -> "Ready for ${worlds.selected}"
                else -> "${worlds.selected} selected"
            },
        )

    override fun tone(): Tone {
        val pinned = pinnedTone(player)
        if (pinned != null) return Tone.entries.first { it.value == pinned }
        return if (isReady) Tone.OK else Tone.WARN
    }

    override fun ready(): Boolean = isReady

    override fun worldName(index: Int): Component = Component.text(worlds.at(index).orEmpty())

    override fun onPickWorld(
        index: Int,
        click: FullscreenClick,
    ) {
        worlds.selectAt(index)
    }

    override fun onPreviousWorld(click: FullscreenClick) = worlds.previous()

    override fun onNextWorld(click: FullscreenClick) = worlds.next()

    override fun onScroll(steps: Int) = repeat(abs(steps)) { if (steps > 0) worlds.next() else worlds.previous() }

    override fun onRightClick(click: FullscreenClick) {
        isReady = !isReady
    }

    /** Edits the name in place: the plate shows the text as it is typed, until the player clicks or presses Escape. */
    @OptIn(ExperimentalFullscreenApi::class)
    override fun onRename(click: FullscreenClick) {
        draft = name
        promptText(name, onEdit = { draft = it }) { text ->
            draft = null
            text?.trim()?.takeIf(String::isNotEmpty)?.let { name = it }
        }
    }

    /** [text] and a caret, dropping its start once it outgrows the plate so the end stays in view. */
    private fun editing(text: String): Component =
        (0..text.length)
            .asSequence()
            .map { Component.text(text.substring(it) + "_") }
            .first { fit("foundry_name", it) == it }

    override fun onPlay(click: FullscreenClick) {
        if (!isReady) return
        close()
        player.sendMessage(Component.text("$name joins ${worlds.selected}."))
    }
}

private val WORLDS = listOf("Ironvale", "Emberly", "Ashmoor", "Slagport", "Rustwood", "Kilnmere", "Coalcrag")
