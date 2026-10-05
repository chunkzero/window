/**
 * @jsxRuntime classic
 * @jsx h
 * @jsxFrag Fragment
 */
import { Box, Container, Fragment, Hud, Tab, Tabs, Text, Window, h } from "../src/authoring/jsx.ts";

export default (
    <>
        <Window name="test" container="generic_9x1">
            <Container>
                <Tabs name="kind">
                    <>
                        <Tab value="all">
                            <Text>All</Text>
                        </Tab>
                        {false && <Tab value="hidden">Hidden</Tab>}
                    </>
                </Tabs>
            </Container>
        </Window>
        <Hud name="status">
            <Text bind="value" width={40} />
        </Hud>
    </>
);

// @ts-expect-error Padding does not support intrinsic sizing keywords.
const invalidPadding = <Box padding="min-content" />;
// @ts-expect-error Insets accept pixel/percentage lengths or auto.
const invalidInset = <Text top="max-content">Hi</Text>;
// @ts-expect-error Track sizing does not accept the dimension keyword stretch.
const invalidTrack = <Box columns={["stretch"]} />;
// @ts-expect-error HUD canvas dimensions are pixel numbers.
const invalidHud = <Hud name="invalid" width="100%" height="100%" />;
void [invalidPadding, invalidInset, invalidTrack, invalidHud];
