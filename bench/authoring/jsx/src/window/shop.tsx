import {
    Box,
    Button,
    Collection,
    Container,
    Header,
    Hotbar,
    Icon,
    Player,
    Row,
    Show,
    Sprite,
    Tab,
    Tabs,
    Text,
    Toggle,
    Window,
} from "#plugins/window";
import type { StateProps } from "#plugins/window";

function enabled(tooltip: string, sprite: string): Record<string, StateProps> {
    return {
        enabled: { tooltip, sprite },
        disabled: { tooltip: tooltip + " unavailable", sprite: sprite + "_disabled" },
    };
}

/** A raised toggle with a lamp at its left and its label centered in the remaining width. */
function LampToggle(props: { name: string; title: string; tooltip: string }) {
    return (
        <Toggle
            name={props.name}
            span={3}
            frame="button"
            on={{ tooltip: props.tooltip + " on" }}
            off={{ tooltip: props.tooltip + " off" }}
        >
            <Icon bind={props.name + "_lamp"} size={4} />
            <Text>{props.title}</Text>
        </Toggle>
    );
}

const rivets: [number, number][] = [
    [-2, 6],
    [173, 6],
    [-2, 129],
    [173, 129],
];

export default (
    <Window
        name="shop"
        container="generic_9x6"
        frame="shell"
        bleed={{ top: 1, right: 4, bottom: 7, left: 4 }}
        text={{ color: "#ffffff", shadow: true, smallCaps: true }}
    >
        <Header padding={{ top: 3 }} align="start">
            <Box frame="recess" width={116} height={12} justify="center" align="center">
                <Text>Foundry Exchange</Text>
            </Box>
        </Header>

        <Box frame="panel" x={4} y={136} width={168} height={82} />
        <Box frame="hazard_bar" x={-4} y={223} width={184} height={6} />
        <Row x={52} y={131} gap={4}>
            {Array.from({ length: 8 }, () => (
                <Sprite name="vent_slot" />
            ))}
        </Row>
        {rivets.map(([x, y]) => (
            <Sprite name="rivet" x={x} y={y} />
        ))}

        <Container frame="panel" outset={{ top: 2, right: 3, bottom: 3, left: 3 }}>
            <Tabs name="category" span={3} sprite="tab" selectedSprite="tab_selected">
                <Tab value="all" tooltip="All items">
                    <Text bind="category_all_label" />
                </Tab>
                <Tab value="gear" tooltip="Gear">
                    <Text bind="category_gear_label" />
                </Tab>
                <Tab value="magic" tooltip="Magic">
                    <Text bind="category_magic_label" />
                </Tab>
            </Tabs>

            <Collection name="products" rows={3} frame="slot" selected="slot_selected" />

            <Button name="previous" span={3} states={enabled("Previous page", "action")}>
                <Text bind="previous_label" />
            </Button>
            <Row span={3} frame="recess">
                <Text bind="page" align="center" />
            </Row>
            <Button name="next" span={3} states={enabled("Next page", "action")}>
                <Text bind="next_label" />
            </Button>

            <Row span={9} frame="recess" padding={{ left: 5, right: 4 }} gap={4}>
                <Text bind="selection" color="#ffb20b" />
                <Text bind="price" width={40} align="right" />
                <Show when="has_price">
                    <Sprite name="coin" />
                </Show>
            </Row>
        </Container>

        <Player>
            <Tabs name="sort" span={3} sprite="tab" selectedSprite="tab_selected">
                <Tab value="featured" tooltip="Top picks first">
                    <Text bind="sort_featured_label" />
                </Tab>
                <Tab value="price" tooltip="Price sort">
                    <Text bind="sort_price_label" />
                </Tab>
                <Tab value="name" tooltip="Name sort">
                    <Text bind="sort_name_label" />
                </Tab>
            </Tabs>

            <LampToggle name="favorites" title="Favs" tooltip="Favorites only" />
            <LampToggle name="affordable" title="Afford" tooltip="Affordable only" />
            <Button name="search" span={2} frame="button" tooltip="Search the catalog">
                Find
            </Button>
            <Button name="clear_search" states={enabled("Clear search", "clear_search")}>
                <Icon bind="clear_search_icon" size={[8, 6]} sprite="icon_clear" />
            </Button>

            <Row span={9} frame="recess" padding={{ left: 5, right: 4 }} gap={4}>
                <Text bind="status" color="#bceeff" />
                <Text bind="balance" width={48} align="right" color="#ffb20b" />
                <Sprite name="coin" />
            </Row>
        </Player>

        <Hotbar>
            <Button name="buy" span={6} states={enabled("Buy selected item", "buy")}>
                <Text bind="buy_label" color="#2a1200" shadow={false} />
            </Button>
            <Button name="exit" span={3} frame="button" close tooltip="Close market">
                Exit
            </Button>
        </Hotbar>
    </Window>
);
