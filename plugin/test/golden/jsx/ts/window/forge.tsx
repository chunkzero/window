import { Box, Case, Hud, Image, Switch, Text, Window } from "plugin:window/ui";
import { action, flag, items, sprite, text, toggle, value } from "plugin:window/bind";
import type { TextHandle } from "plugin:window/bind";
import {
    Button,
    Container,
    Grid,
    Header,
    Hotbar,
    Hotspot,
    Player,
    Repeater,
    Row,
    Show,
    Slots,
    Spacer,
    Toggle,
    art,
} from "plugin:window/theme/industrial";

import { coin } from "./sprites.ts";

const style = { color: "#ffffff", shadow: true, smallCaps: true } as const;

const mode = value("mode", ["idle", "forging"]);
const progress = text("progress");
const recipe = action("recipe", { shape: [3] });
const stack = items("stack", { shape: [3] });
const recipeName = text("recipe_name", { shape: [3] });
const cost = text("cost", { shape: [3] });
const forge = action("forge");
const forgeIcon = sprite("forge_icon");
const forgeLabel = text("forge_label");
const status = text("status");
const auto = toggle("auto");
const autoLamp = sprite("auto_lamp", { only: ["lamp_on", "lamp_off"] });
const overheated = flag("overheated");

function Meter(props: { label: string; bind: TextHandle; color: string }) {
    return (
        <Row gap={2} justify="center">
            <Text color="#8fb3d9">{props.label}</Text>
            <Text bind={props.bind} width={18} color={props.color} />
        </Row>
    );
}

export default (
    <>
        <Window
            name="forge"
            container="generic_9x4"
            frame={art.shell}
            bleed={{ top: 1, right: 4, left: 4 }}
            text={style}
        >
            <Header padding={{ top: 3 }} align="start">
                <Row frame={art.recess} height={12} padding={{ left: 4, right: 4 }} gap={3}>
                    <Image art={coin} translate={[0, 1]} />
                    <Text>Rune Forge</Text>
                    <Image art={coin} translate={[0, 1]} />
                </Row>
            </Header>
            <Switch bind={mode} x={146} y={3}>
                <Case value="idle" frame={art.buttonDanger} width={24} height={12} justify="center" align="center">
                    <Text>New</Text>
                </Case>
                <Case value="forging" frame={art.buttonAccent} justify="center" align="center">
                    <Text bind={progress} width={20} align="center" color="#2a1200" shadow={false} />
                </Case>
            </Switch>

            <Container frame={art.panel} outset={{ top: 2, right: 3, bottom: 3, left: 3 }}>
                <Repeater onClick={recipe} item={stack} cell={[3, 2]} columns={3} rows={1}>
                    {(i) => (
                        <>
                            <Text bind={recipeName.at(i)} width={50} align="center" />
                            <Row gap={1}>
                                <Text bind={cost.at(i)} width={20} align="right" color="#ffb20b" />
                                <Image art={coin} />
                            </Row>
                        </>
                    )}
                </Repeater>

                <Grid span={9} columns={3} frame={art.recess} align="center">
                    <Meter label="Heat" bind={text("heat")} color="#ff8300" />
                    <Meter label="Ore" bind={text("ore")} color="#bceeff" />
                    <Meter label="Time" bind={text("time")} color="#80ff80" />
                </Grid>

                <Hotspot tooltip={{ title: "Forging", lines: ["Pick a recipe", "Then press Forge"] }} />
                <Spacer span={4} />
                <Button onClick={forge} span={4} frame={art.buttonAccent} tooltip="Forge the selected recipe">
                    <Image bind={forgeIcon} size={8} />
                    <Text bind={forgeLabel} color="#2a1200" shadow={false} />
                </Button>
            </Container>

            <Player>
                <Slots span={[9, 2]} />
                <Row span={6} frame={art.recess} padding={{ left: 4, right: 4 }}>
                    <Text bind={status} color="#bceeff" />
                </Row>
                <Toggle bind={auto} span={3} on={{ tooltip: "Auto forge on" }} off={{ tooltip: "Auto forge off" }}>
                    <Image bind={autoLamp} size={4} />
                    Auto
                </Toggle>
            </Player>

            <Hotbar>
                <Button close at={[3, 0]} span={3} tooltip="Leave the forge">
                    Leave
                </Button>
            </Hotbar>
        </Window>

        <Hud
            name="forge_progress"
            anchor="bottom"
            offset={[0, -60]}
            frame={art.hud}
            padding={3}
            gap={2}
            minWidth={90}
            text={style}
        >
            <Row gap={4}>
                <Text grow>Forging</Text>
                <Show when={overheated}>
                    <Row frame={art.recess} padding={{ left: 2, right: 2 }} gap={2}>
                        <Image art={coin} />
                        <Text color="#ff8300">Hot</Text>
                    </Row>
                </Show>
                <Text bind={text("percent")} width={24} align="right" color="#ffd75e" />
            </Row>
            <Grid columns={["1fr", "1fr"]} gap={[4, 1]}>
                <Text bind={text("item")} color="#e0edff" />
                <Text bind={text("eta")} align="right" color="#8fb3d9" />
            </Grid>
        </Hud>
    </>
);
