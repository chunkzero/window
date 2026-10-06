import {
    Button,
    Case,
    Container,
    Grid,
    Header,
    Hotbar,
    Hotspot,
    Hud,
    Icon,
    Item,
    Player,
    Repeater,
    Row,
    Show,
    Slots,
    Spacer,
    Sprite,
    Switch,
    Text,
    Toggle,
    Window,
} from "#plugins/window";

const text = { color: "#ffffff", shadow: true, smallCaps: true };

function Meter(props: { label: string; bind: string; color: string }) {
    return (
        <Row gap={2} justify="center">
            <Text color="#8fb3d9">{props.label}</Text>
            <Text bind={props.bind} width={18} color={props.color} />
        </Row>
    );
}

export default (
    <>
        <Window name="forge" container="generic_9x4" frame="shell" bleed={{ top: 1, right: 4, left: 4 }} text={text}>
            <Header padding={{ top: 3 }} align="start">
                <Row frame="recess" height={12} padding={{ left: 4, right: 4 }} gap={3}>
                    <Sprite name="coin" translate={[0, 1]} />
                    <Text>Rune Forge</Text>
                    <Sprite name="coin" translate={[0, 1]} />
                </Row>
            </Header>
            <Switch bind="mode" x={146} y={3}>
                <Case value="idle" frame="button_danger" width={24} height={12} justify="center" align="center">
                    <Text>New</Text>
                </Case>
                <Case value="forging" frame="button_accent" justify="center" align="center">
                    <Text bind="progress" width={20} align="center" color="#2a1200" shadow={false} />
                </Case>
            </Switch>

            <Container frame="panel" outset={{ top: 2, right: 3, bottom: 3, left: 3 }}>
                <Repeater name="recipe" cell={[3, 2]} columns={3} rows={1} frame="button">
                    <Item name="stack" cellSlot={1} />
                    <Text bind="name" width={50} align="center" />
                    <Row gap={1}>
                        <Text bind="cost" width={20} align="right" color="#ffb20b" />
                        <Sprite name="coin" />
                    </Row>
                </Repeater>

                <Grid span={9} columns={3} frame="recess" align="center">
                    <Meter label="Heat" bind="heat" color="#ff8300" />
                    <Meter label="Ore" bind="ore" color="#bceeff" />
                    <Meter label="Time" bind="time" color="#80ff80" />
                </Grid>

                <Hotspot name="help" tooltip={{ title: "Forging", lines: ["Pick a recipe", "Then press Forge"] }} />
                <Spacer span={4} />
                <Button name="forge" span={4} frame="button_accent" tooltip="Forge the selected recipe">
                    <Icon bind="forge_icon" size={8} />
                    <Text bind="forge_label" color="#2a1200" shadow={false} />
                </Button>
            </Container>

            <Player>
                <Slots name="materials" frame="slot" span={[9, 2]} claim="none" />
                <Row span={6} frame="recess" padding={{ left: 4, right: 4 }}>
                    <Text bind="status" color="#bceeff" />
                </Row>
                <Toggle
                    name="auto"
                    span={3}
                    frame="button"
                    on={{ tooltip: "Auto forge on" }}
                    off={{ tooltip: "Auto forge off" }}
                >
                    <Icon bind="auto_lamp" size={4} />
                    Auto
                </Toggle>
            </Player>

            <Hotbar>
                <Button name="leave" at={[3, 0]} span={3} frame="button" close tooltip="Leave the forge">
                    Leave
                </Button>
            </Hotbar>
        </Window>

        <Hud
            name="forge_progress"
            anchor="bottom"
            offset={[0, -60]}
            frame="hud"
            padding={3}
            gap={2}
            minWidth={90}
            text={text}
        >
            <Row gap={4}>
                <Text grow>Forging</Text>
                <Show when="overheated">
                    <Row frame="recess" padding={{ left: 2, right: 2 }} gap={2}>
                        <Sprite name="coin" />
                        <Text color="#ff8300">Hot</Text>
                    </Row>
                </Show>
                <Text bind="percent" width={24} align="right" color="#ffd75e" />
            </Row>
            <Grid columns={["1fr", "1fr"]} gap={[4, 1]}>
                <Text bind="item" color="#e0edff" />
                <Text bind="eta" align="right" color="#8fb3d9" />
            </Grid>
        </Hud>
    </>
);
