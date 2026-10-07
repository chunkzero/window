import { AnvilInput, Box, Button, Sprite, Text, Window, pattern, slotRects } from "#plugins/window";

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
        <Box frame="shell" x={-4} y={-1} width={184} height={80} />
        <Box frame="recess" x={30} y={3} width={116} height={12} />
        <Box frame="panel" x={4} y={15} width={168} height={54} />
        <Box frame="hazard_bar" x={-4} y={73} width={184} height={6} />
        {rivets.map(([x, y]) => (
            <Sprite name="rivet" x={x} y={y} />
        ))}
        <Text x={0} y={6} width={176} align="center">
            Catalog Search
        </Text>
        <Text x={10} y={24} width={44} color="#bceeff">
            Name
        </Text>
        <AnvilInput bind={query} initial="" />
        <Button
            onClick={back}
            frame="button"
            pattern={pattern.rect({ section: "container", x: 0, y: 0, width: 1, height: 1 })}
        >
            <Sprite name="icon_back" />
        </Button>
        <Button
            onClick={confirm}
            frame="button_accent"
            tooltip="Search"
            pattern={pattern.rect({ section: "container", x: 2, y: 0, width: 1, height: 1 })}
        >
            <Sprite name="icon_check" translate={[-1, 0]} />
        </Button>
        {slotRects("inventory", {
            pattern: pattern.rect({ section: "player", x: 0, y: 0, width: 9, height: 3 }),
            claim: "all",
        })}
        {slotRects("hotbar", {
            pattern: pattern.rect({ section: "hotbar", x: 0, y: 0, width: 9, height: 1 }),
            claim: "all",
        })}
    </Window>
);
