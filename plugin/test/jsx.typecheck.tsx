import { Box, Case, Container, Hud, Row, Show, Sprite, Switch, Tab, Tabs, Text, Window } from "../src/authoring/jsx.ts";

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
            <Switch bind="mode" grow>
                <Case value="buy" frame="recess">
                    <Row>
                        <Text>Buy</Text>
                        <Text bind="price" />
                    </Row>
                </Case>
                <Case value="sell" justify="center" />
            </Switch>
            <Show when="on_sale" align="end">
                <Sprite name="sale_badge" />
            </Show>
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
// @ts-expect-error A case is a box placed by its switch, so it takes no item layout.
const invalidCase = <Case value="buy" grow />;
// @ts-expect-error Show needs the Boolean binding it reads.
const invalidShow = <Show />;
void [invalidPadding, invalidInset, invalidTrack, invalidHud, invalidCase, invalidShow];
