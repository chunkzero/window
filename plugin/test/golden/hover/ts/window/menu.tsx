import { Hud, Region, Section, Text, Window, shape } from "plugin:window/ui";
import { action, text } from "plugin:window/bind";
import { Button } from "plugin:window/theme/industrial";

// Outlined, art-hovered and plain regions beside a shader-placed HUD, so both core shader features share the text shaders.
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
                {/* Art that stretches over its region, and art drawn at its own size around it. */}
                <Region onClick={action("lit")} span={2} hover={shape({ kind: "panel", fill: "#ffcc00", radius: 3 })} />
                <Region
                    onClick={action("ring")}
                    hover={shape({ kind: "panel", fill: "#ff5555", radius: 4, width: 20, height: 20 })}
                />
            </Section>
        </Window>

        <Hud name="wave" anchor="bottom" offset={[0, -60]}>
            <Text bind={text("time")} width={50} align="center" />
        </Hud>
    </>
);
