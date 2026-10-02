use super::*;

#[test]
fn generic_9x6_metrics() {
    let k = ContainerKind::Generic9x6;
    assert_eq!(k.gui_size(), Size::new(176, 222));
    assert_eq!(k.title_origin(), Point::new(8, 6));
    assert_eq!(k.slot_count(), 54);
    assert_eq!(k.slot_rect(0), Some(Rect::new(8, 18, 16, 16)));
    assert_eq!(k.slot_rect(10), Some(Rect::new(26, 36, 16, 16)));
    assert_eq!(k.player_slot_rect(9), Some(Rect::new(8, 140, 16, 16)));
    assert_eq!(k.player_slot_rect(0), Some(Rect::new(8, 198, 16, 16)));
}

#[test]
fn anvil_metrics_match_the_vanilla_screen() {
    let k = ContainerKind::Anvil;
    assert_eq!(k.gui_size(), Size::new(176, 166));
    assert_eq!(k.title_origin(), Point::new(60, 6));
    assert_eq!(k.slot_count(), 3);
    assert_eq!(k.slot_rect(0), Some(Rect::new(27, 47, 16, 16)));
    assert_eq!(k.slot_rect(1), Some(Rect::new(76, 47, 16, 16)));
    assert_eq!(k.slot_rect(2), Some(Rect::new(134, 47, 16, 16)));
    assert_eq!(k.player_slot_rect(9), Some(Rect::new(8, 84, 16, 16)));
    assert_eq!(k.player_slot_rect(0), Some(Rect::new(8, 142, 16, 16)));
}

#[test]
fn generic_sections_preserve_native_gutters() {
    for kind in ContainerKind::ALL.into_iter().filter(|kind| *kind != ContainerKind::Anvil) {
        let rows = kind.rows();
        assert_eq!(kind.section_bounds(InventorySlotSection::Container), Rect::new(8, 18, 160, 18 * rows - 2),);
        assert_eq!(kind.section_bounds(InventorySlotSection::Player), Rect::new(8, 18 * rows as i32 + 32, 160, 52),);
        assert_eq!(kind.section_bounds(InventorySlotSection::Hotbar), Rect::new(8, 18 * rows as i32 + 90, 160, 16),);
        assert_eq!(
            kind.reserved_gutters(),
            [
                InventoryGutter { name: "container-to-player", rect: Rect::new(8, 18 * rows as i32 + 16, 160, 16) },
                InventoryGutter { name: "player-to-hotbar", rect: Rect::new(8, 18 * rows as i32 + 84, 160, 6) },
            ],
        );
    }
}

#[test]
fn anvil_sections_preserve_native_gutters() {
    let kind = ContainerKind::Anvil;
    assert_eq!(kind.section_bounds(InventorySlotSection::Container), Rect::new(27, 47, 123, 16),);
    assert_eq!(
        kind.reserved_gutters(),
        [
            InventoryGutter { name: "container-to-player", rect: Rect::new(8, 63, 160, 21) },
            InventoryGutter { name: "player-to-hotbar", rect: Rect::new(8, 136, 160, 6) },
        ],
    );
}

#[test]
fn parse_round_trips() {
    for k in ContainerKind::ALL {
        assert_eq!(ContainerKind::parse(k.id()), Some(k));
    }
    assert_eq!(ContainerKind::parse("generic_9x7"), None);
}

#[test]
fn button_rect_maps_to_slots() {
    let k = ContainerKind::Generic9x6;
    // A 72×20 button at (52, 190) sits over row 5 (last container row was
    // y = 18 + 5*18 = 108) — actually y=190 is below the container grid,
    // so it overlaps nothing.
    assert!(k.slots_overlapping(&Rect::new(52, 190, 72, 20)).is_empty());
    // A rect over the first two slots.
    assert_eq!(k.slots_overlapping(&Rect::new(8, 18, 34, 16)), vec![0, 1]);
}
