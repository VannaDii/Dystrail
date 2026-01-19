//! Outfitting Store component - Oregon Trail style store with numbered menu navigation.
//!
//! Provides purchasing flow for supplies, vehicle spares, PPE, and documents before the journey.

mod handlers;
mod state;
mod view;

pub use state::OutfittingStoreProps;

#[cfg(target_arch = "wasm32")]
use self::handlers::navigation::{
    get_max_menu_index, handle_back_navigation, handle_menu_selection,
};
use self::state::{StoreScreen, StoreState, load_store_data};
#[cfg(any(target_arch = "wasm32", test))]
use self::view::quantity::render_quantity_screen;
use self::view::{
    cart::render_cart_screen, category::render_category_screen, home::render_home_screen,
};
use crate::dom;
use crate::game::store::calculate_cart_total;
#[cfg(target_arch = "wasm32")]
use crate::input::{numeric_code_to_index, numeric_key_to_index};
#[cfg(target_arch = "wasm32")]
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
        #[cfg(target_arch = "wasm32")]
        {
            Callback::from(move |e: KeyboardEvent| {
                let key = e.key();
                let state = (*store_state).clone();

                if let Some(n) =
                    numeric_key_to_index(&key).or_else(|| numeric_code_to_index(&e.code()))
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
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (store_state, props);
            Callback::from(|_e: KeyboardEvent| {})
        }
    };

    {
        let list_ref = list_ref.clone();
        let focus_idx = store_state.focus_idx;
        #[cfg(target_arch = "wasm32")]
        {
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
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (list_ref, focus_idx);
        }
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
        #[cfg(any(target_arch = "wasm32", test))]
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

#[cfg(test)]
mod tests {
    use super::state::{StoreScreen, StoreState, load_store_data};
    use super::view::{
        cart::render_cart_screen, category::render_category_screen, home::render_home_screen,
        quantity::render_quantity_screen,
    };
    use crate::game::GameState;
    use crate::game::store::{Cart, calculate_cart_total};
    use futures::executor::block_on;
    use yew::LocalServerRenderer;
    use yew::prelude::*;

    fn first_category_id(state: &StoreState) -> String {
        state
            .store_data
            .categories
            .first()
            .map_or_else(|| String::from("fuel_food"), |cat| cat.id.clone())
    }

    fn first_item_id(state: &StoreState) -> String {
        state
            .store_data
            .categories
            .first()
            .and_then(|cat| cat.items.first())
            .map_or_else(|| String::from("food"), |item| item.id.clone())
    }

    #[derive(Properties, Clone, PartialEq)]
    struct StoreViewHarnessProps {
        screen: StoreScreen,
        with_cart_line: bool,
    }

    #[function_component(StoreViewHarness)]
    fn store_view_harness(props: &StoreViewHarnessProps) -> Html {
        crate::i18n::set_lang("en");
        let store_data = load_store_data().expect("store data should load");
        let mut cart = Cart::new();
        if props.with_cart_line
            && let Some(item) = store_data
                .categories
                .first()
                .and_then(|cat| cat.items.first())
        {
            cart.add_item(&item.id, 2);
        }
        let mut base_state = StoreState {
            store_data,
            cart,
            current_screen: props.screen.clone(),
            focus_idx: 1,
            discount_pct: 0.0,
        };
        base_state.cart.total_cents = calculate_cart_total(
            &base_state.cart,
            &base_state.store_data,
            base_state.discount_pct,
        );
        let store_state = use_state(move || base_state);
        let game_state = GameState::default();
        let list_ref = NodeRef::default();
        let on_keydown: Callback<web_sys::KeyboardEvent> = Callback::noop();
        let on_continue: Callback<(GameState, crate::game::store::Grants, Vec<String>)> =
            Callback::noop();
        let store_props = super::state::OutfittingStoreProps {
            game_state: game_state.clone(),
            on_continue,
        };

        match &props.screen {
            StoreScreen::Home => {
                render_home_screen(&store_state, &game_state, &list_ref, &on_keydown)
            }
            StoreScreen::Category(category_id) => render_category_screen(
                category_id,
                &store_state,
                &game_state,
                &list_ref,
                &on_keydown,
            ),
            StoreScreen::QuantityPrompt(item_id) => {
                render_quantity_screen(item_id, &store_state, &game_state, &list_ref, &on_keydown)
            }
            StoreScreen::Cart => render_cart_screen(
                &store_state,
                &game_state,
                &list_ref,
                &on_keydown,
                &store_props,
            ),
        }
    }

    #[test]
    fn store_views_render_all_screens() {
        let base = StoreState {
            store_data: load_store_data().expect("store data should load"),
            cart: Cart::new(),
            current_screen: StoreScreen::Home,
            focus_idx: 1,
            discount_pct: 0.0,
        };
        let category_id = first_category_id(&base);
        let item_id = first_item_id(&base);

        let html = block_on(
            LocalServerRenderer::<StoreViewHarness>::with_props(StoreViewHarnessProps {
                screen: StoreScreen::Home,
                with_cart_line: false,
            })
            .render(),
        );
        assert!(html.contains("outfitting-store"));

        let html = block_on(
            LocalServerRenderer::<StoreViewHarness>::with_props(StoreViewHarnessProps {
                screen: StoreScreen::Category(category_id),
                with_cart_line: false,
            })
            .render(),
        );
        assert!(html.contains("store-shell"));

        let html = block_on(
            LocalServerRenderer::<StoreViewHarness>::with_props(StoreViewHarnessProps {
                screen: StoreScreen::QuantityPrompt(item_id),
                with_cart_line: false,
            })
            .render(),
        );
        assert!(html.contains("qty-title"));

        let html = block_on(
            LocalServerRenderer::<StoreViewHarness>::with_props(StoreViewHarnessProps {
                screen: StoreScreen::Cart,
                with_cart_line: false,
            })
            .render(),
        );
        assert!(html.contains("empty-cart"));

        let html = block_on(
            LocalServerRenderer::<StoreViewHarness>::with_props(StoreViewHarnessProps {
                screen: StoreScreen::Cart,
                with_cart_line: true,
            })
            .render(),
        );
        assert!(html.contains("store-cart-line"));
    }

    #[test]
    fn outfitting_store_component_renders_shell() {
        crate::i18n::set_lang("en");
        let props = super::state::OutfittingStoreProps {
            game_state: GameState::default(),
            on_continue: Callback::noop(),
        };
        let html =
            block_on(LocalServerRenderer::<super::OutfittingStore>::with_props(props).render());
        assert!(html.contains("outfitting-store"));
    }
}
