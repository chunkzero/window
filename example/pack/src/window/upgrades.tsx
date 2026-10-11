import { Bar, Box, Case, Image, Switch, Text, create, createTheme, defineVars, variants } from "plugin:window/ui";
import {
    Button,
    Center,
    Choice,
    Column,
    Container,
    Grid,
    Hotbar,
    Hotspot,
    Player,
    Repeater,
    Row,
    Spacer,
    Window,
    art,
    colors,
} from "plugin:window/theme/industrial";

import { barTrack, coin, maxedPlate, toneFill } from "./art.ts";
import {
    UPGRADES,
    amount,
    balance,
    openShop,
    rank,
    rankName,
    status,
    tone,
    upgrade,
    upgradeCost,
    upgradeIcon,
    upgradeLevel,
    upgradeName,
    upgradeProgress,
    upgradeStage,
} from "./handles.ts";

/** Industrial's colors, recolored for everything in this window. */
const forge = createTheme(colors, {
    border: "#1a0d08",
    highlight: "#e8a36b",
    shell: "#3a2620",
    panel: "#8a4b2c",
    recess: "#24140f",
    slot: "#5a3220",
    button: "#b3663a",
    selected: "#5a2c18",
});

const ink = defineVars({ name: "#ffe2b8", muted: "#f2c9a0", gold: "#ffb20b", pad: 4 });

const s = create({
    card: variants({ cell: { frame: art.panel, gap: 2 } }),
    amount: variants({
        base: { text: { color: ink.name } },
        selected: { frame: art.buttonAccent, text: { color: "#2a1200", shadow: false } },
    }),
    strip: { frame: art.recess, padding: ink.pad, gap: 4 },
    name: { color: ink.name, align: "center" },
    muted: { color: ink.muted },
});

/** What the next purchase of upgrade `index` costs. */
function Cost(props: { index: number }) {
    return (
        <Row gap={2}>
            <Text bind={upgradeCost.at(props.index)} align="right" color={ink.gold} />
            <Image art={coin} />
        </Row>
    );
}

const amounts = {
    one: { label: "x1", tooltip: "Buy one level" },
    five: { label: "x5", tooltip: "Buy five levels" },
    max: { label: "Max", tooltip: "Buy every level you can afford" },
};

export default (
    <Window name="upgrades" container="generic_9x3" title="Foundry Workshop" theme={forge}>
        <Container>
            <Repeater
                cell={[3, 3]}
                columns={UPGRADES}
                rows={1}
                onClick={upgrade}
                item={upgradeIcon}
                tooltip="Click to upgrade"
                style={s.card}
            >
                {(i) => [
                    <Box frame={art.slot} absolute left={0} top={0} width={16} height={16} />,
                    <Column width={46} gap={2}>
                        <Text bind={upgradeLevel.at(i)} align="right" style={s.muted} margin={{ top: 4, bottom: 4 }} />
                        <Text bind={upgradeName.at(i)} style={s.name} />
                        <Switch bind={upgradeStage.at(i)}>
                            <Case value="new" gap={2}>
                                <Text style={s.muted} align="center">
                                    New
                                </Text>
                                <Cost index={i} />
                            </Case>
                            <Case value="open" gap={2}>
                                <Center height={8}>
                                    <Bar
                                        bind={upgradeProgress.at(i)}
                                        width={46}
                                        height={4}
                                        fill={toneFill}
                                        track={barTrack}
                                        tint={ink.gold}
                                    />
                                </Center>
                                <Cost index={i} />
                            </Case>
                            <Case value="maxed" justify="center">
                                <Center frame={maxedPlate} height={12}>
                                    <Text color="#2a1200" shadow={false}>
                                        Maxed
                                    </Text>
                                </Center>
                            </Case>
                        </Switch>
                    </Column>,
                ]}
            </Repeater>
        </Container>

        <Player>
            {amount.values.map((value) => (
                <Choice bind={amount} value={value} span={2} tooltip={amounts[value].tooltip} style={s.amount}>
                    {amounts[value].label}
                </Choice>
            ))}
            <Row span={3} style={s.strip}>
                <Text bind={balance} align="right" color={ink.gold} />
                <Image art={coin} />
                {/* It takes no clicks, so `hover` opts it into the hover outline. */}
                <Hotspot tooltip={{ title: "Balance", lines: ["Coins earned in the shop"] }} hover />
            </Row>

            <Grid span={[9, 2]} columns={[16, "1fr"]} rows={[8, 8]} style={s.strip} alignContent="center">
                <Center row={{ span: 2 }}>
                    <Image bind={rank} size={[9, 11]} tint={tone} />
                </Center>
                <Text bind={rankName} color={ink.name} />
                <Text bind={status} style={s.muted} />
            </Grid>
        </Player>

        <Hotbar>
            <Button onClick={openShop} span={3} tooltip="Back to the shop">
                Shop
            </Button>
            <Spacer span={3} />
            <Button close span={3}>
                Exit
            </Button>
        </Hotbar>
    </Window>
);
