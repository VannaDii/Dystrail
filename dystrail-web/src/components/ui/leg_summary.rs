//! Compact navigation context at the foot of a scene, with detail available on demand.
use super::context_help::ContextHelp;
use crate::{
    game::{GameState, route},
    i18n,
};
use yew::prelude::*;

pub fn services() -> Html {
    html! {<span class="service-indicators">
        <ContextHelp icon={"▣".to_owned()} title={i18n::t("play2.resupply")} text={i18n::t("trail.shop_help")} />
        <ContextHelp icon={"⇄".to_owned()} title={i18n::t("play2.exchange")} text={i18n::t("trail.trade_help")} />
    </span>}
}
pub fn render(gs: &GameState) -> Html {
    render_bar(gs, None)
}
pub fn render_scene(gs: &GameState, moving: bool) -> Html {
    render_bar(gs, Some(moving))
}
fn render_bar(gs: &GameState, moving: Option<bool>) -> Html {
    let town = route::upcoming(gs).next();
    let miles = route::physical_miles(gs);
    html! {<section class="leg-summary" aria-label={i18n::t("journey.next_leg")}>
        if let Some(moving)=moving {<span class="leg-turn">{i18n::t(if moving{"play.traveling"}else{"play.your_turn"})}</span>}
        <div class="leg-destination"><span>{i18n::t("journey.next_stop")}</span><strong>{town.map_or("D.C.",|s|s.name.as_str())}</strong><small>{town.map_or_else(String::new,|s|super::route_map::location::distance((f32::from(s.mile)-miles).max(0.0)))}</small>
        if town.is_some_and(|t|t.name!="D.C.") {{services()}}
        </div>
        <div class="leg-resources"><strong>{i18n::fmt_currency(gs.budget_cents)}</strong><span>{format!("{} {:.0}%",i18n::t("play.vehicle"),gs.vehicle.health)}</span>
        <ContextHelp title={i18n::t("trail.supply_outlook")} text={outlook(gs)} /></div>
    </section>}
}
fn outlook(gs: &GameState) -> String {
    let days: Vec<_> = gs
        .continuity
        .journal
        .iter()
        .filter(|e| e.title == i18n::t("play.last_turn"))
        .rev()
        .take(8)
        .filter(|e| e.before.supplies > e.after.supplies)
        .collect();
    if days.is_empty() {
        return i18n::t("journey.forecast_wait");
    }
    let used: i32 = days
        .iter()
        .map(|e| e.before.supplies - e.after.supplies)
        .sum();
    let elapsed = gs
        .day
        .saturating_sub(days.last().map_or(gs.day, |e| e.day))
        .max(1);
    let estimate = (f64::from(gs.stats.supplies) * f64::from(elapsed) / f64::from(used)).floor();
    i18n::tr(
        "journey.forecast",
        Some(&std::collections::BTreeMap::from([(
            "days",
            format!("{estimate:.0}").as_str(),
        )])),
    )
}
