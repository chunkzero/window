package com.chunkzero.window.internal

import kotlin.properties.ReadWriteProperty
import kotlin.reflect.KProperty

/**
 * A view state delegate that defers to the session's [Reactivity] once [engine] returns one,
 * holding the value locally until then so construction-time state works.
 */
internal class LazyState<T>(
    initial: T,
    private val engine: () -> Reactivity?,
) : ReadWriteProperty<Any?, T> {
    private var delegate: ReadWriteProperty<Any?, T>? = null
    private var pending: T = initial

    private fun resolve(): ReadWriteProperty<Any?, T>? {
        if (delegate == null) {
            delegate = engine()?.state(pending)
        }
        return delegate
    }

    override fun getValue(
        thisRef: Any?,
        property: KProperty<*>,
    ): T {
        val resolved = resolve() ?: return pending
        return resolved.getValue(thisRef, property)
    }

    override fun setValue(
        thisRef: Any?,
        property: KProperty<*>,
        value: T,
    ) {
        val resolved = resolve()
        if (resolved == null) {
            pending = value
        } else {
            resolved.setValue(thisRef, property, value)
        }
    }
}

/**
 * A cached computation that uses the session's [Reactivity] memo once [engine] returns one, and
 * recomputes on every [get] until then.
 */
internal class LazyMemo<T>(
    private val engine: () -> Reactivity?,
    private val compute: () -> T,
) {
    private var memo: Reactivity.Memo<T>? = null

    fun get(): T {
        val resolved = memo ?: engine()?.memo(compute)?.also { memo = it } ?: return compute()
        return resolved.get()
    }
}
