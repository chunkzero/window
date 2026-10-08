import { Box, Image, Switch, Text, Window, industrial } from "#plugins/window";
import type { ToggleHandle } from "#plugins/window";

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
        <industrial.Toggle
            bind={props.bind}
            span={3}
            on={{ tooltip: props.tooltip + " on" }}
            off={{ tooltip: props.tooltip + " off" }}
        >
            <Switch on={props.bind}>
                {{ true: <Image art={industrial.art.lampOn} />, false: <Image art={industrial.art.lampOff} /> }}
            </Switch>
            <Text>{props.title}</Text>
        </industrial.Toggle>
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
        frame={industrial.art.shell}
        bleed={{ top: 1, right: 4, bottom: 7, left: 4 }}
        text={{ color: "#ffffff", shadow: true, smallCaps: true }}
    >
        <industrial.Header padding={{ top: 3 }} align="start">
            <Box frame={industrial.art.recess} width={116} height={12} justify="center" align="center">
                <Text>Foundry Exchange</Text>
            </Box>
        </industrial.Header>

        <Box frame={industrial.art.panel} x={4} y={136} width={168} height={82} />
        <Box frame={industrial.art.hazardBar} x={-4} y={223} width={184} height={6} />
        <industrial.Row x={52} y={131} gap={4}>
            {Array.from({ length: 8 }, () => (
                <Image art={ventSlot} />
            ))}
        </industrial.Row>
        {rivets.map(([x, y]) => (
            <Image art={industrial.art.rivet} x={x} y={y} />
        ))}

        <industrial.Container frame={industrial.art.panel} outset={{ top: 2, right: 3, bottom: 3, left: 3 }}>
            <industrial.Tabs bind={category} span={3} tooltip={(value) => categoryTooltips[value]}>
                {(_, i) => <Text bind={categoryLabel.at(i)} />}
            </industrial.Tabs>

            <industrial.Collection bind={products} rows={3} />

            <industrial.Button
                onClick={previous}
                enabled={canPrevious}
                span={3}
                tooltip="Previous page"
                disabled={{ tooltip: "Previous page unavailable" }}
            >
                <Text bind={previousLabel} />
            </industrial.Button>
            <industrial.Row span={3} frame={industrial.art.recess}>
                <Text bind={page} align="center" />
            </industrial.Row>
            <industrial.Button
                onClick={next}
                enabled={canNext}
                span={3}
                tooltip="Next page"
                disabled={{ tooltip: "Next page unavailable" }}
            >
                <Text bind={nextLabel} />
            </industrial.Button>

            <industrial.Row span={9} frame={industrial.art.recess} padding={{ left: 5, right: 4 }} gap={4}>
                <Text bind={selectedName} color="#ffb20b" />
                <Text bind={price} width={40} align="right" />
                <industrial.Show when={hasPrice}>
                    <Image art={coin} />
                </industrial.Show>
            </industrial.Row>
        </industrial.Container>

        <industrial.Player>
            <industrial.Tabs bind={sort} span={3} tooltip={(value) => sortTooltips[value]}>
                {(_, i) => <Text bind={sortLabel.at(i)} />}
            </industrial.Tabs>

            <LampToggle bind={favorites} title="Favs" tooltip="Favorites only" />
            <LampToggle bind={affordable} title="Afford" tooltip="Affordable only" />
            <industrial.Button onClick={search} span={2} tooltip="Search the catalog">
                Find
            </industrial.Button>
            <industrial.Button
                onClick={clearSearch}
                enabled={hasQuery}
                tooltip="Clear search"
                disabled={{ tooltip: "Clear search unavailable" }}
            >
                <Image art={icons.clear} />
            </industrial.Button>

            <industrial.Row span={9} frame={industrial.art.recess} padding={{ left: 5, right: 4 }} gap={4}>
                <Text bind={status} color="#bceeff" />
                <Text bind={balance} width={48} align="right" color="#ffb20b" />
                <Image art={coin} />
            </industrial.Row>
        </industrial.Player>

        <industrial.Hotbar>
            <industrial.Button
                onClick={buy}
                enabled={canBuy}
                span={6}
                frame={industrial.art.buttonAccent}
                tooltip="Buy selected item"
                disabled={{ tooltip: "Buy selected item unavailable" }}
            >
                <Text bind={buyLabel} color="#2a1200" shadow={false} />
            </industrial.Button>
            <industrial.Button close span={3} tooltip="Close market">
                Exit
            </industrial.Button>
        </industrial.Hotbar>
    </Window>
);
