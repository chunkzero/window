package com.chunkzero.window.internal

/** An action the runtime resolves itself, named by an id in the `window:` namespace. */
internal enum class RuntimeAction(
    val id: String,
) {
    /** Closes the window. */
    CLOSE("window:close"),
    ;

    companion object {
        private val byId = entries.associateBy { it.id }

        /** The runtime action [id] names, or `null` for an unknown id. */
        fun of(id: String): RuntimeAction? = byId[id]
    }
}
