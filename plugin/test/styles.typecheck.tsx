import { create, createTheme, defineVars, derive, mix, raw, shape, variants } from "../src/authoring/index.ts";
import type { BoxStyle, Color, Var, Variants } from "../src/authoring/index.ts";
import {
    Box,
    Case,
    Collection,
    Hud,
    Image,
    Items,
    Region,
    Section,
    Switch,
    Text,
    Window,
} from "../src/authoring/jsx.ts";

const colors = defineVars({ face: "#0994c6", text: "#ffffff" });
const sizes = defineVars({ pad: 2 });
const face: Var<Color> = colors.face;
const pad: Var<number> = sizes.pad;
// @ts-expect-error A color var is not a number var.
const faceAsNumber: Var<number> = colors.face;
// @ts-expect-error Var defaults are "#rrggbb" colors or numbers.
const named = defineVars({ face: "blue" });
void [face, pad, faceAsNumber, named];

const ember = createTheme(colors, { face: "#c0503a" });
const roomy = createTheme(sizes, { pad: 4 });
// @ts-expect-error A color var takes a color.
const numberColor = createTheme(colors, { face: 3 });
// @ts-expect-error A number var takes a number.
const colorNumber = createTheme(sizes, { pad: "#ffffff" });
// @ts-expect-error A theme overrides only its vars.
const unknownVar = createTheme(colors, { border: "#000000" });
void [numberColor, colorNumber, unknownVar];

const raised = (fill: Var<Color>) =>
    derive((get) => {
        const base = get(fill);
        return shape({ kind: "button", fill: base, highlight_color: mix(base, "#ffffff", 0.45) });
    });

const s = create({
    tab: variants({
        base: { frame: shape({ kind: "button", fill: colors.face }), padding: sizes.pad },
        selected: { frame: raised(colors.face), translate: [0, -2] },
    }),
    wide: { width: 64, grow: true },
    label: { color: colors.text, shadow: true, align: "left" },
    centered: { align: "center" },
    item: { span: 2 },
});

const wide = Math.random() > 0.5;
const styled = (
    <Window name="styled" container="generic_9x3" theme={ember} style={{ text: { color: colors.text } }}>
        <Section of="container" style={{ frame: raised(colors.face) }}>
            <Box style={[s.tab.base, wide && s.wide, null]} theme={roomy}>
                <Text style={[s.label, s.centered]}>Buy</Text>
                <Region style={s.item} />
            </Box>
            <Box style={s.tab.selected} padding={sizes.pad} frame={raised(colors.face)} />
            <Switch bind="mode" style={s.item}>
                <Case value="on" style={s.centered} theme={ember} />
            </Switch>
            <Items name="stack" style={s.item} />
            <Collection name="list" style={{ frame: shape({ fill: colors.face }) }} />
        </Section>
        <Image art={raised(colors.face)} style={s.item} />
        <Text color={colors.text}>Hi</Text>
    </Window>
);
const themedHud = <Hud name="hud" theme={ember} style={{ padding: 2, frame: raised(colors.face) }} />;
void [styled, themedHud];

// @ts-expect-error A style with text properties is not a box style, even as a value from `create`.
const colorOnBox = <Box style={s.label} />;
// @ts-expect-error Text aligns left, center, or right; a box aligns its children.
const textAlignOnBox = <Box style={{ align: "left" }} />;
// @ts-expect-error Text takes no frame.
const frameOnText = <Text style={s.tab.base}>Hi</Text>;
// @ts-expect-error An image takes item properties only.
const paddingOnImage = <Image art={raised(colors.face)} style={{ padding: 2 }} />;
// @ts-expect-error A variants group is not a style; pass one of its variants.
const groupAsStyle = <Box style={s.tab} />;
// @ts-expect-error A number var is not a color.
const numberAsColor = <Text color={sizes.pad}>Hi</Text>;
// @ts-expect-error A color var is not padding.
const colorAsPadding = <Box padding={colors.face} />;
// @ts-expect-error Shape colors take color vars.
const numberFill = shape({ fill: sizes.pad });
void [colorOnBox, textAlignOnBox, frameOnText, paddingOnImage, groupAsStyle, numberAsColor, colorAsPadding, numberFill];

// @ts-expect-error Unknown style properties are errors.
const typo = create({ label: { colour: "#ffffff" } });
// @ts-expect-error Style values are typed.
const wrongValue = create({ label: { shadow: "yes" } });
// @ts-expect-error Variants hold styles.
const wrongVariant = variants({ base: { grow: "lots" } });
void [typo, wrongValue, wrongVariant];

interface TabProps {
    style?: Partial<Variants<"base" | "selected", BoxStyle>>;
}
function Tab(props: TabProps) {
    return <Box style={[props.style?.base, props.style?.selected]} />;
}
const tab = <Tab style={s.tab} />;
const partial = <Tab style={variants({ base: { padding: 1 } })} />;
const misspelled = variants({ base: { padding: 1 }, selectd: { padding: 2 } });
// @ts-expect-error A component takes only the variants it names.
const misspelledTab = <Tab style={misspelled} />;
const textVariants = variants({ base: { color: "#ffffff" } });
// @ts-expect-error The component's variants are box styles.
const textTab = <Tab style={textVariants} />;
// @ts-expect-error A plain style is not a variants group.
const styleAsGroup = <Tab style={s.label} />;
void [tab, partial, misspelledTab, textTab, styleAsGroup];

const rawStyled = raw.ui({
    name: "raw",
    container: "generic_9x1",
    theme: ember,
    style: { frame: raised(colors.face) },
    children: [
        raw.box({
            theme: roomy,
            style: [s.tab.base, { minWidth: 4 }],
            children: [raw.label("Hi", { style: s.label })],
        }),
        raw.region({ style: s.item }),
    ],
});
// @ts-expect-error A raw label takes text styles.
const rawLabelFrame = raw.label("Hi", { style: s.tab.base });
void [rawStyled, rawLabelFrame];

const storedText = { label: { text: { color: "#ffffff", colour: "#aaaaaa" } } };
// @ts-expect-error Nested text properties are checked, even on a stored object.
const nestedTypo = create(storedText);
const storedOverrides = { face: "#000000", typo: "#aaaaaa" } as const;
// @ts-expect-error A stored theme names only vars of its set.
const storedUnknownVar = createTheme(colors, storedOverrides);
const storedGood = { face: "#000000" } as const;
const storedTheme = createTheme(colors, storedGood);
const storedBox = create({ box: { padding: 1, color: "#ffffff" } }).box;
// @ts-expect-error A stored style with text properties is not a raw box style.
const rawStoredBox = raw.box({ style: storedBox });
// @ts-expect-error A raw case takes typed styles only.
const rawFlexStyleCase = raw.case("on", { style: { min_width: 4 } });
void [nestedTypo, storedUnknownVar, storedTheme, rawStoredBox, rawFlexStyleCase];
