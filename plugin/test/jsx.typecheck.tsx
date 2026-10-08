import { action, defineWindows, flag, raw, selection, text, theme, toggle, value } from "../src/authoring/index.ts";
import type { Indexed, TextHandle } from "../src/authoring/index.ts";
import {
    Box,
    Button,
    Case,
    Container,
    Hud,
    Row,
    Show,
    Sprite,
    Switch,
    Tab,
    Tabs,
    Text,
    Window,
} from "../src/authoring/jsx.ts";

export default (
    <>
        <Window name="test" container="generic_9x1">
            <Container>
                <Tabs name="kind">
                    <>
                        <Tab value="all">
                            <Text>All</Text>
                        </Tab>
                        {false && <Tab value="hidden">Hidden</Tab>}
                    </>
                </Tabs>
            </Container>
        </Window>
        <Hud name="status">
            <Text bind="value" width={40} />
            <Switch bind="mode" grow>
                <Case value="buy" frame="recess">
                    <Row>
                        <Text>Buy</Text>
                        <Text bind="price" />
                    </Row>
                </Case>
                <Case value="sell" justify="center" />
            </Switch>
            <Show when="on_sale" align="end">
                <Sprite name="sale_badge" />
            </Show>
        </Hud>
    </>
);

// @ts-expect-error Padding does not support intrinsic sizing keywords.
const invalidPadding = <Box padding="min-content" />;
// @ts-expect-error Insets accept pixel/percentage lengths or auto.
const invalidInset = <Text top="max-content">Hi</Text>;
// @ts-expect-error Track sizing does not accept the dimension keyword stretch.
const invalidTrack = <Box columns={["stretch"]} />;
// @ts-expect-error HUD canvas dimensions are pixel numbers.
const invalidHud = <Hud name="invalid" width="100%" height="100%" />;
// @ts-expect-error A case is a box placed by its switch, so it takes no item layout.
const invalidCase = <Case value="buy" grow />;
// @ts-expect-error Show needs the Boolean binding it reads.
const invalidShow = <Show />;
void [invalidPadding, invalidInset, invalidTrack, invalidHud, invalidCase, invalidShow];

const category = selection("category", ["all", "gear", "magic"], { initial: "gear" });
const mode = value("mode", ["buy", "sell"]);
const lamp = flag("lamp", { shape: [2, 3] });
const handles = (
    <Window name="handles" container="generic_9x1">
        <Switch on={mode}>{{ buy: <Text>Buy</Text>, sell: null }}</Switch>
        <Show when={category.is("gear")} />
        <Show when={lamp.at(1, 2)} />
        <Button onClick={category.set("magic")} state={mode} states={{ buy: {}, sell: {} }} />
    </Window>
);
// @ts-expect-error `is` takes one of the selection's values.
const isTypo = category.is("gera");
// @ts-expect-error `set` takes one of the selection's values.
const setTypo = category.set("gera");
// @ts-expect-error `initial` is one of the values.
const initialTypo = selection("sort", ["price", "name"], { initial: "nmae" });
// @ts-expect-error A switch on a handle covers every value.
const partialSwitch = <Switch on={mode}>{{ buy: null }}</Switch>;
// @ts-expect-error An indexed handle is read through `.at()`.
const unindexed = <Show when={lamp} />;
// @ts-expect-error `.at()` takes one index per dimension.
const wrongRank = lamp.at(1);
// @ts-expect-error Button states are keyed by the state handle's values.
const wrongStates = <Button onClick={action("trade")} state={mode} states={{ buy: {}, rent: {} }} />;
void [handles, isTypo, setTypo, initialTypo, partialSwitch, unindexed, wrongRank, wrongStates];

const favorites = toggle("favorites");
const labels = text("label", { shape: [3] });
const label: Indexed<TextHandle, [number]> = labels;
const booleanSwitches = (
    <Window name="booleans" container="generic_9x1">
        <Switch on={favorites}>{{ true: <Text>On</Text>, false: null }}</Switch>
        <Switch on={lamp.at(0, 0)}>{{ true: null, false: <Text>Off</Text> }}</Switch>
        <Tabs bind={category} tooltip={(v) => ({ all: "All", gear: "Gear", magic: "Magic" })[v]} itemModel={(v) => v}>
            {(_, i) => <Text bind={label.at(i)} />}
        </Tabs>
    </Window>
);
const statusHuds = [<Hud name="a" />, <Hud name="b" />];
const definition = defineWindows({ windows: [handles, booleanSwitches], huds: [statusHuds, <Hud name="c" />] });
// @ts-expect-error A switch on a flag or toggle covers `true` and `false`.
const partialFlagSwitch = <Switch on={favorites}>{{ true: null }}</Switch>;
// @ts-expect-error A tab tooltip is called with the selection's values.
const wrongTooltip = Tabs({ bind: category, tooltip: (v: "all" | "rare") => v, children: () => null });
// @ts-expect-error A theme is not a window.
const themeAsWindow = defineWindows({ windows: [theme({})] });
// @ts-expect-error A window document is not a HUD.
const windowAsHud = defineWindows({ huds: [handles, { windows: [] }] });
void [definition, partialFlagSwitch, wrongTooltip, themeAsWindow, windowAsHud];

const rawLabel = raw.label(raw.text.smallCaps("Shop"));
// @ts-expect-error Text helpers live on `raw.text`, not the `text` handle constructor.
const handleSmallCaps = text.smallCaps("Shop");
// @ts-expect-error `toggle` declares a handle; `raw.toggle` builds the button.
const toggleButton = toggle("lamp", { width: 16, height: 16, states: { on: {}, off: {} } });
void [rawLabel, handleSmallCaps, toggleButton];

// @ts-expect-error A comparison is a condition, not the value a switch reads.
const comparedSwitch = <Switch on={mode.is("buy")}>{{ buy: null, sell: null }}</Switch>;
// @ts-expect-error A setter is a click, not the value a button shows.
const setterState = (
    <Button onClick={action("trade")} state={category.set("all")} states={{ all: {}, gear: {}, magic: {} }} />
);
// @ts-expect-error A setter is not a condition.
const setterCondition = <Show when={category.set("all")} />;
// @ts-expect-error A comparison is not a click.
const comparedClick = <Button onClick={category.is("all")} />;
void [comparedSwitch, setterState, setterCondition, comparedClick];
