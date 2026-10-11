import { Bar, Box, Hud, Switch, Text, Window, shape } from "plugin:window/ui";
import { flag, fraction, tint } from "plugin:window/bind";

const tone = tint("tone", { ok: "#3ddc84", warn: "#ffb020", danger: "#ff4d4d" });
const charge = fraction("charge", { shape: [2] });

const track = shape({ kind: "panel", fill: "#20242c", border_color: "#000000", border_width: 1, radius: 0 });
/** White art, so a tint shows as its own color. */
const fill = shape({ kind: "panel", fill: "#ffffff", border_width: 0, radius: 0, inset_depth: 0 });
/** Art that differs along its width, so each clip is its own glyph. */
const capsule = shape({ kind: "panel", fill: "#ffffff", border_color: "#b0b0b0", border_width: 1, radius: 3 });

// Bars in a window and in a shader-placed HUD: plain, tinted by a handle or a fixed color, and in a switch case.
export default (
    <>
        <Window name="furnace" container="generic_9x3">
            <Box x={8} y={18} gap={2}>
                <Bar bind={fraction("progress")} width={100} height={6} fill={fill} track={track} tint={tone} />
                <Bar bind={fraction("fuel")} width={24} height={5} fill={capsule} tint="#ff8040" />
                <Switch on={flag("lit")}>
                    {{ true: <Bar bind={charge.at(1)} width={9} height={2} fill={fill} />, false: null }}
                </Switch>
            </Box>
        </Window>

        <Hud name="vitals" anchor="bottom" offset={[0, -60]} gap={2}>
            <Text>Health</Text>
            <Bar bind={fraction("health")} width={81} height={5} fill={fill} track={track} tint={tone} />
            <Bar bind={fraction("stamina")} width={81} height={3} fill={capsule} />
        </Hud>
    </>
);
