use crate::{game::GameState, i18n};
use yew::prelude::*;
pub fn render_status(gs: &GameState) -> Html {
    html! {<section class="status-sheet" aria-label={i18n::t("play.inventory")}>
        <div><h2>{i18n::t("play.spares")}</h2><p class="muted">{i18n::t("play.vehicle_help")}</p><dl class="inventory-list">
        {for [("spare_tire",gs.inventory.spares.tire),("battery",gs.inventory.spares.battery),("alternator",gs.inventory.spares.alt),("fuel_pump",gs.inventory.spares.pump)].into_iter().map(|(id,n)|html!{<div><dt>{i18n::t(&format!("store.items.{id}.name"))}</dt><dd>{n}</dd></div>})}
        </dl></div>
        <div><h2>{i18n::t("play.equipment")}</h2><ul class="equipment-list">
        {for [("plague_resist","masks"),("cold_resist","coats"),("rain_resist","ponchos"),("permit","press_pass"),("water_jugs","water")].into_iter().filter(|(tag,_)|gs.inventory.tags.contains(*tag)).map(|(_,id)|html!{<li><strong>{i18n::t(&format!("store.items.{id}.name"))}</strong><span>{i18n::t(&format!("store.items.{id}.desc"))}</span></li>})}
        if gs.inventory.tags.is_empty(){<li>{i18n::t("play.none")}</li>}
        </ul><dl class="inventory-list"><div><dt>{i18n::t("play.cash")}</dt><dd>{i18n::fmt_currency(gs.budget_cents)}</dd></div><div><dt>{i18n::t("play.vehicle")}</dt><dd>{format!("{:.0}%",gs.vehicle.health)}</dd></div><div><dt>{i18n::t("play.miles")}</dt><dd>{i18n::fmt_number(f64::from(crate::game::route::physical_miles(gs)).round())}</dd></div></dl></div>
        <div class="crew-roster"><h2>{if gs.party.name.is_empty(){i18n::t("crew.roster")}else{gs.party.name.clone()}}</h2><dl class="inventory-list">{for gs.party.members.iter().map(|m|html!{<div><dt><strong>{&m.name}</strong><span>{i18n::t(&format!("persona.{}.name",m.persona))}</span></dt><dd>{i18n::t(match m.status{crate::game::party::MemberStatus::Active=>if gs.continuity.crew_care.strain.get(&m.persona).copied().unwrap_or(0)>0{"journey.struggling"}else{"crew.active"},crate::game::party::MemberStatus::Dead=>"crew.dead",crate::game::party::MemberStatus::Departed=>"crew.departed"})}</dd></div>})}</dl></div>
    </section>}
}
