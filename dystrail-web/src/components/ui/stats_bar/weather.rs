//! One weather indicator with structured, read-only impact details.
use super::{WeatherBadge, helpers};
use crate::{
    components::ui::stat_card::StatCard,
    game::{GameState, Weather, WeatherConfig},
    i18n,
};
use yew::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct Readout {
    pub cost: String,
    pub applied_day: Option<u32>,
    pub changes: Vec<StatCard>,
    pub rates: Vec<StatCard>,
    pub protection: Option<(String, bool, String)>,
}

#[must_use]
pub fn readout(gs: &GameState, cfg: &WeatherConfig) -> Readout {
    let weather = gs.weather_state.today;
    let recorded = gs
        .continuity
        .weather_impact
        .as_ref()
        .filter(|impact| impact.weather == weather);
    let changes: Vec<_> = recorded
        .into_iter()
        .flat_map(|impact| {
            [
                ("ux.supplies", impact.supplies),
                ("ux.health", impact.hp),
                ("ux.sanity", impact.sanity),
            ]
        })
        .filter(|(_, value)| *value != 0)
        .map(|(key, value)| StatCard {
            key: key.into(),
            value: format!("{value:+}"),
            subtitle: String::new(),
            tone: if value < 0 { "harmful" } else { "helpful" },
        })
        .collect();
    let rates: Vec<_> = cfg
        .effects
        .get(&weather)
        .into_iter()
        .flat_map(|effect| {
            [
                ("play.distance", effect.travel_mult - 1.0, false),
                ("play.risk", effect.enc_delta, true),
            ]
        })
        .filter(|(_, value, _)| value.abs() > f32::EPSILON)
        .map(|(key, value, inverted)| StatCard {
            key: key.into(),
            value: format!("{:+.0}%", value * 100.0),
            subtitle: String::new(),
            tone: if (value < 0.0) == inverted {
                "helpful"
            } else {
                "harmful"
            },
        })
        .collect();
    let cost = if !changes.is_empty() {
        compact(&changes)
    } else if recorded.is_some() && !rates.is_empty() {
        compact(&rates)
    } else {
        i18n::t(if recorded.is_some() || weather == Weather::Clear {
            "weather.hud.no_cost"
        } else {
            "weather.hud.pending"
        })
    };
    let gear = match weather {
        Weather::HeatWave => Some(("water", "water_jugs", "water", "weather.panel.water")),
        Weather::ColdSnap => Some(("coats", "cold_resist", "warm_coat", "weather.panel.coats")),
        Weather::Smoke => Some((
            "masks",
            "plague_resist",
            "plague_resist",
            "weather.panel.masks",
        )),
        Weather::Storm => Some((
            "ponchos",
            "rain_resist",
            "rain_resist",
            "weather.panel.ponchos",
        )),
        Weather::Clear => None,
    };
    Readout {
        cost,
        applied_day: recorded.map(|impact| impact.day),
        changes,
        rates,
        protection: gear.map(|(item, tag, alias, note)| {
            (
                i18n::t(&format!("store.items.{item}.name")),
                gs.inventory.tags.contains(tag) || gs.inventory.tags.contains(alias),
                i18n::t(note),
            )
        }),
    }
}

fn compact(cards: &[StatCard]) -> String {
    cards
        .iter()
        .map(|card| format!("{} {}", i18n::t(&card.key), card.value))
        .collect::<Vec<_>>()
        .join(" · ")
}

pub fn details(readout: &Readout) -> Html {
    let heading = readout.applied_day.map_or_else(
        || i18n::t("weather.panel.latest"),
        |day| {
            i18n::tr(
                "weather.panel.applied",
                Some(&std::collections::BTreeMap::from([(
                    "day",
                    i18n::fmt_number(f64::from(day)).as_str(),
                )])),
            )
        },
    );
    html! {<div class="weather-details impact-details">
        <section><h3>{heading}</h3>
            if readout.changes.is_empty() {<p>{i18n::t(if readout.applied_day.is_some() {"weather.hud.no_cost"} else {"weather.panel.pending"})}</p>}
            else {<ul class="resource-changes">{for readout.changes.iter().map(StatCard::render)}</ul>}
            <p class="weather-explainer">{i18n::t("weather.panel.accounting")}</p>
        </section>
        <section><h3>{i18n::t("weather.panel.modifiers")}</h3>
            if readout.rates.is_empty() {<p>{i18n::t("weather.panel.no_modifiers")}</p>}
            else {<ul class="resource-changes">{for readout.rates.iter().map(StatCard::render)}</ul>}
        </section>
        if let Some((item, carried, note)) = &readout.protection {<section class="weather-protection"><h3>{i18n::t("weather.panel.protection")}</h3><div><strong>{item}</strong><span class={if *carried {"gear-carried"}else{"gear-missing"}}>{i18n::t(if *carried {"weather.panel.carried"}else{"weather.panel.missing"})}</span></div><p>{note}</p></section>}
    </div>}
}

#[derive(Properties, PartialEq, Eq)]
pub struct Props {
    pub badge: WeatherBadge,
    pub readout: Option<Readout>,
}

#[function_component(WeatherIndicator)]
pub fn weather_indicator(p: &Props) -> Html {
    crate::i18n::use_language();
    let notifying = use_context::<crate::app::weather_status::WeatherNotification>()
        .unwrap_or_default()
        .0;
    let name = i18n::t(p.badge.weather.i18n_key());
    html! {<div class={classes!("weather-indicator", notifying.then_some("weather-changed"))} data-weather={format!("{:?}", p.badge.weather)} role="status" aria-live="polite" aria-atomic="true">
        if notifying {<span class="sr-only">{i18n::t("journey.conditions_changed")}{": "}</span>}
        <span class={classes!("weather-name", helpers::weather_sprite_class(p.badge.weather))}><span class="weather-icon" aria-hidden="true">{helpers::weather_symbol(p.badge.weather)}</span><strong>{name}</strong></span>
        if let Some(readout) = &p.readout {
            <span class="sr-only">{&readout.cost}</span>
            <crate::components::ui::context_help::ContextHelp informational={true} icon={"ⓘ".to_owned()} title={i18n::t("play.weather_effects")} >{details(readout)}</crate::components::ui::context_help::ContextHelp>
        }
    </div>}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storm_readout_explains_ponchos_and_retains_other_weather_costs() {
        i18n::set_lang("en");
        let cfg = WeatherConfig::default_config();
        for carried in [false, true] {
            let mut gs = GameState::default();
            gs.weather_state.today = Weather::Storm;
            gs.stats.sanity = 7;
            if carried {
                gs.inventory.tags.insert("rain_resist".into());
            }
            crate::game::weather::apply_weather_effects(&mut gs, &cfg);
            let before = serde_json::to_value(&gs).unwrap();
            let result = readout(&gs, &cfg);
            assert_eq!(
                result.protection,
                Some((
                    "Ponchos".into(),
                    carried,
                    "Ponchos prevent storm sanity loss. Other storm costs still apply.".into(),
                ))
            );
            assert_eq!(result.changes[0].key, "ux.supplies");
            assert_eq!(result.changes[0].value, "-1");
            assert_eq!(
                result
                    .changes
                    .iter()
                    .find(|card| card.key == "ux.sanity")
                    .map(|card| card.value.as_str()),
                if carried { None } else { Some("-1") },
            );
            assert_eq!(result.rates[0].key, "play.distance");
            assert_eq!(result.rates[0].value, "-10%");
            assert_eq!(result.rates[1].key, "play.risk");
            assert_eq!(result.rates[1].value, "+5%");
            assert_eq!(serde_json::to_value(&gs).unwrap(), before);
        }
    }
}
