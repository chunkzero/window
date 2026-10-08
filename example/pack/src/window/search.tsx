import { Box, Image, Input, Text, Window, industrial } from "#plugins/window";

import { icons } from "./art.ts";
import { back, confirm, query } from "./handles.ts";

const rivets: [number, number][] = [
    [-2, 6],
    [173, 6],
    [-2, 68],
    [173, 68],
];

/**
 * A static anvil: nothing on it changes after open, so typing never reopens the screen. Vanilla's anvil art is hidden,
 * and the player's inventory slots are claimed so the screen ends at the hazard bar.
 */
export default (
    <Window
        name="catalog_search"
        container="anvil"
        bleed={{ top: 1, right: 4, bottom: 0, left: 4 }}
        text={{ color: "#ffffff", shadow: true, smallCaps: true }}
    >
        <Box frame={industrial.art.shell} x={-4} y={-1} width={184} height={80} />
        <Box frame={industrial.art.recess} x={30} y={3} width={116} height={12} />
        <Box frame={industrial.art.panel} x={4} y={15} width={168} height={54} />
        <Box frame={industrial.art.hazardBar} x={-4} y={73} width={184} height={6} />
        {rivets.map(([x, y]) => (
            <Image art={industrial.art.rivet} x={x} y={y} />
        ))}
        <Text x={0} y={6} width={176} align="center">
            Catalog Search
        </Text>
        <Text x={10} y={24} width={44} color="#bceeff">
            Name
        </Text>
        <Input bind={query} initial="" />
        <industrial.Container claim="none">
            <industrial.Button onClick={back} at={[0, 0]}>
                <Image art={icons.back} />
            </industrial.Button>
            <industrial.Button onClick={confirm} at={[2, 0]} frame={industrial.art.buttonAccent} tooltip="Search">
                <Image art={icons.check} translate={[-1, 0]} />
            </industrial.Button>
        </industrial.Container>
        <industrial.Player claim="all" />
        <industrial.Hotbar claim="all" />
    </Window>
);
