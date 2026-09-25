//! Illustrated inventory keeps every carried item and crew member within easy reach.
use crate::{components::ui::stat_card::StatCard, game::GameState, i18n};
use yew::prelude::*;

fn item_art(id: &str) -> Html {
    html! {<img class="van-item-art" src={crate::paths::asset_path(&format!("static/img/items/{id}-v1.png"))} alt="" width="64" height="64" decoding="sync" />}
}

fn item(id: &str, amount: String, missing: bool) -> Html {
    html! {<li class={classes!("van-item",missing.then_some("van-item-missing"))} data-item={id.to_owned()}>
        {item_art(id)}<div class="van-item-copy"><h3>{i18n::t(&format!("store.items.{id}.name"))}</h3><p>{i18n::t(&format!("store.items.{id}.desc"))}</p></div>
        <strong class="van-item-count">{amount}</strong>
    </li>}
}

fn crew(gs: &GameState) -> Html {
    use crate::game::party::MemberStatus;
    html! {<section class="van-crew"><h2>{if gs.party.name.is_empty(){i18n::t("crew.roster")}else{gs.party.name.clone()}}</h2>
        <ul class="van-crew-grid">{for gs.party.members.iter().map(|m| {
            let status=match m.status {
                MemberStatus::Active=>if gs.continuity.crew_care.strain.get(&m.persona).copied().unwrap_or(0)>0 {"journey.struggling"}else{"crew.active"},
                MemberStatus::Dead=>"crew.dead",MemberStatus::Departed=>"crew.departed",
            };
            html! {<li class={classes!("van-crew-member",(status!="crew.active").then_some("van-crew-attention"))}>
                <span class="van-roster-art">{super::super::cast_art::art(&m.persona, if status == "journey.struggling" { super::super::cast_art::Pose::Unwell } else { super::super::cast_art::Pose::Standard })}</span>
                <div><h3>{&m.name}</h3><p>{i18n::t(&format!("persona.{}.name",m.persona))}</p><span>{i18n::t(status)}</span></div>
            </li>}
        })}</ul>
    </section>}
}

pub fn render_status(gs: &GameState) -> Html {
    let stats = [
        (
            "ux.supplies",
            i18n::fmt_number(f64::from(gs.stats.supplies)),
            "static/img/items/rations-v1.png",
        ),
        (
            "play.cash",
            i18n::fmt_currency(gs.budget_cents),
            "static/img/status/cash-v1.png",
        ),
        (
            "play.vehicle",
            format!("{:.0}%", gs.vehicle.health),
            "static/img/status/vehicle-v2.png",
        ),
        (
            "play.miles",
            i18n::fmt_number(f64::from(crate::game::route::physical_miles(gs)).round()),
            "static/img/status/distance-v1.png",
        ),
    ];
    html! {<section class="van-inventory" aria-label={i18n::t("play.inventory")}>
        <section class="van-summary" aria-labelledby="van-summary-title"><h2 id="van-summary-title">{i18n::t("play.van_summary")}</h2>
            <ul class="resource-changes van-overview">{for stats.into_iter().map(|(key,value,asset)|StatCard{key:key.into(),value,subtitle:String::new(),tone:"neutral"}.render_illustrated(asset))}</ul>
        </section>
        {crew(gs)}
        <div class="van-equipment-grid">
            <section><h2>{i18n::t("play.spares")}<super::super::context_help::ContextHelp title={i18n::t("play.spares")} text={i18n::t("play.vehicle_help")} /></h2>
                <ul class="van-item-grid">{for [("spare_tire",gs.inventory.spares.tire),("battery",gs.inventory.spares.battery),("alternator",gs.inventory.spares.alt),("fuel_pump",gs.inventory.spares.pump)].into_iter().map(|(id,n)|item(id,i18n::fmt_number(f64::from(n)),n==0))}</ul>
            </section>
            <section><h2>{i18n::t("play.equipment")}</h2><ul class="van-item-grid">
                {for [("plague_resist","masks"),("cold_resist","coats"),("rain_resist","ponchos"),("permit","press_pass"),("water_jugs","water")].into_iter().map(|(tag,id)|{
                    let carried=gs.inventory.tags.contains(tag);
                    item(id,i18n::t(if carried {"weather.panel.carried"}else{"weather.panel.missing"}),!carried)
                })}
            </ul></section>
        </div>
    </section>}
}
