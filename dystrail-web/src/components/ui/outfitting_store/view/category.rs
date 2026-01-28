use super::super::handlers::announce::format_currency;
use super::super::state::{StoreScreen, StoreState, screen_state};
use super::item_card::render_store_item_card;
use crate::i18n;
use yew::prelude::*;

fn set_screen(state: &UseStateHandle<StoreState>, screen: StoreScreen) {
    state.set(screen_state(state, screen));
}

pub fn render_category_screen(
    category_id: &str,
    state: &UseStateHandle<StoreState>,
    game_state: &crate::game::GameState,
    list_ref: &NodeRef,
    on_keydown: &Callback<web_sys::KeyboardEvent>,
) -> Html {
    let Some(category) = state
        .store_data
        .categories
        .iter()
        .find(|c| c.id == *category_id)
    else {
        return html! { <div>{ "Category not found" }</div> };
    };

    let budget_str = format_currency(game_state.budget_cents - state.cart.total_cents);
    let category_name = i18n::t(&format!("store.categories.{category_id}"));
    let title = format!(
        "{category_name} — {budget_label}: {budget_str}",
        budget_label = i18n::t("store.budget")
    );

    let items = category.items.clone();
    let on_nav = {
        let state = state.clone();
        Callback::from(move |_| set_screen(&state, StoreScreen::Home))
    };

    let cart_cta = {
        let state = state.clone();
        Callback::from(move |_| set_screen(&state, StoreScreen::Cart))
    };

    html! {
        <main class="outfitting-store">
            <section role="region" aria-labelledby="category-title" onkeydown={on_keydown} class="store-shell" tabindex="0" data-testid="outfitting-store">
                <header class="store-header">
                    <div>
                        <h1 id="category-title">{ title }</h1>
                    </div>
                    <div class="store-budget">
                        <span class="label">{ i18n::t("store.budget") }</span>
                        <span class="value">{ budget_str }</span>
                    </div>
                </header>
                <div class="store-cart-summary" role="status" aria-live="polite">
                    <span>{ i18n::t("store.cart.title") }</span>
                    <span class="value">{ format_currency(state.cart.total_cents) }</span>
                </div>
                <div class="store-item-grid" ref={list_ref}>
                    { for items.iter().enumerate().map(|(i, item)| render_store_item_card(
                        u8::try_from(i + 1).unwrap_or_default(),
                        item,
                        state,
                        game_state
                    )) }
                </div>
                <div class="store-footer-row">
                    <button class="retro-btn-secondary" onclick={on_nav}>{ i18n::t("store.menu.back") }</button>
                    <button class="retro-btn-primary" onclick={cart_cta}>{ i18n::t("store.menu.view_cart") }</button>
                </div>
                <div aria-live="polite" aria-atomic="true" class="sr-only" id="store-status"></div>
            </section>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::state::load_store_data;
    use super::*;
    use crate::game::GameState;
    use crate::game::store::Cart;
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[function_component(MissingCategoryHarness)]
    fn missing_category_harness() -> Html {
        let store_data = load_store_data().expect("store data should load");
        let base_state = StoreState {
            store_data,
            cart: Cart::new(),
            current_screen: StoreScreen::Home,
            focus_idx: 1,
            discount_pct: 0.0,
        };
        let state = use_state(|| base_state);
        let list_ref = NodeRef::default();
        let on_keydown: Callback<web_sys::KeyboardEvent> = Callback::noop();
        render_category_screen(
            "missing",
            &state,
            &GameState::default(),
            &list_ref,
            &on_keydown,
        )
    }

    #[test]
    fn render_category_screen_reports_missing_category() {
        let html = block_on(LocalServerRenderer::<MissingCategoryHarness>::new().render());
        assert!(html.contains("Category not found"));
    }

    #[test]
    fn navigate_home_sets_screen_and_focus() {
        let store_data = load_store_data().expect("store data should load");
        let base_state = StoreState {
            store_data,
            cart: Cart::new(),
            current_screen: StoreScreen::Category(String::from("fuel_food")),
            focus_idx: 4,
            discount_pct: 0.0,
        };
        let next = screen_state(&base_state, StoreScreen::Home);
        assert!(matches!(next.current_screen, StoreScreen::Home));
        assert_eq!(next.focus_idx, 1);
    }

    #[test]
    fn open_cart_sets_screen_and_focus() {
        let store_data = load_store_data().expect("store data should load");
        let base_state = StoreState {
            store_data,
            cart: Cart::new(),
            current_screen: StoreScreen::Home,
            focus_idx: 2,
            discount_pct: 0.0,
        };
        let next = screen_state(&base_state, StoreScreen::Cart);
        assert!(matches!(next.current_screen, StoreScreen::Cart));
        assert_eq!(next.focus_idx, 1);
    }

    #[test]
    fn set_screen_executes_state_update() {
        #[function_component(SetScreenHarness)]
        fn set_screen_harness() -> Html {
            let store_data = load_store_data().expect("store data should load");
            let base_state = StoreState {
                store_data,
                cart: Cart::new(),
                current_screen: StoreScreen::Home,
                focus_idx: 2,
                discount_pct: 0.0,
            };
            let state = use_state(|| base_state);
            let invoked = use_mut_ref(|| false);
            if !*invoked.borrow() {
                *invoked.borrow_mut() = true;
                set_screen(&state, StoreScreen::Cart);
            }
            let called = if *invoked.borrow() { "true" } else { "false" };
            html! { <div data-called={called} /> }
        }

        let html = block_on(LocalServerRenderer::<SetScreenHarness>::new().render());
        assert!(html.contains("data-called=\"true\""));
    }
}
