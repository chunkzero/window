import { Box, Case, Fullscreen, Image, Region, Switch, Text, create, shape } from "plugin:window/ui";
import type { ActionHandle, FlagHandle, Indexed, ValueHandle } from "plugin:window/bind";
import { Button, Column, Repeater, Row, Show, Spacer, Tabs, art } from "plugin:window/theme/industrial";

import { coin, glass, icons, productIcon, storeIcons, toneLamp } from "./art.ts";
import {
    STORE_ICONS,
    addToCart,
    buyNow,
    buyNowLabel,
    canBuyNow,
    canCheckout,
    canLess,
    canMore,
    cardFeatured,
    cardFilled,
    cardIcon,
    cardName,
    cardPrice,
    cardSelected,
    cartDown,
    cartEmpty,
    cartName,
    cartPrice,
    cartQuantity,
    cartThumb,
    cartTotal,
    cartUp,
    catalogDown,
    catalogFocused,
    catalogThumb,
    catalogUp,
    checkout,
    clearCart,
    detailBlurb,
    detailIcon,
    detailInCart,
    detailKind,
    detailName,
    detailPrice,
    detailTier,
    less,
    more,
    openCart,
    openDetails,
    pickCard,
    quantity,
    removeLine,
    storeBalance,
    storeCartCount,
    storeCategory,
    storeClearSearch,
    storeFind,
    storeHasSearch,
    storePane,
    storeResults,
    storeSearch,
    storeSort,
    storeSortLabel,
    storeToast,
    tone,
} from "./handles.ts";

type StoreIcon = (typeof STORE_ICONS)[number];

const s = create({
    nav: { frame: glass, tint: "#24508c", height: 24, padding: { left: 6, right: 4 }, gap: 4 },
    pane: { frame: glass, tint: "#1b3766" },
    footer: { frame: glass, tint: "#24508c", height: 16, padding: { left: 5, right: 2 }, gap: 4 },
    field: { frame: art.recess, width: 116, height: 16, padding: { left: 4, right: 3 }, gap: 4 },
    dim: { color: "#9cc4e4" },
    gold: { color: "#ffd27a" },
});

/** The bar under the selected card. */
const selectedBar = shape({
    kind: "panel",
    width: 48,
    height: 2,
    fill: "#ffb20b",
    border_width: 0,
    radius: 0,
    inset_depth: 0,
});

const categories = { all: "All", gear: "Gear", magic: "Magic", deals: "Deals" };

/** A product's art, switched by an icon handle and stretched over `size` pixels. */
function ProductIcon(props: { bind: ValueHandle<StoreIcon>; size?: number }) {
    const size = props.size ?? 16;
    return (
        <Switch bind={props.bind}>
            {STORE_ICONS.map((id) => (
                <Case value={id} width={size} height={size} frame={id === "none" ? undefined : productIcon(id)} />
            ))}
        </Switch>
    );
}

/** A side pane tab; clicking it opens its pane and gives it the wheel, even when it is already selected. */
function PaneTab(props: { label: string; open: ActionHandle; value: "details" | "cart" }) {
    const face = (frame: typeof art.button) => (
        <Button onClick={props.open} frame={frame} style={{ base: { width: 78, height: 16 } }}>
            <Text>{props.label}</Text>
        </Button>
    );
    return (
        <Switch bind={storePane.is(props.value)}>
            <Case value="true">{face(art.buttonSelected)}</Case>
            <Case value="false">{face(art.button)}</Case>
        </Switch>
    );
}

/** A lamp that lights while its pane has the scroll wheel. */
function FocusLamp(props: { lit: "true" | "false" }) {
    const unlit = props.lit === "true" ? "false" : "true";
    return (
        <Switch bind={catalogFocused}>
            <Case value={props.lit}>
                <Image art={art.lampOn} />
            </Case>
            <Case value={unlit}>
                <Image art={art.lampOff} />
            </Case>
        </Switch>
    );
}

/**
 * A vertical scrollbar: arrow buttons around a track whose segments light while their stretch of the list is in view,
 * so the thumb shows both where the view is and how much of the list it covers.
 */
function Scrollbar(props: {
    thumb: Indexed<FlagHandle, readonly [number]>;
    segments: number;
    segment: number;
    up: ActionHandle;
    down: ActionHandle;
}) {
    const segment = shape({
        kind: "panel",
        width: 4,
        height: props.segment,
        fill: "#ffb20b",
        border_width: 0,
        radius: 0,
        inset_depth: 0,
    });
    const arrow = { base: { width: 8, height: 10, padding: 0 } };
    return (
        <Column gap={2} align="center">
            <Button onClick={props.up} style={arrow}>
                <Image art={storeIcons.arrowUp} />
            </Button>
            <Column frame={art.recess} width={8} padding={2} gap={1} align="center">
                {Array.from({ length: props.segments }, (_, i) => (
                    <Show when={props.thumb.at(i)}>
                        <Image art={segment} />
                    </Show>
                ))}
            </Column>
            <Button onClick={props.down} style={arrow}>
                <Image art={storeIcons.arrowDown} />
            </Button>
        </Column>
    );
}

const nav = (
    <Row style={s.nav}>
        <Box frame={productIcon("nether_star")} width={16} height={16} />
        <Text color="#ffe2b8">Foundry Market</Text>
        <Tabs bind={storeCategory} style={{ base: { width: 34, height: 16 } }}>
            {(value) => <Text>{categories[value]}</Text>}
        </Tabs>
        <Spacer />
        <Row style={s.field}>
            <Image art={storeIcons.magnifier} />
            <Switch on={storeHasSearch}>
                {{
                    true: <Text bind={storeSearch} width={94} color="#ffffff" />,
                    false: <Text style={s.dim}>Search products</Text>,
                }}
            </Switch>
            <Region onClick={storeFind} tooltip={{ title: "Search", lines: ["Type to filter, then press Esc"] }} />
        </Row>
        <Button onClick={storeClearSearch} enabled={storeHasSearch} style={{ base: { width: 16, height: 16 } }}>
            <Image art={icons.clear} />
        </Button>
        <Button
            onClick={openCart}
            frame={art.buttonAccent}
            style={{ base: { width: 50, height: 16 } }}
            tooltip="Open your cart"
        >
            <Image art={storeIcons.cart} />
            <Text bind={storeCartCount} width={20} color="#2a1200" />
        </Button>
    </Row>
);

const catalog = (
    <Column style={s.pane} width={292} padding={6} gap={4}>
        <Row height={14} gap={4}>
            <FocusLamp lit="true" />
            <Text bind={storeResults} style={s.gold} />
            <Button onClick={storeSort} style={{ base: { width: 70, height: 14 } }}>
                <Text bind={storeSortLabel} width={66} />
            </Button>
        </Row>
        <Row gap={4} align="start">
            <Repeater
                fixed
                cell={[5, 3]}
                columns={3}
                rows={3}
                onClick={pickCard}
                style={{ cell: { frame: glass, tint: "#4f86c6", gap: 2 } }}
                tooltip={{ title: "View product", lines: ["Click for details"] }}
            >
                {(i) => [
                    <ProductIcon bind={cardIcon.at(i)} />,
                    <Text bind={cardName.at(i)} width={84} align="center" overflow="ellipsis" />,
                    <Show when={cardFilled.at(i)}>
                        <Row gap={2}>
                            <Image art={coin} />
                            <Text bind={cardPrice.at(i)} width={30} style={s.gold} />
                        </Row>
                    </Show>,
                    <Show when={cardFeatured.at(i)} absolute top={3} right={3}>
                        <Image art={art.lampOn} tint="#ffb20b" />
                    </Show>,
                    <Show when={cardSelected.at(i)} absolute bottom={1} left={20}>
                        <Image art={selectedBar} />
                    </Show>,
                ]}
            </Repeater>
            <Scrollbar thumb={catalogThumb} segments={8} segment={16} up={catalogUp} down={catalogDown} />
        </Row>
        <Text style={s.dim}>Scroll the wheel or use the arrows to browse</Text>
    </Column>
);

const details = (
    <Column gap={4}>
        <Row gap={6} align="start">
            <Row frame={art.recess} width={36} height={36} justify="center">
                <ProductIcon bind={detailIcon} size={32} />
            </Row>
            <Column gap={2}>
                <Text bind={detailName} width={116} color={detailTier} />
                <Text bind={detailKind} width={116} style={s.dim} />
                <Row gap={2}>
                    <Image art={coin} />
                    <Text bind={detailPrice} width={60} style={s.gold} />
                </Row>
            </Column>
        </Row>
        <Text bind={detailBlurb} width={158} lines={3} color="#d8e8f8" />
        <Text bind={detailInCart} width={158} style={s.dim} />
        <Row gap={3}>
            <Text width={40}>Quantity</Text>
            <Button onClick={less} enabled={canLess} style={{ base: { width: 16, height: 16 } }}>
                -
            </Button>
            <Row frame={art.recess} width={26} height={16} justify="center">
                <Text bind={quantity} width={20} align="center" />
            </Row>
            <Button onClick={more} enabled={canMore} style={{ base: { width: 16, height: 16 } }}>
                +
            </Button>
        </Row>
        <Row gap={4}>
            <Button onClick={addToCart} frame={art.buttonAccent} style={{ base: { width: 77, height: 18 } }}>
                <Text color="#2a1200">Add to cart</Text>
            </Button>
            <Button
                onClick={buyNow}
                enabled={canBuyNow}
                frame={art.buttonConfirm}
                style={{ base: { width: 77, height: 18 } }}
            >
                <Text bind={buyNowLabel} width={73} />
            </Button>
        </Row>
    </Column>
);

const cart = (
    <Column gap={4}>
        <Row gap={4} height={10}>
            <FocusLamp lit="false" />
            <Text style={s.dim}>Click a line to remove one</Text>
        </Row>
        <Switch on={cartEmpty}>
            {{
                true: (
                    <Row frame={art.recess} width={158} height={90} justify="center">
                        <Text style={s.dim}>Your cart is empty</Text>
                    </Row>
                ),
                false: (
                    <Row gap={4} align="start">
                        <Repeater
                            fixed
                            cell={[8, 1]}
                            columns={1}
                            rows={5}
                            onClick={removeLine}
                            style={{ cell: { frame: glass, tint: "#4f86c6" } }}
                        >
                            {(i) => (
                                <Row width={138} gap={2}>
                                    <Text bind={cartName.at(i)} width={80} overflow="ellipsis" />
                                    <Text bind={cartQuantity.at(i)} width={18} style={s.dim} />
                                    <Text bind={cartPrice.at(i)} width={32} align="right" style={s.gold} />
                                </Row>
                            )}
                        </Repeater>
                        <Scrollbar thumb={cartThumb} segments={5} segment={10} up={cartUp} down={cartDown} />
                    </Row>
                ),
            }}
        </Switch>
        <Row gap={2}>
            <Text width={30}>Total</Text>
            <Image art={coin} />
            <Text bind={cartTotal} width={60} style={s.gold} />
        </Row>
        <Row gap={4}>
            <Button
                onClick={checkout}
                enabled={canCheckout}
                frame={art.buttonConfirm}
                style={{ base: { width: 77, height: 18 } }}
            >
                Checkout
            </Button>
            <Button onClick={clearCart} frame={art.buttonDanger} style={{ base: { width: 77, height: 18 } }}>
                Clear
            </Button>
        </Row>
    </Column>
);

const footer = (
    <Row style={s.footer}>
        <Image art={coin} />
        <Text bind={storeBalance} width={44} style={s.gold} />
        <Image art={toneLamp} tint={tone} />
        <Text bind={storeToast} color={tone} grow={1} />
        <Text style={s.dim}>Right click: cart / details</Text>
        <Button close style={{ base: { width: 40, height: 12 } }} tooltip="Leave the market (or sneak)">
            Close
        </Button>
    </Row>
);

/**
 * A web-style storefront over the world, in horse mode so the view holds still behind it: no backdrop, translucent
 * glass panes, category tabs, a search box the player types into in place, a product grid and a cart that each scroll,
 * and a detail pane to buy from.
 */
export default (
    <Fullscreen name="storefront" mode="horse" padding={6} gap={6} text={{ color: "#ffffff" }}>
        {nav}
        <Row gap={6} align="stretch" grow={1}>
            {catalog}
            <Column style={s.pane} grow={1} padding={5} gap={4}>
                <Row gap={4}>
                    <PaneTab label="Details" open={openDetails} value="details" />
                    <PaneTab label="Cart" open={openCart} value="cart" />
                </Row>
                <Switch on={storePane}>{{ details, cart }}</Switch>
            </Column>
        </Row>
        {footer}
    </Fullscreen>
);
