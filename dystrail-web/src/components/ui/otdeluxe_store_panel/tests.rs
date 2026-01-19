use super::OtDeluxeStorePanel;
use super::view::OtDeluxeStorePanelProps;
use super::view::{build_purchase_lines, current_quantity};
use crate::game::otdeluxe_state::{OtDeluxeInventory, OtDeluxeOxenState, OtDeluxeState};
use crate::game::{GameState, MechanicalPolicyId, OtDeluxeStoreItem};
use futures::executor::block_on;
use std::rc::Rc;
use yew::{Callback, LocalServerRenderer};

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

#[test]
fn store_panel_renders_cards_and_disabled_checkout() {
    crate::i18n::set_lang("en");
    let state = GameState {
        mechanical_policy: MechanicalPolicyId::OtDeluxe90s,
        ot_deluxe: OtDeluxeState {
            inventory: OtDeluxeInventory {
                cash_cents: 500,
                ..OtDeluxeInventory::default()
            },
            ..OtDeluxeState::default()
        },
        ..GameState::default()
    };
    let props = OtDeluxeStorePanelProps {
        state: Rc::new(state),
        on_purchase: Callback::noop(),
        on_leave: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<OtDeluxeStorePanel>::with_props(props).render());
    assert!(html.contains("otdeluxe-store-title"));
    assert!(html.contains("store-card"));
    assert!(html.contains("retro-btn-primary"));
}
