use super::super::handlers::announce::format_currency;
use super::super::state::set_screen;
use super::super::state::{StoreScreen, StoreState};
use crate::i18n;
use std::collections::BTreeMap;
use yew::prelude::*;

pub fn render_home_screen(
    state: &UseStateHandle<StoreState>,
    game_state: &crate::game::GameState,
    list_ref: &NodeRef,
    on_keydown: &Callback<web_sys::KeyboardEvent>,
) -> Html {
    let budget_str = format_currency(game_state.budget_cents - state.cart.total_cents);
    let title = i18n::tr(
        "store.menu.home",
        Some(&{
            let mut vars = BTreeMap::new();
            vars.insert("budget", budget_str.as_str());
            vars
        }),
    );

    let categories = [
        (1u8, i18n::t("store.categories.fuel_food")),
        (2u8, i18n::t("store.categories.vehicle")),
        (3u8, i18n::t("store.categories.ppe")),
        (4u8, i18n::t("store.categories.docs")),
        (5u8, i18n::t("store.menu.view_cart")),
        (0u8, i18n::t("store.menu.continue")),
    ];

    let remaining_budget = game_state.budget_cents - state.cart.total_cents;
    let can_continue = remaining_budget >= 0;
    let budget_class = if remaining_budget < 0 {
        "budget over"
    } else {
        "budget ok"
    };

    let on_tab = |category_id: &str| {
        let state = state.clone();
        let cat_id = category_id.to_string();
        Callback::from(move |_| {
            set_screen(&state, StoreScreen::Category(cat_id.clone()));
        })
    };

    html! {
        <main class="outfitting-store">
            <section role="region" aria-labelledby="store-title" onkeydown={on_keydown} class="store-shell">
                <header class="store-header">
                    <div>
                        <h1 id="store-title">{ title.clone() }</h1>
                    </div>
                    <div class={classes!("store-budget", budget_class)}>
                        <span class="label">{ i18n::t("store.budget") }</span>
                        <span class="value">{ budget_str.clone() }</span>
                    </div>
                </header>
                <nav class="store-tabs" aria-label={i18n::t("store.title")}>
                    {
                        state.store_data.categories.iter().enumerate().map(|(i, cat)| {
                            let idx = u8::try_from(i + 1).unwrap_or_default();
                            let focused = state.focus_idx == idx;
                            html! {
                                <button
                                    class="store-tab"
                                    data-key={idx.to_string()}
                                    aria-pressed="false"
                                    tabindex={if focused { "0" } else { "-1" }}
                                    onclick={on_tab(&cat.id)}>
                                    { i18n::t(&format!("store.categories.{}", cat.id)) }
                                </button>
                            }
                        }).collect::<Html>()
                    }
                </nav>
                <ul role="menu" aria-label={i18n::t("store.title")} ref={list_ref} class="store-menu">
                    { for categories.iter().enumerate().map(|(i, (idx, label))| {
                        let focused = state.focus_idx == *idx;
                        let disabled = *idx == 0 && !can_continue;
                        let posinset = u8::try_from(i).unwrap_or_default().saturating_add(1);

                        html!{
                            <li role="menuitem"
                                tabindex={if focused && !disabled { "0" } else { "-1" }}
                                data-key={idx.to_string()}
                                aria-posinset={posinset.to_string()}
                                aria-setsize="6"
                                aria-disabled={disabled.to_string()}
                                class={classes!("ot-menuitem", disabled.then_some("disabled"))}>
                                <span class="num">{ format!("{})", idx) }</span>
                                <span class="label">{ label.clone() }</span>
                            </li>
                        }
                    }) }
                </ul>
                <div aria-live="polite" aria-atomic="true" class="sr-only" id="store-status"></div>
            </section>
        </main>
    }
}
