use super::{
    handlers::handle_cart_selection,
    state::{OutfittingStoreProps, StoreState},
    view::item_card::render_store_item_card,
};
use crate::components::ui::context_help::ContextHelp;
use crate::{game::store::Grants, i18n};
use yew::prelude::*;

pub fn render(
    state: &UseStateHandle<StoreState>,
    p: &OutfittingStoreProps,
    category: &UseStateHandle<String>,
    review: &UseStateHandle<bool>,
) -> Html {
    let total = state.cart.total_cents;
    let remaining = p.game_state.budget_cents - total;
    let on_review = {
        let review = review.clone();
        Callback::from(move |_| {
            review.set(!*review);
            if let Some(w) = web_sys::window() {
                w.scroll_to_with_x_and_y(0.0, 0.0);
            }
        })
    };
    let checkout = {
        let state = state.clone();
        let p = p.clone();
        Callback::from(move |_| {
            if state.cart.total_cents <= p.game_state.budget_cents {
                handle_cart_selection(0, &state, &state, &p);
            }
        })
    };
    let added: i32 = state
        .cart
        .lines
        .iter()
        .filter_map(|line| {
            state
                .store_data
                .find_item(&line.item_id)
                .map(|item| item.grants.supplies * line.qty)
        })
        .sum();
    let stocked = p.game_state.stats.supplies + added;
    html! {<section class="outfit-workspace" aria-labelledby="store-title">
        <header class="outfit-heading"><p class="eyebrow">{i18n::t(if p.resupply{"play2.route_stop"}else{"app.title"})}</p><h1 id="store-title" tabindex="-1">{i18n::t(if **review{"play.loadout"}else{"play.outfit"})}<ContextHelp title={i18n::t("play.outfit")} text={i18n::t("play.outfit_help")} /></h1><p>{i18n::t(if **review{"play.review_help"}else if p.resupply{"journey.shop_help"}else{"play.pack_empty"})}</p></header>
        if p.resupply {<button onclick={{let close=p.on_close.clone();Callback::from(move |_|close.emit(()))}}>{i18n::t("play2.leave_shop")}</button>}
        {crate::components::ui::leg_summary::render(&p.game_state)}
        <div class="outfit-columns"><div class="outfit-catalog">
        if !**review {<nav class="category-tabs" aria-label={i18n::t("play.shop")}>
            {for ["all","fuel_food","vehicle","ppe","docs"].into_iter().map(|id|{let category=category.clone();let selected=category.as_str()==id;html!{<button aria-pressed={selected.to_string()} onclick={Callback::from(move |_|category.set(id.to_owned()))}>{i18n::t(&if id=="all"{"play.all".to_owned()}else{format!("store.categories.{id}")})}</button>}})}
        </nav>}
        {for state.store_data.categories.iter().filter(|c|if **review{c.items.iter().any(|item|state.cart.get_quantity(&item.id)>0)}else{category.as_str()=="all"||category.as_str()==c.id}).map(|cat|html!{<section class="outfit-category">
            <h2>{i18n::t(&format!("store.categories.{}",cat.id))}</h2><p class="category-help">{i18n::t(&format!("play.{}_help",cat.id))}</p>
            <div class="store-item-grid">{for cat.items.iter().filter(|item|!**review||state.cart.get_quantity(&item.id)>0).map(|item|render_store_item_card(0,item,state,&p.game_state))}</div>
        </section>})}
        </div><aside class="loadout-panel" aria-label={i18n::t("play.loadout")}>
            <div class="loadout-van" aria-hidden="true">{"▰"}</div><h2>{i18n::t("play.loadout")}</h2>
            <dl class="inventory-list">if p.resupply {<div><dt>{i18n::t("play.base_supplies")}</dt><dd>{p.game_state.stats.supplies}</dd></div>}<div><dt>{i18n::t("play.packed_supplies")}</dt><dd>{stocked.min(20)}</dd></div></dl>
            if stocked>20 {<p class="capacity-warning" role="status">{i18n::t("journey.capacity")}</p>}
            <ul class="loadout-items">{for state.cart.lines.iter().filter_map(|line|state.store_data.find_item(&line.item_id).map(|item|(line,item))).map(|(line,item)|html!{<li><span>{i18n::t(&format!("store.items.{}.name",item.id))}</span><strong>{format!("×{}",line.qty)}</strong></li>})}</ul>
            <dl class="inventory-list store-cart-summary"><div><dt>{i18n::t("play.spent")}</dt><dd class="value">{i18n::fmt_currency(total)}</dd></div><div><dt>{i18n::t("play.cash")}</dt><dd class="cart-total">{i18n::fmt_currency(remaining)}</dd></div></dl>
            if **review {<button class="retro-btn-primary" onclick={checkout} disabled={remaining<0 || stocked>20}>{i18n::t(if p.resupply{"play2.buy_return"}else{"play.depart"})}</button>}
            if !p.resupply {<button class="clear-loadout" onclick={{let state=state.clone();Callback::from(move |_|{let mut next=(*state).clone();next.cart.clear();state.set(next);})}}>{i18n::t("trail.clear_cart")}</button>}
            <button class={if **review{"retro-btn-secondary"}else{"retro-btn-primary"}} onclick={on_review.clone()}>{i18n::t(if **review{"play.edit"}else{"play.review"})}</button>
            <div id="store-status" role="status" class="sr-only"></div>
        </aside></div>
        if !**review {<div class="mobile-packbar"><span>{i18n::t("play.cash")}<strong>{i18n::fmt_currency(remaining)}</strong></span><button class="retro-btn-primary" onclick={on_review}>{i18n::t("play.review")}</button></div>}

    </section>}
}

pub(super) fn grant_text(g: &Grants) -> String {
    let mut rows = Vec::new();
    for (key, n) in [
        ("ux.supplies", g.supplies),
        ("play.credibility", g.credibility),
        ("store.items.spare_tire.name", g.spare_tire),
        ("store.items.battery.name", g.spare_battery),
        ("store.items.alternator.name", g.spare_alt),
        ("store.items.fuel_pump.name", g.spare_pump),
    ] {
        if n != 0 {
            rows.push(format!("{} {n:+}", i18n::t(key)));
        }
    }
    rows.join(" · ")
}
