package com.chunkzero.window.example

import com.chunkzero.window.FullscreenClick
import com.chunkzero.window.example.generated.LobbyFullscreen
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import kotlin.math.abs

/**
 * The full-screen lobby: a paged world list moved by the arrows or the scroll wheel, a foundry name renamed through a
 * native dialog, and a ready flag flipped by right clicking anywhere.
 */
class MyLobby(
    player: Player,
) : LobbyFullscreen(player) {
    private var name by state("${player.username}'s Foundry")
    private var isReady by state(false)
    private val worlds = list(cells = WORLD_NAME_SIZE, key = { it: String -> it }, step = 1) { WORLDS }

    override fun foundryName(): Component = fit("foundry_name", Component.text(name))

    override fun lobbyStatus(): Component =
        Component.text(if (isReady) "Ready for ${worlds.selected}" else "${worlds.selected} selected")

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

    /** Asks for a new name in a native dialog; the lobby is back once the player confirms or cancels. */
    override fun onRename(click: FullscreenClick) {
        promptText(Component.text("Name your foundry"), name) { text ->
            text?.trim()?.takeIf(String::isNotEmpty)?.let { name = it }
        }
    }

    override fun onPlay(click: FullscreenClick) {
        if (!isReady) return
        close()
        player.sendMessage(Component.text("$name joins ${worlds.selected}."))
    }
}

private val WORLDS = listOf("Ironvale", "Emberly", "Ashmoor", "Slagport", "Rustwood", "Kilnmere", "Coalcrag")
