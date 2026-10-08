import {
    Box,
    Image,
    Text,
    Window,
    action,
    collection,
    derive,
    flag,
    industrial,
    selection,
    sprite,
    text,
    texture,
    toggle,
} from "#plugins/window";
import type { ToggleHandle } from "#plugins/window";

import { coin } from "./sprites.ts";

const ventSlot = derive((get) => ({
    ...get(industrial.art.recess),
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
        <industrial.Button
            onClick={action(props.id)}
            enabled={flag(`can_${props.id}`)}
            span={props.span ?? 1}
            tooltip={props.tooltip}
            disabled={{ tooltip: props.tooltip + " unavailable" }}
            {...(props.accent === true ? { frame: industrial.art.buttonAccent } : {})}
        >
            {props.children as never}
        </industrial.Button>
    );
}

/** A raised toggle with a lamp at its left and its label centered in the remaining width. */
function LampToggle(props: { bind: ToggleHandle; title: string; tooltip: string }) {
    return (
        <industrial.Toggle
            bind={props.bind}
            span={3}
            on={{ tooltip: props.tooltip + " on" }}
            off={{ tooltip: props.tooltip + " off" }}
        >
            <Image bind={sprite(props.bind.id + "_lamp", { only: ["lamp_on", "lamp_off"] })} size={4} />
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
            <industrial.Tabs bind={category} span={3}>
                <industrial.Tab value="all" tooltip="All items">
                    <Text bind={text("category_all_label")} />
                </industrial.Tab>
                <industrial.Tab value="gear" tooltip="Gear">
                    <Text bind={text("category_gear_label")} />
                </industrial.Tab>
                <industrial.Tab value="magic" tooltip="Magic">
                    <Text bind={text("category_magic_label")} />
                </industrial.Tab>
            </industrial.Tabs>

            <industrial.Collection bind={products} rows={3} />

            <Action id="previous" span={3} tooltip="Previous page">
                <Text bind={text("previous_label")} />
            </Action>
            <industrial.Row span={3} frame={industrial.art.recess}>
                <Text bind={text("page")} align="center" />
            </industrial.Row>
            <Action id="next" span={3} tooltip="Next page">
                <Text bind={text("next_label")} />
            </Action>

            <industrial.Row span={9} frame={industrial.art.recess} padding={{ left: 5, right: 4 }} gap={4}>
                <Text bind={text("selection")} color="#ffb20b" />
                <Text bind={text("price")} width={40} align="right" />
                <industrial.Show when={hasPrice}>
                    <Image art={coin} />
                </industrial.Show>
            </industrial.Row>
        </industrial.Container>

        <industrial.Player>
            <industrial.Tabs bind={sort} span={3}>
                <industrial.Tab value="featured" tooltip="Top picks first">
                    <Text bind={text("sort_featured_label")} />
                </industrial.Tab>
                <industrial.Tab value="price" tooltip="Price sort">
                    <Text bind={text("sort_price_label")} />
                </industrial.Tab>
                <industrial.Tab value="name" tooltip="Name sort">
                    <Text bind={text("sort_name_label")} />
                </industrial.Tab>
            </industrial.Tabs>

            <LampToggle bind={toggle("favorites")} title="Favs" tooltip="Favorites only" />
            <LampToggle bind={toggle("affordable")} title="Afford" tooltip="Affordable only" />
            <industrial.Button onClick={action("search")} span={2} tooltip="Search the catalog">
                Find
            </industrial.Button>
            <Action id="clear_search" tooltip="Clear search">
                <Image art={texture("window/icons/clear.png")} />
            </Action>

            <industrial.Row span={9} frame={industrial.art.recess} padding={{ left: 5, right: 4 }} gap={4}>
                <Text bind={text("status")} color="#bceeff" />
                <Text bind={text("balance")} width={48} align="right" color="#ffb20b" />
                <Image art={coin} />
            </industrial.Row>
        </industrial.Player>

        <industrial.Hotbar>
            <Action id="buy" span={6} accent tooltip="Buy selected item">
                <Text bind={text("buy_label")} color="#2a1200" shadow={false} />
            </Action>
            <industrial.Button close span={3} tooltip="Close market">
                Exit
            </industrial.Button>
        </industrial.Hotbar>
    </Window>
);
