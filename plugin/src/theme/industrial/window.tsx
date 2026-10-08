/** Industrial's window shell, placed from its container's geometry. */
import { Box, Image, Section, Window as BaseWindow } from "../../ui/components.ts";
import { containerLayout } from "../../ui/containers.ts";
import type { Child, WindowProps as BaseWindowProps } from "../../ui/components.ts";
import type { Window as WindowDef } from "../../ui/document.ts";
import { assign } from "../../ui/styles.ts";
import type { TextProps } from "../../ui/styles.ts";
import type { ArtRef } from "../../ui/types.ts";
import { art } from "./art.ts";
import { Header } from "./layout.tsx";

export interface WindowProps extends Omit<BaseWindowProps, "frame" | "bleed" | "style"> {
    /** Header content, such as a label or bound `<Text>`, centered in a recess along the top edge. */
    title?: Child;
    /**
     * Shows the player's inventory and hotbar on a panel. `false` claims their slots, so no items show there, and
     * omits the panel; the window then rejects its own `<Player>` and `<Hotbar>` sections. An anvil window with an
     * `<Input>` also ends the shell below its slots, since packs with an `<Input>` hide vanilla's anvil art.
     */
    inventory?: boolean;
    /** Art of the shell; defaults to `art.shell`. */
    frame?: ArtRef;
}

interface Scan {
    claimsInventory: boolean;
    hasInput: boolean;
}

function scan(child: unknown, found: Scan = { claimsInventory: false, hasInput: false }): Scan {
    if (Array.isArray(child)) {
        child.forEach((entry) => scan(entry, found));
    } else if (typeof child === "object" && child !== null) {
        const node = child as { type?: unknown; section?: unknown; children?: unknown };
        found.claimsInventory ||= node.type === "section" && node.section !== "container";
        found.hasInput ||= node.type === "anvil_input";
        scan(node.children, found);
    }
    return found;
}

/**
 * A container window in the industrial shell: a raised shell with corner rivets and a hazard bar along its bottom, a
 * title recess, a panel under an anvil's input and slots, and a panel under the player's inventory. Labels default to
 * white small caps with a shadow.
 */
export function Window(props: WindowProps): { windows: WindowDef[] } {
    const { title, inventory = true, frame = art.shell, text, children, ...rest } = props;
    const { claimsInventory, hasInput } = scan(children);
    if (!inventory && claimsInventory) {
        throw new Error(
            "<Window inventory={false}> claims the player inventory and hotbar; remove its <Player> and <Hotbar> sections",
        );
    }
    const labels: Record<string, unknown> = { color: "#ffffff", shadow: true, smallCaps: true };
    assign(labels, (text ?? {}) as Record<string, unknown>);
    const layout = containerLayout(props.container);
    const { container, player, hotbar } = layout.sections;
    const panel = { x: 4, y: player.bounds.y - 3, bottom: hotbar.bounds.y + hotbar.bounds.height + 5 };
    const containerBottom = container.bounds.y + container.bounds.height + 5;
    const endsEarly = !inventory && hasInput;
    const end = endsEarly ? containerBottom : panel.bottom;
    const hazard = end + 5;
    const rivetY = endsEarly ? end : panel.y - 7;
    const rivets = [6, rivetY].flatMap((y) => [
        [-2, y],
        [173, y],
    ]);
    return (
        <BaseWindow
            {...rest}
            bleed={{ top: 1, right: 4, bottom: Math.max(0, hazard + 6 - layout.height), left: 4 }}
            text={labels as TextProps}
        >
            <Box frame={frame} x={-4} y={-1} width={184} height={hazard + 7} />
            {"input" in layout ? (
                <Box
                    frame={art.panel}
                    x={4}
                    y={layout.input.y - 5}
                    width={168}
                    height={containerBottom - (layout.input.y - 5)}
                />
            ) : null}
            {inventory ? (
                <Box frame={art.panel} x={panel.x} y={panel.y} width={168} height={panel.bottom - panel.y} />
            ) : (
                [<Section of="player" claim="all" />, <Section of="hotbar" claim="all" />]
            )}
            <Box frame={art.hazardBar} x={-4} y={hazard} width={184} height={6} />
            {rivets.map(([x, y]) => (
                <Image art={art.rivet} x={x} y={y} />
            ))}
            {title === undefined ? null : (
                <Header padding={{ top: 3 }} align="start">
                    <Box frame={art.recess} width={116} height={12} justify="center" align="center">
                        {title}
                    </Box>
                </Header>
            )}
            {children}
        </BaseWindow>
    ) as unknown as { windows: WindowDef[] };
}
