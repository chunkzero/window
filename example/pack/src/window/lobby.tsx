import { Case, Fullscreen, Image, Region, Switch, Text, create } from "plugin:window/ui";
import { Button, Column, Repeater, Row, art } from "plugin:window/theme/industrial";

import { icons } from "./art.ts";
import {
    WORLDS,
    foundryName,
    lobbyStatus,
    nextWorld,
    pickWorld,
    play,
    previousWorld,
    ready,
    rename,
    tone,
    worldName,
} from "./handles.ts";

const s = create({
    title: { color: "#ffe2b8" },
    plate: { frame: art.recess, width: 160, height: 16, justify: "center", align: "center" },
    arrow: { width: 16, height: 18 },
    action: { width: 60, height: 18 },
});

/**
 * A main menu drawn over the whole screen while the world turns behind it. Click the name to rename the foundry in a
 * native dialog, pick a world from the paged list (the arrows or the scroll wheel move it), right click anywhere to
 * ready up, then play.
 */
export default (
    <Fullscreen name="lobby" backdrop="#140b08" align="center" justify="center" text={{ color: "#ffffff" }}>
        <Column frame={art.panel} width={232} padding={8} gap={6} align="center">
            <Text style={s.title}>Foundry Lobby</Text>
            <Row style={s.plate}>
                <Text bind={foundryName} width={150} align="center" color="#ffe2b8" />
                <Region onClick={rename} tooltip="Click to rename" />
            </Row>
            <Text bind={lobbyStatus} width={216} align="center" color={tone} />
            <Row gap={2} align="center">
                <Button onClick={previousWorld} style={{ base: s.arrow }}>
                    {"<"}
                </Button>
                <Repeater fixed cell={[3, 1]} columns={WORLDS} rows={1} onClick={pickWorld} tooltip="Click to select">
                    {(i) => <Text bind={worldName.at(i)} width={48} align="center" />}
                </Repeater>
                <Button onClick={nextWorld} style={{ base: s.arrow }}>
                    {">"}
                </Button>
            </Row>
            <Switch bind={ready} height={9}>
                <Case value="true" direction="row" gap={2} align="center">
                    <Image art={icons.check} />
                    <Text color="#3ddc84">Ready</Text>
                </Case>
                <Case value="false">
                    <Text color="#f2c9a0">Right click when ready</Text>
                </Case>
            </Switch>
            <Row gap={4}>
                <Button
                    onClick={play}
                    style={{ base: s.action }}
                    tooltip={{ title: "Play", lines: ["Joins the selected world once you are ready"] }}
                >
                    Play
                </Button>
                <Button close style={{ base: s.action }}>
                    Quit
                </Button>
            </Row>
        </Column>
    </Fullscreen>
);
