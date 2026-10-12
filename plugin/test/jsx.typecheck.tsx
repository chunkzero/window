import { containerLayout, defineWindows, preview, shape, texture } from "../src/ui/index.ts";
import {
    action,
    builtin,
    collection,
    flag,
    fraction,
    input,
    items,
    selection,
    sprite,
    text,
    tint,
    toggle,
    value,
} from "../src/bind/index.ts";
import type { Indexed, TextHandle } from "../src/bind/index.ts";
import * as raw from "../src/raw/index.ts";
import { Button, Prompt } from "../src/theme/industrial/index.ts";
import { Bar, Box, Case, Hud, Image, Items, Region, Section, Switch, Text, Window } from "../src/ui/components.ts";

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
const quietRegion = <Region onClick={category.set("all")} hover={false} />;
const quietButton = <Button onClick={action("buy")} hover={false} />;
const artRegion = <Region onClick={category.set("all")} hover={texture("ui/hover.png", { insets: 2 })} />;
const artButton = <Button onClick={action("buy")} hover={shape({ kind: "panel", fill: "#ff0000" })} />;
const fitButton = (
    <Button onClick={action("buy")} hover={{ art: texture("ui/hover.png"), fit: "center", tint: "#ffcc00" }} />
);
// @ts-expect-error `hover` is a boolean or art, not a color.
const styledHover = <Region onClick={category.set("all")} hover="#ffffff" />;
const hoverTone = tint("hover_tone", { ok: "#3ddc84" });
// @ts-expect-error A hover's tint is baked in, so it is a color and not a `tint` handle.
const handleHover = <Region onClick={category.set("all")} hover={{ art: texture("ui/hover.png"), tint: hoverTone }} />;
// @ts-expect-error The object form names its art.
const artlessHover = <Region onClick={category.set("all")} hover={{ tint: "#ffcc00" }} />;
void [comparedSwitch, setterCondition, comparedClick, quietRegion, quietButton, artRegion, artButton, fitButton];
void [styledHover, handleHover, artlessHover];

const tone = tint("tone", { ok: "#3ddc84", warn: "#ffb020" });
const tinted = (
    <Box frame={shape({ kind: "panel" })} tint={tone}>
        <Text color={tone}>Wave</Text>
        <Image art={texture("window/coin.png")} tint="#ffaa00" />
    </Box>
);
// @ts-expect-error A tint is a tint handle or a color, not another handle.
const valueTint = <Box tint={mode} />;
// @ts-expect-error A text color is a color or a tint handle.
const flagColor = <Text color={favorites}>Wave</Text>;
// @ts-expect-error A bound image takes a tint handle, not a fixed color.
const fixedSlotTint = <Image bind={sprite("lamp_icon")} size={8} tint="#ffaa00" />;
// @ts-expect-error A HUD's own frame is not tinted.
const tintedHud = <Hud name="tinted" tint={tone} />;
void [tinted, valueTint, flagColor, fixedSlotTint, tintedHud];

const health = fraction("health");
const charges = fraction("charge", { shape: [3] });
const bars = (
    <Box>
        <Bar bind={health} width={80} height={6} fill={shape({ kind: "panel" })} tint={tone} grow />
        <Bar
            bind={charges.at(1)}
            width={20}
            height={4}
            fill={texture("window/coin.png")}
            track={recess}
            tint="#ff5555"
        />
        {raw.bar(health, { width: 80, height: 6, fill: recess })}
    </Box>
);
const barPreview = preview(handles, [
    [health, 0.42],
    [charges, [0, 0.5, 1]],
]);
// @ts-expect-error A bar binds a fraction handle.
const flagBar = <Bar bind={favorites} width={80} height={6} fill={recess} />;
// @ts-expect-error A bar needs its fill art.
const emptyBar = <Bar bind={health} width={80} height={6} />;
// @ts-expect-error A bar's size is given in pixels.
const unsizedBar = <Bar bind={health} fill={recess} />;
// @ts-expect-error A bar's tint is a color or a tint handle.
const valueTintBar = <Bar bind={health} width={80} height={6} fill={recess} tint={mode} />;
// @ts-expect-error An indexed fraction is read through `.at()`.
const unindexedBar = <Bar bind={charges} width={80} height={6} fill={recess} />;
// @ts-expect-error A fraction takes a number.
const namedFraction = preview(handles, [[health, "half"]]);
void [bars, barPreview, flagBar, emptyBar, unsizedBar, valueTintBar, unindexedBar, namedFraction];

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

const lamps = flag("lamps", { shape: [2, 3] });
const badge = sprite("badge", { only: ["lamp_on", "lamp_off"] });
const previewed = preview(handles, [
    [category, "magic"],
    [price, "1,250"],
    [lamp.at(1, 2), true],
    [lamps, [[true, false], [true]]],
    [badge, "lamp_on"],
    [collection("products"), { count: 4, selected: 0 }],
    [input("query"), "amethyst"],
    [tone, "warn"],
]);
// @ts-expect-error A selection takes one of its values.
const unknownCase = preview(handles, [[category, "tools"]]);
// @ts-expect-error Text takes a string.
const numericText = preview(handles, [[price, 1250]]);
// @ts-expect-error An indexed handle takes a list, or `.at()` names one member.
const unindexed = preview(handles, [[lamps, true]]);
// @ts-expect-error A narrowed sprite takes one of its `only` names.
const unknownSprite = preview(handles, [[badge, "coin"]]);
// @ts-expect-error An action has no state.
const actionState = preview(handles, [[action("buy"), true]]);
// @ts-expect-error A tint takes one of its palette entry names.
const unknownEntry = preview(handles, [[tone, "danger"]]);
void [previewed, unknownCase, numericText, unindexed, unknownSprite, actionState, unknownEntry];
