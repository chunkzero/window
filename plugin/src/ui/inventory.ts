import type {
    SlotArea,
    SlotGridPattern,
    SlotGridPatternOptions,
    SlotRange,
    SlotRectPattern,
    SlotRectPatternOptions,
    SlotRef,
    SlotSection,
    SlotSlotsPattern,
    SlotSlotsPatternOptions,
} from "./types.ts";

function requireObject<T extends object>(value: T | undefined, label: string): T {
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
        throw new Error(`${label} must be an object`);
    }
    return value;
}

function requireNonNegativeInteger(value: unknown, label: string): number {
    if (typeof value !== "number" || !Number.isInteger(value) || value < 0) {
        throw new Error(`${label} must be a non-negative integer`);
    }
    return value;
}

function requirePositiveInteger(value: unknown, label: string): number {
    if (typeof value !== "number" || !Number.isInteger(value) || value <= 0) {
        throw new Error(`${label} must be a positive integer`);
    }
    return value;
}

function slotRef(area: SlotArea, label: string, index: number): SlotRef {
    return { area, index: requireNonNegativeInteger(index, `${label} index`) };
}

function slotRange(area: SlotArea, label: string, first: number, last: number): SlotRange[] {
    const start = requireNonNegativeInteger(first, `${label} first`);
    const end = requireNonNegativeInteger(last, `${label} last`);
    if (end < start) {
        throw new Error(`${label} last must be greater than or equal to first`);
    }
    return [{ area, first: start, last: end }];
}

function section(value: SlotSection | undefined): SlotSection {
    if (value === undefined) {
        return "container";
    }
    if (value !== "container" && value !== "player" && value !== "hotbar") {
        throw new Error("slot pattern section must be `container`, `player`, or `hotbar`");
    }
    return value;
}

/** A slot in the opened container inventory. */
export function containerSlot(index: number): SlotRef {
    return slotRef("container", "containerSlot", index);
}

/** A slot in the viewing player's inventory, using server inventory indices. */
export function playerSlot(index: number): SlotRef {
    return slotRef("player", "playerSlot", index);
}

/** An inclusive range of slots in the opened container inventory. */
export function containerSlots(first: number, last: number): SlotRange[] {
    return slotRange("container", "containerSlots", first, last);
}

/** An inclusive range of slots in the viewing player's inventory. */
export function playerSlots(first: number, last: number): SlotRange[] {
    return slotRange("player", "playerSlots", first, last);
}

/** A slot in the viewing player's hotbar (indices 0 through 8). */
export function hotbarSlot(index: number): SlotRef {
    requireNonNegativeInteger(index, "hotbarSlot index");
    if (index > 8) {
        throw new Error("hotbarSlot index must be between 0 and 8");
    }
    return slotRef("player", "hotbarSlot", index);
}

/** An inclusive range of slots in the viewing player's hotbar. */
export function hotbarSlots(first: number, last: number): SlotRange[] {
    if (last > 8) {
        throw new Error("hotbarSlots last must be between 0 and 8");
    }
    return slotRange("player", "hotbarSlots", first, last);
}

/** Slot-space patterns for visible inventory sections. */
export const pattern = {
    /** A rectangular group in a visible inventory section. */
    rect(opts: SlotRectPatternOptions): SlotRectPattern {
        const o = requireObject(opts, "pattern.rect options");
        return {
            kind: "rect",
            section: section(o.section),
            x: requireNonNegativeInteger(o.x, "pattern.rect x"),
            y: requireNonNegativeInteger(o.y, "pattern.rect y"),
            width: requirePositiveInteger(o.width, "pattern.rect width"),
            height: requirePositiveInteger(o.height, "pattern.rect height"),
        };
    },

    /** Explicit local slot indices in a visible inventory section. */
    slots(indices: number[], options: SlotSlotsPatternOptions = {}): SlotSlotsPattern {
        const o = requireObject(options, "pattern.slots options");
        const slots = indices.map((index) => requireNonNegativeInteger(index, "pattern.slots index"));
        if (slots.length === 0) {
            throw new Error("pattern.slots requires at least one slot index");
        }
        return { kind: "slots", section: section(o.section), slots };
    },

    /** A grid of repeated rectangular cell groups in a visible section. */
    grid(opts: SlotGridPatternOptions): SlotGridPattern {
        const o = requireObject(opts, "pattern.grid options");
        return {
            kind: "grid",
            section: section(o.section),
            x: requireNonNegativeInteger(o.x, "pattern.grid x"),
            y: requireNonNegativeInteger(o.y, "pattern.grid y"),
            columns: requirePositiveInteger(o.columns, "pattern.grid columns"),
            rows: requirePositiveInteger(o.rows, "pattern.grid rows"),
            cell_width: requirePositiveInteger(o.cell_width, "pattern.grid cell_width"),
            cell_height: requirePositiveInteger(o.cell_height, "pattern.grid cell_height"),
        };
    },
};
