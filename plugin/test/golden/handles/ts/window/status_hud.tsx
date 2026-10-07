import { Hud, Row, Sprite, Switch, Text, text, value } from "#plugins/window";
import type { TextHandle, TextProps } from "#plugins/window";

const coins = text("coins");
const rate = text("rate");
const power = text("power");
const runtime = text("runtime");
const wave = text("wave");
const biome = text("biome");
const latency = text("latency");
const coords = text("coords");
const altitude = text("altitude");
const objective = text("objective");
const stock = text("stock");
const hint = text("hint");
const waveProgress = value("wave_progress", ["on", "off"], { shape: [10] });

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
            {props.coin === true && <Sprite name="coin" />}
        </Row>
    );
}

/** The status HUDs, one per anchor. */
export default [
    <Hud
        name="status_top_left"
        anchor="top-left"
        offset={[4, 4]}
        frame="hud"
        padding={4}
        gap={2}
        minWidth={80}
        text={style}
    >
        <Stat label="Coins" bind={coins} color="#ffd75e" coin />
        <Stat label="Rate" bind={rate} color="#80ff80" />
        <Stat label="Power" bind={power} />
    </Hud>,

    <Hud name="status_top_center" anchor="top" offset={[0, 4]} frame="hud" padding={3} text={style}>
        <Text bind={runtime} width={34} align="center" />
    </Hud>,

    <Hud
        name="status_top_right"
        anchor="top-right"
        offset={[-4, 4]}
        frame="hud"
        padding={4}
        gap={2}
        minWidth={80}
        text={style}
    >
        <Stat label="Wave" bind={wave} />
        <Stat label="Biome" bind={biome} />
        <Stat label="Ping" bind={latency} />
    </Hud>,

    <Hud name="status_left_side" anchor="left" offset={[4, 0]} gap={2} text={style}>
        <Text bind={coords} width={64} />
        <Text bind={altitude} width={64} color={soft} />
    </Hud>,

    <Hud name="status_right_side" anchor="right" offset={[-4, 0]} gap={2} text={{ ...style, align: "right" }}>
        <Text bind={objective} width={80} color="#ffd75e" />
        <Text bind={stock} width={80} color={soft} />
    </Hud>,

    <Hud name="status_bottom_center" anchor="bottom" offset={[0, -72]} gap={2} align="center" text={style}>
        <Text bind={hint} width={160} align="center" color={soft} />
        <Row gap={1}>
            {Array.from({ length: 10 }, (_, i) => (
                <Switch on={waveProgress.at(i)}>
                    {{ on: <Sprite name="lamp_on" />, off: <Sprite name="lamp_off" /> }}
                </Switch>
            ))}
        </Row>
    </Hud>,
];
