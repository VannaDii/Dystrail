use super::view::{build_purchase_lines, current_quantity};
use crate::game::OtDeluxeStoreItem;
use crate::game::otdeluxe_state::{OtDeluxeInventory, OtDeluxeOxenState};

#[test]
fn current_quantity_counts_partial_ammo_boxes() {
    let inventory = OtDeluxeInventory {
        bullets: 15,
        ..OtDeluxeInventory::default()
    };
    let oxen = OtDeluxeOxenState::default();
    let qty = current_quantity(OtDeluxeStoreItem::AmmoBox, &inventory, oxen, 20);
    assert_eq!(qty, 1);
}

#[test]
fn build_purchase_lines_skips_zero_quantities() {
    let cart = vec![1, 0, 2, 0, 0, 3, 0];
    let lines = build_purchase_lines(&cart);
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].item, OtDeluxeStoreItem::Oxen);
    assert_eq!(lines[0].quantity, 1);
    assert_eq!(lines[1].item, OtDeluxeStoreItem::ClothesSet);
    assert_eq!(lines[1].quantity, 2);
    assert_eq!(lines[2].item, OtDeluxeStoreItem::Axle);
    assert_eq!(lines[2].quantity, 3);
}
