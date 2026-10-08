/** Handles: the named values and events a window exchanges with its generated Kotlin view. */
export { action, builtin, collection, flag, input, items, selection, sprite, text, toggle, value } from "./handles.ts";
export type {
    Action as ActionHandle,
    Builtin as BuiltinHandle,
    BuiltinId,
    ClickAction,
    Collection as CollectionHandle,
    Condition,
    Flag as FlagHandle,
    HandleKind,
    Indexed,
    Input as InputHandle,
    Is,
    Items as ItemsHandle,
    Ref,
    Selection as SelectionHandle,
    Set as SetAction,
    Shape,
    Sprite as SpriteHandle,
    Text as TextHandle,
    Toggle as ToggleHandle,
    Value as ValueHandle,
} from "./handles.ts";
