use super::super::state::StoreState;
use crate::{
    game::{GameState, store::StoreItem, store::calculate_effective_price},
    i18n,
};
use std::collections::BTreeMap;
use yew::prelude::*;

pub fn render_store_item_card(
    item: &StoreItem,
    state: &UseStateHandle<StoreState>,
    gs: &GameState,
    resupply: bool,
) -> Html {
    let price = i18n::fmt_currency(calculate_effective_price(
        item.price_cents,
        state.discount_pct,
    ));
    let grants = super::super::planner::grant_text(&item.grants);
    let spare = match item.id.as_str() {
        "spare_tire" => Some(gs.inventory.spares.tire),
        "battery" => Some(gs.inventory.spares.battery),
        "alternator" => Some(gs.inventory.spares.alt),
        "fuel_pump" => Some(gs.inventory.spares.pump),
        _ => None,
    };
    let owned = spare.filter(|_| resupply).map_or_else(
        || {
            if resupply
                && !item.tags.is_empty()
                && item.tags.iter().all(|tag| gs.inventory.tags.contains(tag))
            {
                i18n::t("play.owned")
            } else {
                String::new()
            }
        },
        |count| {
            i18n::tr(
                "shopping.in_van",
                Some(&BTreeMap::from([("count", count.to_string().as_str())])),
            )
        },
    );
    html! {<article key={item.id.clone()} role="group" aria-labelledby={format!("store-item-{}",item.id)}
        class={classes!("store-card",(state.cart.get_quantity(&item.id)>0).then_some("store-item-selected"))}>
        <img class="store-item-art" src={crate::paths::asset_path(&format!("static/img/items/{}-v1.png",item.id))} alt="" width="72" height="72" decoding="sync" />
        <div class="store-card-body">
            <h3 id={format!("store-item-{}",item.id)}>{i18n::t(&format!("store.items.{}.name",item.id))}</h3>
            <p class="store-item-detail"><span>{i18n::t(&format!("store.items.{}.desc",item.id))}</span>
                if !grants.is_empty() {<span class="item-grants">{grants}</span>}
                if !owned.is_empty() {<span class="store-owned">{owned}</span>}
            </p>
        </div>
        <span class="store-price">{i18n::tr("shopping.each",Some(&BTreeMap::from([("price",price.as_str())])))}</span>
        {super::item_quantity::render(item,state,gs.budget_cents)}
    </article>}
}
