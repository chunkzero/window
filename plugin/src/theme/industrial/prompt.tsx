/** Industrial's anvil text prompt: a window with a native input, an optional label, and back/confirm buttons. */
import { Input, Text } from "../../ui/components.ts";
import { containerLayout } from "../../ui/containers.ts";
import type { ClickAction, Input as InputHandle } from "../../bind/handles.ts";
import type { Child } from "../../ui/components.ts";
import type { Window as WindowDef } from "../../ui/document.ts";
import type { Tooltip } from "../../ui/types.ts";
import { art } from "./art.ts";
import { Button } from "./controls.tsx";
import { Container } from "./layout.tsx";
import { Window } from "./window.tsx";
import type { WindowProps } from "./window.tsx";

export interface PromptProps extends Omit<WindowProps, "container" | "inventory"> {
    /** The `input` handle receiving the typed value. */
    bind: InputHandle;
    initial?: string;
    /** Static text drawn left of the text field. */
    label?: string;
    /** Clicked on the anvil's third slot. */
    onConfirm: ClickAction;
    /** Clicked on the anvil's first slot; without it, that slot has no button. */
    onBack?: ClickAction;
    /** Content of the confirm button; defaults to "OK". */
    confirm?: Child;
    confirmTooltip?: string | Tooltip;
    /** Content of the back button; defaults to "<". */
    back?: Child;
    /** Extra window content, drawn after the prompt's. */
    children?: Child;
}

/**
 * An anvil text prompt in the industrial shell, without the player's inventory: `label` beside the native field, a back
 * button on the first anvil slot, and an accent confirm button on the third.
 */
export function Prompt(props: PromptProps): { windows: WindowDef[] } {
    const {
        bind,
        initial,
        label,
        onConfirm,
        onBack,
        confirm = "OK",
        confirmTooltip,
        back = "<",
        children,
        ...rest
    } = props;
    const { input } = containerLayout("anvil");
    const labelX = 10;
    return (
        <Window {...rest} container="anvil" inventory={false}>
            {label === undefined ? null : (
                <Text x={labelX} y={input.y + 4} width={input.x - 5 - labelX} color="#bceeff">
                    {label}
                </Text>
            )}
            <Input bind={bind} initial={initial} />
            <Container claim="none">
                {onBack === undefined ? null : (
                    <Button onClick={onBack} at={[0, 0]}>
                        {back}
                    </Button>
                )}
                <Button onClick={onConfirm} at={[2, 0]} frame={art.buttonAccent} tooltip={confirmTooltip}>
                    {confirm}
                </Button>
            </Container>
            {children}
        </Window>
    ) as unknown as { windows: WindowDef[] };
}
