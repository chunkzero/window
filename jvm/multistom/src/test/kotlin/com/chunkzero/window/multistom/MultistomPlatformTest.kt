package com.chunkzero.window.multistom

import com.chunkzero.window.WindowScope
import com.chunkzero.window.WindowView
import com.chunkzero.window.Windows
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import net.kyori.adventure.text.Component
import net.minestom.server.ServerProcess
import net.minestom.server.entity.Player
import net.minestom.server.item.ItemStack
import net.minestom.server.listener.WindowListener
import net.minestom.server.network.packet.client.play.ClientClickWindowPacket
import net.minestom.server.network.packet.server.SendablePacket
import net.minestom.server.network.player.GameProfile
import net.minestom.server.network.player.PlayerConnection
import java.net.InetSocketAddress
import java.net.SocketAddress
import java.util.UUID

class MultistomPlatformTest :
    StringSpec({
        "a window opens, routes clicks, and re-renders on the player's own process" {
            val process = ServerProcess.create()
            val player = Player(SilentConnection(process), GameProfile(UUID.randomUUID(), "WindowTest"))
            val view =
                object : WindowView("menu") {
                    var clicks by state(0)

                    override fun WindowScope.bind() {
                        slot("title") { Component.text("Clicks $clicks") }
                        button("press") { clicks++ }
                    }
                }

            val session = Windows.load(MANIFEST, MultistomPlatform).open(player, view)
            try {
                player.openInventory shouldBe session.inventory
                val opened = session.inventory.title

                WindowListener.clickWindowListener(
                    ClientClickWindowPacket(
                        session.inventory.windowId.toInt(),
                        0,
                        0,
                        0,
                        ClientClickWindowPacket.ClickType.PICKUP,
                        emptyMap<Short, ItemStack.Hash>(),
                        ItemStack.Hash.AIR,
                    ),
                    player,
                )
                view.clicks shouldBe 1

                process.schedulerManager().processTick()
                session.inventory.title shouldNotBe opened
            } finally {
                session.close()
            }
        }
    })

private class SilentConnection(
    process: ServerProcess,
) : PlayerConnection(process) {
    override fun sendPacket(packet: SendablePacket) = Unit

    override fun getRemoteAddress(): SocketAddress = InetSocketAddress("127.0.0.1", 25565)
}

private val MANIFEST =
    """
    {
      "version": 5,
      "namespace": "window",
      "font": "window:ui",
      "spacers": { "983040": -1024, "983061": 1024, "983050": -1, "983051": 1 },
      "text_advances": { " ": 4, "C": 6, "c": 6, "i": 2, "k": 6, "l": 3, "s": 6, "0": 6, "1": 6 },
      "text_glyph_widths": { " ": 0, "C": 5, "c": 5, "i": 1, "k": 5, "l": 2, "s": 5, "0": 5, "1": 5 },
      "font_metrics": {
        "window:y0": {
          "advances": { " ": 4, "C": 6, "c": 6, "i": 2, "k": 6, "l": 3, "s": 6, "0": 6, "1": 6 },
          "glyph_widths": { " ": 0, "C": 5, "c": 5, "i": 1, "k": 5, "l": 2, "s": 5, "0": 5, "1": 5 },
          "bold_advance": 1
        }
      },
      "windows": {
        "menu": {
          "surface": {
            "kind": "container",
            "container": "generic_9x1",
            "size": [176, 132],
            "title_origin": [8, 6]
          },
          "static": "",
          "slots": {
            "title": {
              "x": 8, "y": 6, "width": 160, "align": "left",
              "font": "window:y0", "color": "#404040", "shadow": false
            }
          },
          "buttons": {
            "press": {
              "x": 8, "y": 18, "width": 18, "height": 18,
              "slots": [{ "area": "container", "index": 0 }],
              "default": null
            }
          }
        }
      }
    }
    """.trimIndent()
