package dev.oglass.window.example

import com.sun.net.httpserver.HttpExchange
import com.sun.net.httpserver.HttpServer
import net.kyori.adventure.resource.ResourcePackCallback
import net.kyori.adventure.resource.ResourcePackInfo
import net.kyori.adventure.resource.ResourcePackRequest
import net.kyori.adventure.text.Component
import java.net.InetSocketAddress
import java.net.URI
import java.nio.file.Files
import java.nio.file.Path
import java.security.MessageDigest
import java.util.UUID
import java.util.concurrent.Executors

/** Serves the built example resource-pack zip and creates the client pack request. */
class PackServer(
    private val packZip: Path,
    private val bindPort: Int,
    private val externalUrl: String,
) {
    private val bytes: ByteArray = Files.readAllBytes(packZip)
    private var server: HttpServer? = null

    val sha1Hex: String =
        MessageDigest.getInstance("SHA-1").digest(bytes).joinToString("") { "%02x".format(it) }

    val packId: UUID = UUID.nameUUIDFromBytes(sha1Hex.toByteArray())

    fun start() {
        if (server != null) return
        val http = HttpServer.create(InetSocketAddress(bindPort), 0)
        http.executor =
            Executors.newSingleThreadExecutor { runnable ->
                Thread(runnable, "window-example-pack").apply { isDaemon = true }
            }
        http.createContext("/") { exchange -> handle(exchange) }
        http.start()
        server = http
    }

    fun stop() {
        server?.stop(0)
        server = null
    }

    fun request(
        prompt: Component,
        callback: ResourcePackCallback = ResourcePackCallback.noOp(),
    ): ResourcePackRequest {
        val info =
            ResourcePackInfo
                .resourcePackInfo()
                .id(packId)
                .uri(URI.create(externalUrl))
                .hash(sha1Hex)
                .build()
        return ResourcePackRequest
            .resourcePackRequest()
            .packs(info)
            .callback(callback)
            .prompt(prompt)
            .required(true)
            .replace(true)
            .build()
    }

    private fun handle(exchange: HttpExchange) {
        exchange.use {
            if (exchange.requestMethod != "GET") {
                exchange.sendResponseHeaders(405, -1)
                return
            }
            exchange.responseHeaders.add("Content-Type", "application/zip")
            exchange.sendResponseHeaders(200, bytes.size.toLong())
            exchange.responseBody.use { body -> body.write(bytes) }
        }
    }

    private inline fun HttpExchange.use(block: () -> Unit) {
        try {
            block()
        } finally {
            close()
        }
    }
}
