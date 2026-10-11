import { Bar, Box, Hud, Image, Switch, Text } from "plugin:window/ui";
import type { TextProps } from "plugin:window/ui";
import { flag, fraction, text } from "plugin:window/bind";
import type { TextHandle } from "plugin:window/bind";
import { Row, art } from "plugin:window/theme/industrial";

import { barTrack, coin, toneFill, tonePill } from "./art.ts";
import clock from "./clock_hud.ts";
import { tone } from "./handles.ts";

export const coins = text("coins");
export const rate = text("rate");
export const power = text("power");
const wave = text("wave");
const biome = text("biome");
const latency = text("latency");
const coords = text("coords");
const altitude = text("altitude");
const objective = text("objective");
const stock = text("stock");
export const hint = text("hint");
const waveProgress = flag("wave_progress", { shape: [10] });
/** How far the current wave's countdown has run. */
export const waveCharge = fraction("wave_charge");

const white = "#ffffff";
const style: TextProps = { shadow: true, smallCaps: true, color: white };
const muted = "#8fb3d9";
const soft = "#e0edff";

/** A label and a right-aligned value; `coin` follows the value with a coin. */
function Stat(props: { label: string; bind: TextHandle; color?: string; coin?: boolean }) {
    return (
        <Row gap={2}>
            <Text width={34} color={muted}>
                {props.label}
            </Text>
            <Text bind={props.bind} align="right" color={props.color ?? white} />
            {props.coin === true && <Image art={coin} />}
        </Row>
    );
}

export const topLeft = (
    <Hud
        name="status_top_left"
        anchor="top-left"
        offset={[4, 4]}
        frame={art.hud}
        padding={4}
        gap={2}
        minWidth={80}
        text={style}
    >
        <Stat label="Coins" bind={coins} color="#ffd75e" coin />
        <Stat label="Rate" bind={rate} color="#80ff80" />
        <Stat label="Power" bind={power} />
    </Hud>
);

const topRight = (
    <Hud
        name="status_top_right"
        anchor="top-right"
        offset={[-4, 4]}
        frame={art.hud}
        padding={4}
        gap={2}
        minWidth={80}
        text={style}
    >
        <Stat label="Wave" bind={wave} />
        <Stat label="Biome" bind={biome} />
        <Stat label="Ping" bind={latency} />
    </Hud>
);

const leftSide = (
    <Hud name="status_left_side" anchor="left" offset={[4, 0]} gap={2} text={style}>
        <Text bind={coords} width={64} />
        <Text bind={altitude} width={64} color={soft} />
    </Hud>
);

const rightSide = (
    <Hud name="status_right_side" anchor="right" offset={[-4, 0]} gap={2} text={{ ...style, align: "right" }}>
        <Text bind={objective} width={80} color="#ffd75e" />
        <Text bind={stock} width={80} color={soft} />
    </Hud>
);

export const bottomCenter = (
    <Hud name="status_bottom_center" anchor="bottom" offset={[0, -72]} gap={1} align="center" text={style}>
        <Box frame={tonePill} tint={tone} padding={{ top: 2, right: 6, bottom: 1, left: 6 }}>
            <Text bind={hint} width={100} align="center" color={tone} />
        </Box>
        <Row gap={1}>
            {Array.from({ length: 10 }, (_, i) => (
                <Switch on={waveProgress.at(i)}>
                    {{ true: <Image art={art.lampOn} />, false: <Image art={art.lampOff} /> }}
                </Switch>
            ))}
        </Row>
    </Hud>
);

/**
 * Window's own bar where vanilla draws the experience bar, which `vanilla.hud.experience` blanks: 182x5, centered,
 * its top 29 px above the bottom of the GUI.
 */
export const waveBar = (
    <Hud name="status_wave_bar" anchor="bottom" offset={[0, -24]}>
        <Bar bind={waveCharge} width={182} height={5} fill={toneFill} track={barTrack} tint={tone} />
    </Hud>
);

/** The status HUDs, one per anchor. */
export default [topLeft, clock, topRight, leftSide, rightSide, bottomCenter, waveBar];
