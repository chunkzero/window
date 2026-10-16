package com.chunkzero.window

/**
 * Marks full-screen window API that relies on client behaviour Window cannot fully control, such as the keyboard text
 * prompt, which disguises a vanilla anvil. Its behaviour and signature may change between releases.
 */
@RequiresOptIn(
    message = "This full-screen window API is experimental and may change between releases.",
    level = RequiresOptIn.Level.WARNING,
)
@Retention(AnnotationRetention.BINARY)
@Target(AnnotationTarget.FUNCTION, AnnotationTarget.CLASS)
public annotation class ExperimentalFullscreenApi
