mod announce;
mod checkout;
mod quantity;
use super::state::{OutfittingStoreProps, StoreState};
pub use announce::announce_quantity_change;
pub use quantity::maximum_quantity;
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
