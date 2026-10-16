import { Bar, Fullscreen, Text } from "plugin:window/ui";
import { Button, Column, Row, art } from "plugin:window/theme/industrial";

import { barTrack, toneFill } from "./art.ts";
import { charge, signal } from "./handles.ts";

/** A horse-mode window: the view stays locked while the player aims the cursor, and charging fills the signal bar. */
export default (
    <Fullscreen
        name="lookout"
        mode="horse"
        backdrop="#10141a"
        align="center"
        justify="center"
        text={{ color: "#ffffff" }}
    >
        <Column frame={art.panel} width={160} padding={8} gap={6} align="center">
            <Text color="#ffe2b8">Lookout</Text>
            <Bar bind={signal} width={120} height={6} fill={toneFill} track={barTrack} tint="#ffb20b" />
            <Row gap={4}>
                <Button onClick={charge} width={60} height={18} tooltip="Charges the signal">
                    Charge
                </Button>
                <Button close width={60} height={18}>
                    Leave
                </Button>
            </Row>
        </Column>
    </Fullscreen>
);
