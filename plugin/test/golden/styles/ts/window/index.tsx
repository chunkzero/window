import {
    Box,
    Image,
    Region,
    Section,
    Text,
    Window,
    create,
    createTheme,
    defineVars,
    defineWindows,
    derive,
    mix,
    shape,
    variants,
} from "plugin:window/ui";
import type { Color, Var } from "plugin:window/ui";
import { action } from "plugin:window/bind";

const colors = defineVars({ face: "#0994c6", text: "#ffffff" });
const ember = createTheme(colors, { face: "#c0503a" });
const moss = createTheme(colors, { face: "#3a8c40", text: "#eeeeaa" });

const raised = (fill: Var<Color>) =>
    derive((get) => {
        const base = get(fill);
        return shape({
            kind: "button",
            fill: base,
            border_color: "#03091f",
            border_width: 1,
            inset_depth: 1,
            highlight_color: mix(base, "#5ff0ff", 0.45),
            shadow_color: mix(base, "#020a30", 0.4),
        });
    });

const s = create({
    tab: variants({
        base: { frame: raised(colors.face), span: 3, padding: 2 },
        selected: { translate: [0, -2] },
    }),
    label: { color: colors.text, shadow: true },
});

const lamp = shape({
    kind: "button",
    fill: colors.text,
    border_color: colors.face,
    border_width: 1,
    width: 6,
    height: 6,
});

export default defineWindows({
    sprites: { lamp },
    windows: [
        <Window name="styled" container="generic_9x1" frame={raised(colors.face)}>
            <Section of="container">
                <Box style={s.tab.base}>
                    <Text style={s.label}>Base</Text>
                    <Region onClick={action("base")} />
                </Box>
                <Box style={[s.tab.base, s.tab.selected]} theme={ember}>
                    <Text style={s.label}>Ember</Text>
                    <Region onClick={action("ember")} />
                </Box>
                <Box style={s.tab.base} theme={moss}>
                    <Image art={lamp} />
                    <Text style={s.label}>Moss</Text>
                </Box>
            </Section>
        </Window>,
    ],
});
