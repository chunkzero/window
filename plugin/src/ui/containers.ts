/** Vanilla container screen geometry, in GUI pixels from the top-left of the screen's texture. */
import type { ContainerKind } from "./types.ts";

export interface Rect {
    x: number;
    y: number;
    width: number;
    height: number;
}

/** One visible slot grid of a screen. */
export interface SectionLayout<Columns extends number = number, Rows extends number = number> {
    columns: Columns;
    rows: Rows;
    /** Each slot's 16x16 interior, row-major; `<Section>` cells are these grown by 1px. */
    slots: readonly Rect[];
    /** The union of `slots`. */
    bounds: Rect;
}

interface Shapes {
    generic_9x1: [9, 1];
    generic_9x2: [9, 2];
    generic_9x3: [9, 3];
    generic_9x4: [9, 4];
    generic_9x5: [9, 5];
    generic_9x6: [9, 6];
    anvil: [3, 1];
}

const SHAPES: Shapes = {
    generic_9x1: [9, 1],
    generic_9x2: [9, 2],
    generic_9x3: [9, 3],
    generic_9x4: [9, 4],
    generic_9x5: [9, 5],
    generic_9x6: [9, 6],
    anvil: [3, 1],
};

/** The geometry of one container screen, as the compiler lays windows out over it. */
export type ContainerLayout<K extends ContainerKind = ContainerKind> = K extends ContainerKind
    ? {
          container: K;
          /** The GUI texture's width; window children are placed relative to its top-left. */
          width: 176;
          height: number;
          /** Where the title text starts. */
          title: { x: number; y: number };
          sections: {
              container: SectionLayout<Shapes[K][0], Shapes[K][1]>;
              player: SectionLayout<9, 3>;
              hotbar: SectionLayout<9, 1>;
          };
      } & (K extends "anvil" ? { /** The native rename field `<Input>` types into. */ input: Rect } : {})
    : never;

const slot = (x: number, y: number): Rect => ({ x, y, width: 16, height: 16 });

function section<C extends number, R extends number>(columns: C, rows: R, slots: Rect[]): SectionLayout<C, R> {
    const x = Math.min(...slots.map((s) => s.x));
    const y = Math.min(...slots.map((s) => s.y));
    const right = Math.max(...slots.map((s) => s.x + s.width));
    const bottom = Math.max(...slots.map((s) => s.y + s.height));
    return { columns, rows, slots, bounds: { x, y, width: right - x, height: bottom - y } };
}

function grid<C extends number, R extends number>(x: number, y: number, columns: C, rows: R): SectionLayout<C, R> {
    const slots = Array.from({ length: columns * rows }, (_, i) =>
        slot(x + (i % columns) * 18, y + Math.floor(i / columns) * 18),
    );
    return section(columns, rows, slots);
}

/**
 * The geometry of the `container` screen: its size, title origin, and the slots of its container, player, and hotbar
 * sections; for an anvil, also its native input field.
 */
export function containerLayout<K extends ContainerKind>(container: K): ContainerLayout<K> {
    const shape = SHAPES[container] as [number, number] | undefined;
    if (shape === undefined) {
        throw new Error(`unknown container \`${String(container)}\``);
    }
    const [columns, rows] = shape;
    const anvil = container === "anvil";
    const player = anvil ? 84 : 31 + rows * 18;
    const layout = {
        container,
        width: 176,
        height: anvil ? 166 : 114 + rows * 18,
        title: anvil ? { x: 60, y: 6 } : { x: 8, y: 6 },
        sections: {
            container: anvil
                ? section(columns, rows, [slot(27, 47), slot(76, 47), slot(134, 47)])
                : grid(8, 18, columns, rows),
            player: grid(8, player, 9, 3),
            hotbar: grid(8, player + 58, 9, 1),
        },
        ...(anvil ? { input: { x: 59, y: 20, width: 110, height: 16 } } : {}),
    };
    return layout as unknown as ContainerLayout<K>;
}
