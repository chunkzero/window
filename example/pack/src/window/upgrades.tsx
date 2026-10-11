import {
    Bar,
    Box,
    Case,
    Image,
    Region,
    Switch,
    Text,
    create,
    createTheme,
    defineVars,
    variants,
} from "plugin:window/ui";
import type { Child } from "plugin:window/ui";
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
    card: { frame: art.panel, justify: "center", align: "center", gap: 3 },
    amount: variants({
        base: { text: { color: ink.name } },
        selected: { frame: art.buttonAccent, text: { color: "#2a1200", shadow: false } },
    }),
    strip: { frame: art.recess, padding: ink.pad, gap: 4 },
    name: { color: ink.name, align: "center" },
    level: { color: ink.muted, align: "center" },
    muted: { color: ink.muted },
});

/** What the next purchase of upgrade `index` costs. */
function Cost(props: { index: number }) {
    return (
        <Row gap={2} width={46}>
            <Text bind={upgradeCost.at(props.index)} align="right" color={ink.gold} />
            <Image art={coin} />
        </Row>
    );
}

/**
 * One face of upgrade `index`'s card: its name and level above `children`. `buy` makes the face take clicks, which
 * also outlines it on hover; a face without it only shows its tooltip.
 */
function Face(props: { index: number; tooltip: string; buy?: boolean; children: Child }) {
    const i = props.index;
    return [
        <Text bind={upgradeName.at(i)} width={46} style={s.name} />,
        <Text bind={upgradeLevel.at(i)} width={46} style={s.level} />,
        <Column height={19} justify="between" align="center">
            {props.children}
        </Column>,
        <Region onClick={props.buy === true ? upgrade.at(i) : undefined} tooltip={props.tooltip} />,
    ];
}

/** An upgrade card over 3x3 slots. Each stage is its own face, so a maxed card takes no clicks. */
function Card(props: { index: number }) {
    const i = props.index;
    return (
        <Box span={[3, 3]} padding={1}>
            <Switch bind={upgradeStage.at(i)} grow>
                <Case value="new" style={s.card}>
                    <Face index={i} tooltip="Click to upgrade" buy>
                        <Text style={s.level}>New</Text>
                        <Cost index={i} />
                    </Face>
                </Case>
                <Case value="open" style={s.card}>
                    <Face index={i} tooltip="Click to upgrade" buy>
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
                    </Face>
                </Case>
                <Case value="maxed" style={s.card}>
                    <Face index={i} tooltip="Fully upgraded">
                        <Center frame={maxedPlate} width={46} height={12} margin={{ top: "auto", bottom: "auto" }}>
                            <Text color="#2a1200" shadow={false}>
                                Maxed
                            </Text>
                        </Center>
                    </Face>
                </Case>
            </Switch>
        </Box>
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
            {Array.from({ length: UPGRADES }, (_, i) => (
                <Card index={i} />
            ))}
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
                <Hotspot tooltip={{ title: "Balance", lines: ["Coins shared with the shop"] }} hover />
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
