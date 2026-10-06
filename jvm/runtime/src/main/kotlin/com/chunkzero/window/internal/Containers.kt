package com.chunkzero.window.internal

import com.chunkzero.window.HudChannel
import com.chunkzero.window.host.ContainerKind

/** Maps window-core container kind ids to [ContainerKind]s. */
internal object Containers {
    private val byKind =
        mapOf(
            "generic_9x1" to ContainerKind.CHEST_1_ROW,
            "generic_9x2" to ContainerKind.CHEST_2_ROW,
            "generic_9x3" to ContainerKind.CHEST_3_ROW,
            "generic_9x4" to ContainerKind.CHEST_4_ROW,
            "generic_9x5" to ContainerKind.CHEST_5_ROW,
            "generic_9x6" to ContainerKind.CHEST_6_ROW,
            "anvil" to ContainerKind.ANVIL,
        )

    private val byChannel =
        mapOf(
            "actionbar" to HudChannel.ACTION_BAR,
            "bossbar" to HudChannel.BOSS_BAR,
            "sidebar" to HudChannel.SIDEBAR,
        )

    /**
     * Resolves a container kind id to its [ContainerKind].
     *
     * @throws IllegalArgumentException if the kind is not a supported container.
     */
    fun kind(container: String): ContainerKind =
        byKind[container]
            ?: throw IllegalArgumentException(
                "Unsupported container kind '$container'; supported: ${byKind.keys.sorted()}",
            )

    /**
     * Resolves a HUD channel id to its [HudChannel].
     *
     * @throws IllegalArgumentException if the channel is not supported.
     */
    fun channel(channel: String): HudChannel =
        byChannel[channel]
            ?: throw IllegalArgumentException(
                "Unsupported HUD channel '$channel'; supported: ${byChannel.keys.sorted()}",
            )
}
