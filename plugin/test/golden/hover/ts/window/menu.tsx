import { Hud, Region, Section, Text, Window } from "plugin:window/ui";
import { action, text } from "plugin:window/bind";
import { Button } from "plugin:window/theme/industrial";

// Outlined and plain regions beside a shader-placed HUD, so both core shader features share the text shaders.
export default (
    <>
        <Window name="menu" container="generic_9x3">
            <Section of="container">
                <Button onClick={action("buy")} span={3} tooltip="Buy">
                    Buy
                </Button>
                <Button onClick={action("sell")} span={3}>
                    Sell
                </Button>
                <Button onClick={action("quiet")} span={3} tooltip="Quiet" hover={false}>
                    Quiet
                </Button>
                <Region tooltip="Info" span={2} />
                <Region tooltip="Help" span={2} hover />
            </Section>
        </Window>

        <Hud name="wave" anchor="bottom" offset={[0, -60]}>
            <Text bind={text("time")} width={50} align="center" />
        </Hud>
    </>
);
