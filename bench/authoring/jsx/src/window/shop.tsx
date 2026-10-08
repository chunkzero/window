import { Box, Image, Text, Window, derive, texture } from "plugin:window/ui";
import { action, collection, flag, selection, sprite, text, toggle } from "plugin:window/bind";
import type { ToggleHandle } from "plugin:window/bind";
import {
    Button,
    Collection,
    Container,
    Header,
    Hotbar,
    Player,
    Row,
    Show,
    Tab,
    Tabs,
    Toggle,
    art,
} from "plugin:window/theme/industrial";

import { coin } from "./sprites.ts";

const ventSlot = derive((get) => ({
    ...get(art.recess),
    border_width: 0,
    inset_depth: 0,
    width: 6,
    height: 2,
}));

const category = selection("category", ["all", "gear", "magic"]);
const sort = selection("sort", ["featured", "price", "name"]);
const products = collection("products", { selectable: true });
const hasPrice = flag("has_price");

/** A button that is disabled while `enabled` is false. */
function Action(props: { id: string; tooltip: string; span?: number; children?: unknown; accent?: boolean }) {
    return (
        <Button
            onClick={action(props.id)}
            enabled={flag(`can_${props.id}`)}
            span={props.span ?? 1}
            tooltip={props.tooltip}
            disabled={{ tooltip: props.tooltip + " unavailable" }}
            {...(props.accent === true ? { frame: art.buttonAccent } : {})}
        >
            {props.children as never}
        </Button>
    );
}

/** A raised toggle with a lamp at its left and its label centered in the remaining width. */
function LampToggle(props: { bind: ToggleHandle; title: string; tooltip: string }) {
    return (
        <Toggle
            bind={props.bind}
            span={3}
            on={{ tooltip: props.tooltip + " on" }}
            off={{ tooltip: props.tooltip + " off" }}
        >
            <Image bind={sprite(props.bind.id + "_lamp", { only: ["lamp_on", "lamp_off"] })} size={4} />
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
        frame={art.shell}
        bleed={{ top: 1, right: 4, bottom: 7, left: 4 }}
        text={{ color: "#ffffff", shadow: true, smallCaps: true }}
    >
        <Header padding={{ top: 3 }} align="start">
            <Box frame={art.recess} width={116} height={12} justify="center" align="center">
                <Text>Foundry Exchange</Text>
            </Box>
        </Header>

        <Box frame={art.panel} x={4} y={136} width={168} height={82} />
        <Box frame={art.hazardBar} x={-4} y={223} width={184} height={6} />
        <Row x={52} y={131} gap={4}>
            {Array.from({ length: 8 }, () => (
                <Image art={ventSlot} />
            ))}
        </Row>
        {rivets.map(([x, y]) => (
            <Image art={art.rivet} x={x} y={y} />
        ))}

        <Container frame={art.panel} outset={{ top: 2, right: 3, bottom: 3, left: 3 }}>
            <Tabs bind={category} span={3}>
                <Tab value="all" tooltip="All items">
                    <Text bind={text("category_all_label")} />
                </Tab>
                <Tab value="gear" tooltip="Gear">
                    <Text bind={text("category_gear_label")} />
                </Tab>
                <Tab value="magic" tooltip="Magic">
                    <Text bind={text("category_magic_label")} />
                </Tab>
            </Tabs>

            <Collection bind={products} rows={3} />

            <Action id="previous" span={3} tooltip="Previous page">
                <Text bind={text("previous_label")} />
            </Action>
            <Row span={3} frame={art.recess}>
                <Text bind={text("page")} align="center" />
            </Row>
            <Action id="next" span={3} tooltip="Next page">
                <Text bind={text("next_label")} />
            </Action>

            <Row span={9} frame={art.recess} padding={{ left: 5, right: 4 }} gap={4}>
                <Text bind={text("selection")} color="#ffb20b" />
                <Text bind={text("price")} width={40} align="right" />
                <Show when={hasPrice}>
                    <Image art={coin} />
                </Show>
            </Row>
        </Container>

        <Player>
            <Tabs bind={sort} span={3}>
                <Tab value="featured" tooltip="Top picks first">
                    <Text bind={text("sort_featured_label")} />
                </Tab>
                <Tab value="price" tooltip="Price sort">
                    <Text bind={text("sort_price_label")} />
                </Tab>
                <Tab value="name" tooltip="Name sort">
                    <Text bind={text("sort_name_label")} />
                </Tab>
            </Tabs>

            <LampToggle bind={toggle("favorites")} title="Favs" tooltip="Favorites only" />
            <LampToggle bind={toggle("affordable")} title="Afford" tooltip="Affordable only" />
            <Button onClick={action("search")} span={2} tooltip="Search the catalog">
                Find
            </Button>
            <Action id="clear_search" tooltip="Clear search">
                <Image art={texture("window/icons/clear.png")} />
            </Action>

            <Row span={9} frame={art.recess} padding={{ left: 5, right: 4 }} gap={4}>
                <Text bind={text("status")} color="#bceeff" />
                <Text bind={text("balance")} width={48} align="right" color="#ffb20b" />
                <Image art={coin} />
            </Row>
        </Player>

        <Hotbar>
            <Action id="buy" span={6} accent tooltip="Buy selected item">
                <Text bind={text("buy_label")} color="#2a1200" shadow={false} />
            </Action>
            <Button close span={3} tooltip="Close market">
                Exit
            </Button>
        </Hotbar>
    </Window>
);
