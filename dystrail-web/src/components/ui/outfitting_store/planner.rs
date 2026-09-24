use super::{
    state::{OutfittingStoreProps, StoreState},
    view::item_card::render_store_item_card,
};
use crate::{components::ui::context_help::ContextHelp, game::store::Grants, i18n};
use yew::prelude::*;

pub fn render(
    state: &UseStateHandle<StoreState>,
    p: &OutfittingStoreProps,
    review: &UseStateHandle<bool>,
) -> Html {
    let on_review = {
        let review = review.clone();
        Callback::from(move |_| review.set(!*review))
    };
    html! {<section class={classes!("outfit-workspace",p.resupply.then_some("resupply-workspace"))} aria-labelledby="store-title">
        <header class="outfit-heading">
            <div class="outfit-title-row"><h1 id="store-title" tabindex="-1">{i18n::t(if **review{"play.loadout"}else{"play.outfit"})}<ContextHelp title={i18n::t("play.outfit")} text={i18n::t(if p.resupply {"play.resupply_help"} else {"play.outfit_help"})} /></h1>
            if p.resupply {<button class="leave-store" onclick={{let close=p.on_close.clone();Callback::from(move |_|close.emit(()))}}>{i18n::t("play2.leave_shop")}</button>}</div>
            if !p.resupply {<p>{i18n::t(if **review{"play.review_help"}else{"play.pack_empty"})}</p>}
            if **review && !p.resupply {if let Some(unit)=crate::app::visual_content::departure_unit(&p.game_state) {
                <p class="departure-intro" data-departure-unit={unit.clone()}><strong>{i18n::t(&format!("encounter_copy.{unit}.name"))}{". "}</strong>{i18n::t(&format!("encounter_copy.{unit}.desc"))}</p>
            }}
        </header>
        if !p.resupply {{crate::components::ui::leg_summary::render(&p.game_state)}}
        <div class="outfit-catalog">
            <span id="store-quantity-label" class="sr-only">{i18n::t("shopping.quantity")}</span>
            if **review && state.cart.lines.is_empty() {<p class="empty-cart">{i18n::t("ux.empty")}</p>}
            {for state.store_data.categories.iter().filter(|c|!**review||c.items.iter().any(|item|state.cart.get_quantity(&item.id)>0)).map(|cat|html!{<section class="outfit-category" key={cat.id.clone()}>
                <h2>{i18n::t(&format!("store.categories.{}",cat.id))}<ContextHelp title={i18n::t(&format!("store.categories.{}",cat.id))} text={i18n::t(&format!("play.{}_help",cat.id))} /></h2>
                <div class="store-item-grid">{for cat.items.iter().filter(|item|!**review||state.cart.get_quantity(&item.id)>0).map(|item|render_store_item_card(item,state,&p.game_state,p.resupply))}</div>
            </section>})}
        </div>
        {super::summary::render(state,p,**review,on_review)}
    </section>}
}

pub(super) fn grant_text(g: &Grants) -> String {
    let mut rows = Vec::new();
    for (key, n) in [
        ("ux.supplies", g.supplies),
        ("play.credibility", g.credibility),
    ] {
        if n != 0 {
            rows.push(format!("{} {n:+}", i18n::t(key)));
        }
    }
    rows.join(" · ")
}
