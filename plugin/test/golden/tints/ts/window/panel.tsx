import { Box, Case, Hud, Image, Switch, Text, Window } from "plugin:window/ui";
import { flag, sprite, text } from "plugin:window/bind";

import { dot, pill, plate, tone } from "./art.ts";

// Tinted art among untinted art and text, in a window and in a shader-placed HUD.
export default (
    <>
        <Window name="panel" container="generic_9x3" frame={plate}>
            <Box x={8} y={18} direction="row" gap={4} align="center">
                <Box frame={pill} tint="#4080ff" padding={3}>
                    <Text color="#ffffff">Fixed</Text>
                </Box>
                <Box frame={pill} tint={tone} padding={3}>
                    <Text bind={text("status")} width={30} color="#101010" />
                </Box>
                <Image art={dot} tint={tone} />
                <Image art={dot} />
                <Image bind={sprite("icon")} size={8} tint={tone} />
                <Text color={tone}>Level</Text>
                <Switch bind={flag("alert")}>
                    <Case value="true" frame={pill} tint={tone} padding={2}>
                        <Text color={tone}>!</Text>
                    </Case>
                    <Case value="false" />
                </Switch>
            </Box>
        </Window>

        <Hud name="wave" anchor="bottom" offset={[0, -60]} text={{ color: tone, shadow: true }}>
            <Box frame={pill} tint={tone} padding={3} direction="row" gap={3} align="center">
                <Image art={dot} />
                <Text bind={text("time")} width={50} align="center" />
            </Box>
        </Hud>
    </>
);
