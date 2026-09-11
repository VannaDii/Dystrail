use crate::components::ui::{context_help::ContextHelp, stats_bar::weather_symbol};
use crate::{
    game::{GameState, weather::WeatherConfig},
    i18n,
};
use yew::prelude::*;

pub fn render_weather_details(gs: &GameState) -> Html {
    let cfg = WeatherConfig::default_config();
    let today = gs.weather_state.today;
    let effect = cfg.effects.get(&today);
    let mit = cfg
        .mitigation
        .get(&today)
        .filter(|m| gs.inventory.tags.contains(&m.tag));
    let mut rows = Vec::new();
    if let Some(e) = effect {
        for (key, v) in [
            ("ux.supplies", e.supplies),
            ("ux.sanity", mit.and_then(|m| m.sanity).unwrap_or(e.sanity)),
            ("ux.pants", mit.and_then(|m| m.pants).unwrap_or(e.pants)),
        ] {
            if v != 0 {
                rows.push(format_delta(&i18n::t(key), v));
            }
        }
        if (e.travel_mult - 1.0).abs() > f32::EPSILON {
            rows.push(format_percent(
                &i18n::t("play.distance"),
                e.travel_mult - 1.0,
            ));
        }
        if e.enc_delta.abs() > f32::EPSILON {
            rows.push(format_percent(&i18n::t("play.risk"), e.enc_delta));
        }
    }
    html! {<section class="weather-summary" aria-label={i18n::t("play.weather_effects")}>
        <div><span class="condition-icon" aria-hidden="true">{weather_symbol(today)}</span><strong>{i18n::t(today.i18n_key())}</strong><ContextHelp title={i18n::t("play.weather_effects")} text={i18n::t("play.weather_help")} /></div>
        <p>{if rows.is_empty(){i18n::t("play.no_weather")}else{rows.join(" · ")}}</p>
        if cfg.mitigation.contains_key(&today) {<span class="protection-status">{i18n::t(if mit.is_some(){"play.protected"}else{"play.unprotected"})}</span>}
    </section>}
}
pub(super) fn format_delta(stat: &str, v: i32) -> String {
    format!("{stat} {v:+}")
}
pub(super) fn format_percent(stat: &str, v: f32) -> String {
    format!("{stat} {:+.0}%", v * 100.0)
}
#[cfg(test)]
pub(super) fn format_weather_announcement(
    today: crate::game::weather::Weather,
    e: Option<&crate::game::weather::WeatherEffect>,
) -> String {
    format!(
        "Weather: {}. {}",
        i18n::t(today.i18n_key()),
        e.map_or_else(String::new, |v| format_delta("Sanity", v.sanity))
    )
}
