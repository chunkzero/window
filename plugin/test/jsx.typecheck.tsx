import { containerLayout, defineWindows, shape, texture } from "../src/ui/index.ts";
import { action, builtin, flag, input, items, selection, sprite, text, toggle, value } from "../src/bind/index.ts";
import type { Indexed, TextHandle } from "../src/bind/index.ts";
import * as raw from "../src/raw/index.ts";
import { Prompt } from "../src/theme/industrial/index.ts";
import { Box, Case, Hud, Image, Items, Region, Section, Switch, Text, Window } from "../src/ui/components.ts";

const recess = shape({ kind: "slot", fill: "#102040" });
const price = text("price");

export default (
    <>
        <Window name="test" container="generic_9x1">
            <Section of="container">
                <>
                    <Text>All</Text>
                    {false && <Text>Hidden</Text>}
                </>
            </Section>
        </Window>
        <Hud name="status">
            <Text bind={text("value")} width={40} />
            <Switch bind={value("mode", ["buy", "sell"])} grow>
                <Case value="buy" frame={recess}>
                    <Box direction="row">
                        <Text>Buy</Text>
                        <Text bind={price} />
                    </Box>
                </Case>
                <Case value="sell" justify="center" />
            </Switch>
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
// @ts-expect-error Text binds a text handle, not a name.
const namedText = <Text bind="price" />;
// @ts-expect-error Frames take art values, not theme names.
const namedFrame = <Box frame="recess" />;
void [invalidPadding, invalidInset, invalidTrack, invalidHud, invalidCase, namedText, namedFrame];

const category = selection("category", ["all", "gear", "magic"], { initial: "gear" });
const mode = value("mode", ["buy", "sell"]);
const lamp = flag("lamp", { shape: [2, 3] });
const handles = (
    <Window name="handles" container="generic_9x1">
        <Switch on={mode}>{{ buy: <Text>Buy</Text>, sell: null }}</Switch>
        <Switch on={category}>{{ all: null, gear: <Text>Gear</Text>, magic: null }}</Switch>
        <Switch on={lamp.at(1, 2)}>{{ true: null, false: null }}</Switch>
        <Region onClick={category.set("magic")} />
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
const unindexed = <Switch on={lamp}>{{ true: null, false: null }}</Switch>;
// @ts-expect-error `.at()` takes one index per dimension.
const wrongRank = lamp.at(1);
void [handles, isTypo, setTypo, initialTypo, partialSwitch, unindexed, wrongRank];

const favorites = toggle("favorites");
const labels = text("label", { shape: [3] });
const label: Indexed<TextHandle, [number]> = labels;
const booleanSwitches = (
    <Window name="booleans" container="generic_9x1">
        <Switch on={favorites}>{{ true: <Text>On</Text>, false: null }}</Switch>
        <Switch on={lamp.at(0, 0)}>{{ true: null, false: <Text>Off</Text> }}</Switch>
        <Text bind={label.at(2)} />
        <Region onClick={builtin("window:close")} />
    </Window>
);
const statusHuds = [<Hud name="a" />, <Hud name="b" />];
const definition = defineWindows({ windows: [handles, booleanSwitches], huds: [statusHuds, <Hud name="c" />] });
// @ts-expect-error A switch on a flag or toggle covers `true` and `false`.
const partialFlagSwitch = <Switch on={favorites}>{{ true: null }}</Switch>;
// @ts-expect-error A sprite catalog is not a window.
const catalogAsWindow = defineWindows({ windows: [{ sprites: {} }] });
// @ts-expect-error defineWindows takes no themes.
const themes = defineWindows({ themes: [] });
// @ts-expect-error A window document is not a HUD.
const windowAsHud = defineWindows({ huds: [handles, { windows: [] }] });
void [definition, partialFlagSwitch, catalogAsWindow, themes, windowAsHud];

const rawLabel = raw.text(raw.text.smallCaps("Shop"));
// @ts-expect-error Text helpers live on `raw.text`, not the `text` handle constructor.
const handleSmallCaps = text.smallCaps("Shop");
// @ts-expect-error `toggle` declares a handle and takes only its initial value.
const toggleButton = toggle("lamp", { width: 16, height: 16 });
// @ts-expect-error raw keeps only primitives and roots.
const rawButton = raw.button("buy", {});
void [rawLabel, handleSmallCaps, toggleButton, rawButton];

// @ts-expect-error A comparison is a condition, not the value a switch reads.
const comparedSwitch = <Switch on={mode.is("buy")}>{{ buy: null, sell: null }}</Switch>;
// @ts-expect-error A setter is not a condition.
const setterCondition = Switch({ bind: category.set("all"), children: [] });
// @ts-expect-error A comparison is not a click.
const comparedClick = <Region onClick={category.is("all")} />;
void [comparedSwitch, setterCondition, comparedClick];

const coin = texture("window/coin.png");
const bevel = shape({ kind: "button", fill: "#3a3a3a" }, { name: "industrial/button" });
const icon = sprite("icon", { only: ["lamp_on", "lamp_off"] });
const primitives = (
    <Window name="primitives" container="generic_9x3" frame={bevel}>
        <Section of="container">
            <Box span={3} frame={bevel} debugName="tab-all">
                <Switch bind={category.is("all")}>
                    <Case value="true">
                        <Image art={coin} />
                    </Case>
                    <Case value="false" />
                </Switch>
                <Region onClick={category.set("all")} tooltip="All" />
            </Box>
            <Switch bind={mode} span={2}>
                <Case value="buy">
                    <Region onClick={action("buy")} />
                </Case>
                <Case value="sell">
                    <Items bind={items("stack")} />
                </Case>
            </Switch>
        </Section>
        <Image bind={icon} size={8} x={4} y={4} />
    </Window>
);
const catalog = defineWindows({ sprites: { lamp_on: coin, lamp_off: bevel }, windows: [primitives] });
// @ts-expect-error An image draws art or a sprite handle, not both.
const artAndBind = <Image art={coin} bind={icon} size={8} />;
// @ts-expect-error A bound image needs its slot size.
const unsizedBind = <Image bind={icon} />;
// @ts-expect-error `only` names the sprites a handle returns.
const iconOnly: readonly "lamp_on"[] | undefined = icon.only;
// @ts-expect-error A section names its grid.
const unknownSection = <Section of="armor" />;
// @ts-expect-error The catalog holds inline art, not theme names.
const namedCatalog = defineWindows({ sprites: { coin: "coin" } });
void [catalog, artAndBind, unsizedBind, iconOnly, unknownSection, namedCatalog];

const rawPrimitives = raw.box({
    frame: bevel,
    children: [
        raw.image(coin),
        raw.image(icon, { width: 8, height: 8 }),
        raw.region({ on_click: action("buy"), debug_name: "buy" }),
        raw.text("Label"),
        raw.text(text("title")),
        raw.switchOn(mode, [raw.case("buy", { children: [raw.items(items("stack"))] }), raw.case("sell")]),
        raw.section("hotbar", { children: [raw.input(input("query"))] }),
    ],
});
void rawPrimitives;

containerLayout("anvil").input satisfies { width: number };
containerLayout("generic_9x3").sections.container.rows satisfies 3;
// @ts-expect-error Only an anvil has a native input field.
void containerLayout("generic_9x3").input;

// @ts-expect-error A prompt needs `onConfirm`.
const unconfirmed = <Prompt name="prompt" bind={input("query")} />;
void unconfirmed;
