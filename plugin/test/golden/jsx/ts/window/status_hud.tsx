import { Hud, Image, Text, industrial, text as textHandle } from "#plugins/window";
import type { TextHandle, TextProps } from "#plugins/window";

import { coin } from "./sprites.ts";

const text: TextProps = { shadow: true, smallCaps: true, color: "#ffffff" };
const muted = "#8fb3d9";
const soft = "#e0edff";

/** A label and a right-aligned value; `coin` follows the value with a coin. */
function Stat(props: { label: string; bind: TextHandle; color?: string; coin?: boolean }) {
    return (
        <industrial.Row gap={2}>
            <Text width={34} color={muted}>
                {props.label}
            </Text>
            <Text bind={props.bind} align="right" color={props.color} />
            {props.coin === true && <Image art={coin} />}
        </industrial.Row>
    );
}

export default (
    <>
        <Hud
            name="status_top_left"
            anchor="top-left"
            offset={[4, 4]}
            frame={industrial.art.hud}
            padding={4}
            gap={2}
            minWidth={80}
            text={text}
        >
            <Stat label="Coins" bind={textHandle("coins")} color="#ffd75e" coin />
            <Stat label="Rate" bind={textHandle("rate")} color="#80ff80" />
            <Stat label="Power" bind={textHandle("power")} />
        </Hud>

        <Hud name="status_top_center" anchor="top" offset={[0, 4]} frame={industrial.art.hud} padding={3} text={text}>
            <Text bind={textHandle("runtime")} width={34} align="center" />
        </Hud>

        <Hud
            name="status_top_right"
            anchor="top-right"
            offset={[-4, 4]}
            frame={industrial.art.hud}
            padding={4}
            gap={2}
            minWidth={80}
            text={text}
        >
            <Stat label="Wave" bind={textHandle("wave")} />
            <Stat label="Biome" bind={textHandle("biome")} />
            <Stat label="Ping" bind={textHandle("latency")} />
        </Hud>

        <Hud name="status_left_side" anchor="left" offset={[4, 0]} gap={2} text={text}>
            <Text bind={textHandle("coords")} width={64} />
            <Text bind={textHandle("altitude")} width={64} color={soft} />
        </Hud>

        <Hud name="status_right_side" anchor="right" offset={[-4, 0]} gap={2} text={{ ...text, align: "right" }}>
            <Text bind={textHandle("objective")} width={80} color="#ffd75e" />
            <Text bind={textHandle("stock")} width={80} color={soft} />
        </Hud>

        <Hud name="status_bottom_center" anchor="bottom" offset={[0, -72]} text={text}>
            <Text bind={textHandle("hint")} width={160} align="center" color={soft} />
        </Hud>
    </>
);
