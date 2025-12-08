//! Outfitting Store component - Oregon Trail style store with numbered menu navigation.
//!
//! Provides purchasing flow for supplies, vehicle spares, PPE, and documents before the journey.

mod handlers;
mod state;
mod view;

pub use state::OutfittingStoreProps;

use self::handlers::{get_max_menu_index, handle_back_navigation, handle_menu_selection};
use self::state::{StoreScreen, StoreState, load_store_data};
use self::view::{
    cart::render_cart_screen, category::render_category_screen, home::render_home_screen,
    quantity::render_quantity_screen,
};
use crate::dom;
use crate::game::store::calculate_cart_total;
use crate::input::{numeric_code_to_index, numeric_key_to_index};
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

#[function_component(OutfittingStore)]
pub fn outfitting_store(props: &OutfittingStoreProps) -> Html {
    let store_state = use_state(StoreState::default);
    let list_ref = use_node_ref();
    let _live_region_ref = use_node_ref();

    let discount_pct = f64::from(props.game_state.mods.store_discount_pct);

    {
        let store_state = store_state.clone();
        use_effect_with((), move |()| match load_store_data() {
            Ok(store_data) => {
                let mut state = (*store_state).clone();
                state.store_data = store_data;
                state.discount_pct = discount_pct;
                state.cart.total_cents =
                    calculate_cart_total(&state.cart, &state.store_data, discount_pct);
                store_state.set(state);
            }
            Err(e) => {
                dom::console_error(&format!("Failed to load store data: {e}"));
            }
        });
    }

    {
        let store_state = store_state.clone();
        use_effect_with(store_state.cart.clone(), move |cart| {
            let mut state = (*store_state).clone();
            state.cart.total_cents =
                calculate_cart_total(cart, &state.store_data, state.discount_pct);
            store_state.set(state);
        });
    }

    let on_keydown = {
        let store_state = store_state.clone();
        let props = props.clone();
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            let state = (*store_state).clone();

            if let Some(n) = numeric_key_to_index(&key).or_else(|| numeric_code_to_index(&e.code()))
            {
                handle_menu_selection(n, &state, &store_state, &props);
                e.prevent_default();
                return;
            }

            match key.as_str() {
                "Enter" | " " => {
                    handle_menu_selection(state.focus_idx, &state, &store_state, &props);
                    e.prevent_default();
                }
                "Escape" => {
                    handle_back_navigation(&state, &store_state);
                    e.prevent_default();
                }
                "ArrowDown" => {
                    let max_idx = get_max_menu_index(&state);
                    let next = if state.focus_idx >= max_idx {
                        1
                    } else {
                        state.focus_idx + 1
                    };
                    let mut new_state = state;
                    new_state.focus_idx = next;
                    store_state.set(new_state);
                    e.prevent_default();
                }
                "ArrowUp" => {
                    let max_idx = get_max_menu_index(&state);
                    let prev = if state.focus_idx <= 1 {
                        max_idx
                    } else {
                        state.focus_idx - 1
                    };
                    let mut new_state = state;
                    new_state.focus_idx = prev;
                    store_state.set(new_state);
                    e.prevent_default();
                }
                _ => {}
            }
        })
    };

    {
        let list_ref = list_ref.clone();
        let focus_idx = store_state.focus_idx;
        use_effect_with(focus_idx, move |idx| {
            if let Some(list) = list_ref.cast::<web_sys::Element>() {
                let sel = format!("[role='menuitem'][data-key='{idx}']");
                if let Ok(Some(el)) = list.query_selector(&sel) {
                    let _ = el
                        .dyn_into::<web_sys::HtmlElement>()
                        .ok()
                        .map(|e| e.focus());
                }
            }
        });
    }

    match &store_state.current_screen {
        StoreScreen::Home => {
            render_home_screen(&store_state, &props.game_state, &list_ref, &on_keydown)
        }
        StoreScreen::Category(category_id) => render_category_screen(
            category_id,
            &store_state,
            &props.game_state,
            &list_ref,
            &on_keydown,
        ),
        StoreScreen::QuantityPrompt(item_id) => render_quantity_screen(
            item_id,
            &store_state,
            &props.game_state,
            &list_ref,
            &on_keydown,
        ),
        StoreScreen::Cart => render_cart_screen(
            &store_state,
            &props.game_state,
            &list_ref,
            &on_keydown,
            props,
        ),
    }
}
