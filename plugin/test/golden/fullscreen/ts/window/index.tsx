import { Box, Case, Fullscreen, Region, Switch, Text, defineWindows, shape } from "plugin:window/ui";
import { action, builtin, flag, text, tint } from "plugin:window/bind";
import * as raw from "plugin:window/raw";
import { Repeater } from "plugin:window/theme/industrial";

const panel = shape({
    kind: "panel",
    fill: "#1a1f27",
    border_color: "#4a5568",
    border_width: 2,
    radius: 0,
    inset_depth: 0,
});
const button = shape({
    kind: "panel",
    fill: "#2f4c78",
    border_color: "#7fa6e0",
    border_width: 1,
    radius: 2,
    inset_depth: 0,
});
const tone = tint("tone", { ok: "#3ddc84", low: "#ff4d4d" });
const pick = action("pick", { shape: [3] });
const slotName = text("slot_name", { shape: [3] });

// An unlocked full-screen menu over a backdrop, with buttons, bound and tinted text, a switch, and a repeater whose
// cells and play button have tooltips; and a locked view.
const menu = (
    <Fullscreen name="menu" backdrop="#0b0e13" align="center" justify="center">
        <Box frame={panel} width={240} height={180} direction="column" padding={8} gap={6} align="center">
            <Text color="#f2e9d8">Main menu</Text>
            <Text bind={text("count")} width={120} align="center" color={tone} />
            <Box frame={button} width={64} height={24} align="center" justify="center">
                <Region onClick={action("play")} tooltip={{ title: "Play", lines: ["Starts a round"] }} />
                <Text color="#ffffff">Play</Text>
            </Box>
            <Box frame={button} width={64} height={24} align="center" justify="center">
                <Region onClick={builtin("window:close")} hover={false} />
                <Text color="#ffffff">Quit</Text>
            </Box>
            <Repeater cell={[2, 1]} columns={3} rows={1} onClick={pick} tooltip={(i) => `Save ${i + 1}`}>
                {(i) => <Text bind={slotName.at(i)} width={30} align="center" />}
            </Repeater>
            <Switch bind={flag("ready")}>
                <Case value="true">
                    <Text color="#3ddc84">Ready</Text>
                </Case>
                <Case value="false" />
            </Switch>
        </Box>
    </Fullscreen>
);

const lookout = raw.fullscreen({
    name: "lookout",
    mode: "horse",
    children: [raw.text("Lookout", { x: 8, y: 8, color: "#ffffff" })],
});

export default defineWindows({ fullscreens: [menu, lookout] });
