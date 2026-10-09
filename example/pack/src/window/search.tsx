import { Image, Input, Text } from "plugin:window/ui";
import { Button, Container, Window, art } from "plugin:window/theme/industrial";

import { icons } from "./art.ts";
import { back, confirm, query } from "./handles.ts";

/**
 * A static anvil: nothing on it changes after open, so typing never reopens the screen. Vanilla's anvil art is hidden,
 * and without the inventory the shell ends at the hazard bar below the anvil's slots.
 */
export default (
    <Window name="catalog_search" container="anvil" title="Catalog Search" inventory={false}>
        <Text x={10} y={24} width={44} color="#bceeff">
            Name
        </Text>
        <Input bind={query} initial="" />
        <Container claim="none">
            <Button onClick={back} at={[0, 0]}>
                <Image art={icons.back} />
            </Button>
            <Button onClick={confirm} at={[2, 0]} frame={art.buttonAccent} tooltip="Search">
                <Image art={icons.check} translate={[-1, 0]} />
            </Button>
        </Container>
    </Window>
);
