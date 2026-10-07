import {
    Box,
    Button,
    Choice,
    Collection,
    Container,
    Header,
    Hotbar,
    Icon,
    Player,
    Row,
    Show,
    Sprite,
    Text,
    Toggle,
    Window,
} from "#plugins/window";
import type { Indexed, SelectionHandle, SpriteHandle, TextHandle, ToggleHandle } from "#plugins/window";

import {
    affordable,
    affordableLamp,
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
    favoritesLamp,
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

/** One tab per value of `bind`, labelled by the matching entry of `label`. */
function SelectionTabs<V extends string>(props: {
    bind: SelectionHandle<V>;
    label: Indexed<TextHandle, readonly [number]>;
    tooltips: Record<V, string>;
}) {
    return props.bind.values.map((value, i) => (
        <Choice
            onClick={props.bind.set(value)}
            span={3}
            selected={{ tooltip: props.tooltips[value], sprite: "tab_selected" }}
            unselected={{ tooltip: props.tooltips[value], sprite: "tab" }}
        >
            <Text bind={props.label.at(i)} />
        </Choice>
    ));
}

/** A raised toggle with a lamp at its left and its label centered in the remaining width. */
function LampToggle(props: { bind: ToggleHandle; lamp: SpriteHandle; title: string; tooltip: string }) {
    return (
        <Toggle
            bind={props.bind}
            span={3}
            frame="button"
            on={{ tooltip: props.tooltip + " on" }}
            off={{ tooltip: props.tooltip + " off" }}
        >
            <Icon bind={props.lamp} size={4} />
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
            <SelectionTabs
                bind={category}
                label={categoryLabel}
                tooltips={{ all: "All items", gear: "Gear", magic: "Magic" }}
            />

            <Collection bind={products} rows={3} frame="slot" selected="slot_selected" />

            <Button
                onClick={previous}
                enabled={canPrevious}
                span={3}
                frame="button"
                tooltip="Previous page"
                disabled={{ tooltip: "Previous page unavailable", sprite: "action_disabled" }}
            >
                <Text bind={previousLabel} />
            </Button>
            <Row span={3} frame="recess">
                <Text bind={page} align="center" />
            </Row>
            <Button
                onClick={next}
                enabled={canNext}
                span={3}
                frame="button"
                tooltip="Next page"
                disabled={{ tooltip: "Next page unavailable", sprite: "action_disabled" }}
            >
                <Text bind={nextLabel} />
            </Button>

            <Row span={9} frame="recess" padding={{ left: 5, right: 4 }} gap={4}>
                <Text bind={selectedName} color="#ffb20b" />
                <Text bind={price} width={40} align="right" />
                <Show when={hasPrice}>
                    <Sprite name="coin" />
                </Show>
            </Row>
        </Container>

        <Player>
            <SelectionTabs
                bind={sort}
                label={sortLabel}
                tooltips={{ featured: "Top picks first", price: "Price sort", name: "Name sort" }}
            />

            <LampToggle bind={favorites} lamp={favoritesLamp} title="Favs" tooltip="Favorites only" />
            <LampToggle bind={affordable} lamp={affordableLamp} title="Afford" tooltip="Affordable only" />
            <Button onClick={search} span={2} frame="button" tooltip="Search the catalog">
                Find
            </Button>
            <Button
                onClick={clearSearch}
                enabled={hasQuery}
                frame="button"
                tooltip="Clear search"
                disabled={{ tooltip: "Clear search unavailable", sprite: "clear_search_disabled" }}
            >
                <Icon bind="clear_search_icon" size={[8, 6]} sprite="icon_clear" />
            </Button>

            <Row span={9} frame="recess" padding={{ left: 5, right: 4 }} gap={4}>
                <Text bind={status} color="#bceeff" />
                <Text bind={balance} width={48} align="right" color="#ffb20b" />
                <Sprite name="coin" />
            </Row>
        </Player>

        <Hotbar>
            <Button
                onClick={buy}
                enabled={canBuy}
                span={6}
                frame="button_accent"
                tooltip="Buy selected item"
                disabled={{ tooltip: "Buy selected item unavailable", sprite: "buy_disabled" }}
            >
                <Text bind={buyLabel} color="#2a1200" shadow={false} />
            </Button>
            <Button close span={3} frame="button" tooltip="Close market">
                Exit
            </Button>
        </Hotbar>
    </Window>
);
