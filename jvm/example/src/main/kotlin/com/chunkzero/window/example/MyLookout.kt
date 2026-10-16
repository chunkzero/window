package com.chunkzero.window.example

import com.chunkzero.window.FullscreenClick
import com.chunkzero.window.example.generated.LookoutFullscreen
import net.minestom.server.entity.Player

/** The horse-mode lookout: each Charge click fills a quarter of the signal bar, and a full bar starts over. */
class MyLookout(
    player: Player,
) : LookoutFullscreen(player) {
    private var charges by state(0)

    override fun signal(): Float = charges / 4f

    override fun onCharge(click: FullscreenClick) {
        charges = (charges + 1) % 5
    }
}
