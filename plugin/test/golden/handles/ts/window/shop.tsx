import { Box, Image, Switch, Text, Window } from "plugin:window/ui";
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
    Tabs,
    Toggle,
    art,
} from "plugin:window/theme/industrial";

import { coin, icons, ventSlot } from "./art.ts";
import {
    affordable,
    balance,
    buy,
    buyLabel,
    canBuy,
    canNext,
    canPrevious,
    category,
    categoryLabel,
    clearSearch,
    favorites,
    hasPrice,
    hasQuery,
    next,
    nextLabel,
    page,
    previous,
    previousLabel,
    price,
    products,
    search,
    selectedName,
    sort,
    sortLabel,
    status,
} from "./handles.ts";

const categoryTooltips = { all: "All items", gear: "Gear", magic: "Magic" };
const sortTooltips = { featured: "Top picks first", price: "Price sort", name: "Name sort" };

/** A raised toggle with a lamp at its left and its label centered in the remaining width. */
function LampToggle(props: { bind: ToggleHandle; title: string; tooltip: string }) {
    return (
        <Toggle
            bind={props.bind}
            span={3}
            on={{ tooltip: props.tooltip + " on" }}
            off={{ tooltip: props.tooltip + " off" }}
        >
            <Switch on={props.bind}>{{ true: <Image art={art.lampOn} />, false: <Image art={art.lampOff} /> }}</Switch>
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
            <Tabs bind={category} span={3} tooltip={(value) => categoryTooltips[value]}>
                {(_, i) => <Text bind={categoryLabel.at(i)} />}
            </Tabs>

            <Collection bind={products} rows={3} />

            <Button
                onClick={previous}
                enabled={canPrevious}
                span={3}
                tooltip="Previous page"
                disabled={{ tooltip: "Previous page unavailable" }}
            >
                <Text bind={previousLabel} />
            </Button>
            <Row span={3} frame={art.recess}>
                <Text bind={page} align="center" />
            </Row>
            <Button
                onClick={next}
                enabled={canNext}
                span={3}
                tooltip="Next page"
                disabled={{ tooltip: "Next page unavailable" }}
            >
                <Text bind={nextLabel} />
            </Button>

            <Row span={9} frame={art.recess} padding={{ left: 5, right: 4 }} gap={4}>
                <Text bind={selectedName} color="#ffb20b" />
                <Text bind={price} width={40} align="right" />
                <Show when={hasPrice}>
                    <Image art={coin} />
                </Show>
            </Row>
        </Container>

        <Player>
            <Tabs bind={sort} span={3} tooltip={(value) => sortTooltips[value]}>
                {(_, i) => <Text bind={sortLabel.at(i)} />}
            </Tabs>

            <LampToggle bind={favorites} title="Favs" tooltip="Favorites only" />
            <LampToggle bind={affordable} title="Afford" tooltip="Affordable only" />
            <Button onClick={search} span={2} tooltip="Search the catalog">
                Find
            </Button>
            <Button
                onClick={clearSearch}
                enabled={hasQuery}
                tooltip="Clear search"
                disabled={{ tooltip: "Clear search unavailable" }}
            >
                <Image art={icons.clear} />
            </Button>

            <Row span={9} frame={art.recess} padding={{ left: 5, right: 4 }} gap={4}>
                <Text bind={status} color="#bceeff" />
                <Text bind={balance} width={48} align="right" color="#ffb20b" />
                <Image art={coin} />
            </Row>
        </Player>

        <Hotbar>
            <Button
                onClick={buy}
                enabled={canBuy}
                span={6}
                frame={art.buttonAccent}
                tooltip="Buy selected item"
                disabled={{ tooltip: "Buy selected item unavailable" }}
            >
                <Text bind={buyLabel} color="#2a1200" shadow={false} />
            </Button>
            <Button close span={3} tooltip="Close market">
                Exit
            </Button>
        </Hotbar>
    </Window>
);
