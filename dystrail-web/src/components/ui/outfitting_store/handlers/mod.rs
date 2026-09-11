mod announce;
mod checkout;
mod quantity;
use super::state::{OutfittingStoreProps, StoreState};
pub use announce::{announce_quantity_change, format_currency};
pub use quantity::can_add_item;
use yew::prelude::*;
pub fn handle_cart_selection(
    index: u8,
    state: &StoreState,
    _handle: &UseStateHandle<StoreState>,
    p: &OutfittingStoreProps,
) {
    if index == 0 {
        checkout::handle_checkout(state, p);
    }
}
