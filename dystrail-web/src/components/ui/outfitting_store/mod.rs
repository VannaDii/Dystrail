//! A single outfitting workspace with visible item effects and a persistent loadout.
mod handlers;
mod planner;
mod state;
mod summary;
mod view;
use crate::game::store::calculate_cart_total;
pub use state::OutfittingStoreProps;
use state::{StoreState, load_store_data};
use yew::prelude::*;

#[function_component(OutfittingStore)]
pub fn outfitting_store(p: &OutfittingStoreProps) -> Html {
    crate::i18n::use_language();
    let state = use_state(StoreState::default);
    let review = use_state(|| false);
    let failed = use_state(|| false);
    let cart_key = format!(
        "dystrail.cart.{}.{}.{:?}.{}",
        p.game_state.seed, p.game_state.day, p.game_state.persona_id, p.resupply
    );
    {
        let state = state.clone();
        let failed = failed.clone();
        let cart_key = cart_key.clone();
        let review = review.clone();
        let resupply = p.resupply;
        let discount = f64::from(p.game_state.mods.store_discount_pct);
        use_effect_with((), move |()| match load_store_data() {
            Ok(data) => {
                let mut next = (*state).clone();
                next.store_data = data;
                if !resupply {
                    for (id, qty) in [
                        ("rations", 4),
                        ("water", 2),
                        ("spare_tire", 1),
                        ("battery", 1),
                        ("masks", 1),
                    ] {
                        next.cart.add_item(id, qty);
                    }
                }
                if let Some(saved) = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                    .and_then(|s| s.get_item(&cart_key).ok().flatten())
                    .and_then(|s| serde_json::from_str(&s).ok())
                {
                    next.cart = saved;
                }
                if let Some(saved_review) = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                    .and_then(|s| s.get_item(&format!("{cart_key}.view")).ok().flatten())
                    .and_then(|s| serde_json::from_str::<bool>(&s).ok())
                {
                    review.set(saved_review);
                }
                next.discount_pct = discount;
                next.cart.total_cents =
                    calculate_cart_total(&next.cart, &next.store_data, discount);
                state.set(next);
            }
            Err(_) => failed.set(true),
        });
    }
    {
        let cart = if state.store_data.categories.is_empty() {
            None
        } else {
            serde_json::to_string(&state.cart).ok()
        };
        let view = serde_json::to_string(&*review).ok();
        use_effect_with((cart, cart_key, view), move |(cart, key, view)| {
            if let Some(cart) = cart
                && let Some(storage) =
                    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = storage.set_item(key, cart);
                if let Some(view) = view {
                    let _ = storage.set_item(&format!("{key}.view"), view);
                }
            }
        });
    }
    if *failed {
        return html! {<p role="alert">{crate::i18n::t("ux.store_error")}</p>};
    }
    planner::render(&state, p, &review)
}
