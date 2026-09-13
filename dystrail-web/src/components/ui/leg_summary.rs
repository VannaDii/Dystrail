//! Destination and vehicle readouts shared by the HUD and outfitting review.
use super::context_help::ContextHelp;
use super::stat_card::StatCard;
use crate::{
    game::{GameState, journal::JournalEntry, route},
    i18n,
};
use std::collections::BTreeMap;
use yew::prelude::*;

#[cfg(test)]
mod tests;

pub fn render(gs: &GameState) -> Html {
    html! {<section class="leg-summary" aria-label={i18n::t("journey.next_leg")}>
        {destination(gs)}{resources(gs)}
    </section>}
}
pub fn render_hud_resources(gs: &GameState) -> Html {
    html! {<div class="hud-trip-context">{resources(gs)}</div>}
}
pub fn render_hud_destination(gs: &GameState) -> Html {
    html! {<div class="hud-trip-context">{destination(gs)}</div>}
}
fn destination(gs: &GameState) -> Html {
    let town = gs
        .continuity
        .route_services
        .stop
        .and_then(|mile| route::settlement(gs, mile))
        .or_else(|| route::upcoming(gs).next());
    let miles = route::physical_miles(gs);
    html! {<div class="leg-destination">{super::journey_icon::render("location")}<span>{i18n::t("journey.next_stop")}</span><strong>{town.map_or("D.C.",|s|s.name.as_str())}</strong>
        if gs.continuity.route_services.stop.is_none() {<small>{town.map_or_else(String::new,|s|super::route_map::location::distance((f32::from(s.mile)-miles).max(0.0)))}</small>}
    </div>}
}
fn resources(gs: &GameState) -> Html {
    html! {<div class="leg-resources">
        <span class="leg-indicator leg-cash">{super::journey_icon::render("cash")}<span class="sr-only">{i18n::t("ux.cash")}</span><strong><bdi>{i18n::fmt_currency(gs.budget_cents)}</bdi></strong></span>
        <span class="leg-indicator leg-vehicle">{super::journey_icon::render("vehicle")}<span class="sr-only">{i18n::t("play.vehicle")}</span><strong><bdi>{format!("{:.0}%",gs.vehicle.health)}</bdi></strong></span>
        <ContextHelp title={i18n::t("trail.supply_outlook")}>{outlook(gs)}</ContextHelp></div>
    }
}

#[derive(Debug, PartialEq)]
enum SupplyOutlook {
    Waiting,
    Stable,
    Days(f64),
}

struct DailyStock {
    opening: i32,
    closing: i32,
    traveled: bool,
}

fn accounting_day(entry: &JournalEntry) -> u32 {
    // Older saves stamp a completed driving hour at the next day's morning.
    // Its elapsed-day receipt preserves the day on which the action began.
    entry
        .resources
        .iter()
        .find(|r| r.key == "play.elapsed" && r.before < r.after)
        .and_then(|r| u32::try_from(r.before).ok())
        .filter(|day| *day <= entry.day)
        .unwrap_or(entry.day)
}

fn supply_outlook(gs: &GameState) -> SupplyOutlook {
    let mut days = BTreeMap::<u32, DailyStock>::new();
    for entry in &gs.continuity.journal {
        let day = accounting_day(entry);
        if day >= gs.day {
            continue;
        }
        let stock = days.entry(day).or_insert(DailyStock {
            opening: entry.before.supplies,
            closing: entry.after.supplies,
            traveled: false,
        });
        // Use the day's actual endpoints: summing hourly snapshots can charge
        // overlapping costs twice and overlook replenishment between actions.
        stock.closing = entry.after.supplies;
        stock.traveled |= entry.action_kind == "travel";
    }
    let (count, used) = days.values().rev().filter(|day| day.traveled).take(8).fold(
        (0_u32, 0.0_f64),
        |(count, used), day| {
            (
                count + 1,
                used + (f64::from(day.opening) - f64::from(day.closing)).max(0.0),
            )
        },
    );
    if count == 0 {
        SupplyOutlook::Waiting
    } else if gs.stats.supplies <= 0 {
        SupplyOutlook::Days(0.0)
    } else if used <= 0.0 {
        SupplyOutlook::Stable
    } else {
        SupplyOutlook::Days((f64::from(gs.stats.supplies.max(0)) * f64::from(count) / used).floor())
    }
}

fn outlook(gs: &GameState) -> Html {
    let body = match supply_outlook(gs) {
        SupplyOutlook::Waiting => html! {<p>{i18n::t("journey.forecast_wait")}</p>},
        SupplyOutlook::Stable => html! {<p>{i18n::t("journey.forecast_stable")}</p>},
        SupplyOutlook::Days(days) => {
            let card = StatCard {
                key: "journey.forecast_days".into(),
                value: i18n::fmt_number(days),
                subtitle: i18n::t("journey.forecast_basis"),
                tone: "neutral",
            };
            html! {<>
                <ul class="resource-changes">{card.render()}</ul>
                <p>{i18n::t("journey.forecast_caveat")}</p>
            </>}
        }
    };
    html! {<div class="impact-details supply-outlook">{body}</div>}
}
