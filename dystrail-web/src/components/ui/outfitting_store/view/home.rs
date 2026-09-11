use super::super::handlers::format_currency;
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
    ];

    let remaining_budget = game_state.budget_cents - state.cart.total_cents;
    let can_continue = remaining_budget >= 0;
    let budget_class = if remaining_budget < 0 {
        "budget over"
    } else {
        "budget ok"
    };

    html! {
        <div class="outfitting-store">
            <section role="region" aria-labelledby="store-title" onkeydown={on_keydown} class="store-shell">
                <header class="store-header">
                    <div>
                        <h1 id="store-title">{ title.clone() }</h1>
                    </div>
                    <div class={classes!("store-budget", budget_class)}>
                        <span class="label">{ i18n::t("ux.budget") }</span>
                        <span class="value">{ budget_str.clone() }</span>
                    </div>
                </header>
                <p class="store-intro">{i18n::t("ux.prepare_help")}</p>
                <ul role="menu" aria-label={i18n::t("store.title")} ref={list_ref} class="store-menu">
                    { for categories.iter().enumerate().map(|(i, (idx, label))| {
                        let focused = state.focus_idx == *idx;
                        let disabled = *idx == 0 && !can_continue;
                        let posinset = u8::try_from(i).unwrap_or_default().saturating_add(1);

                        html!{
                            <li role="menuitem"
                                onclick={{ let handle = state.clone(); let target = *idx; Callback::from(move |_| { let screen = match target { 1 => StoreScreen::Category("fuel_food".into()), 2 => StoreScreen::Category("vehicle".into()), 3 => StoreScreen::Category("ppe".into()), 4 => StoreScreen::Category("docs".into()), _ => StoreScreen::Cart }; set_screen(&handle, screen); }) }}
                                tabindex={if focused && !disabled { "0" } else { "-1" }}
                                data-key={idx.to_string()}
                                aria-posinset={posinset.to_string()}
                                aria-setsize="5"
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
        </div>
    }
}
