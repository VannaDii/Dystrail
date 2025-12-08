use crate::components::ui::stats_bar::weather_symbol;
use crate::game::weather::{Weather, WeatherConfig, WeatherEffect};
use crate::game::{DietId, GameState, PaceId, PacingConfig, Region};
use crate::i18n;
use yew::prelude::*;

pub(super) fn render_weather_info(game_state: &GameState) -> Html {
    let weather_cfg = WeatherConfig::default_config();
    let today = game_state.weather_state.today;
    let icon = weather_symbol(today);

    let effect = weather_cfg.effects.get(&today);
    let effects_text = effect.map_or_else(String::new, |eff| {
        let mut parts = Vec::new();

        if eff.supplies != 0 {
            parts.push(format_delta("Sup", eff.supplies));
        }
        if eff.sanity != 0 {
            parts.push(format_delta("San", eff.sanity));
        }
        if eff.pants != 0 {
            parts.push(format_delta("Pants", eff.pants));
        }
        if eff.enc_delta != 0.0 {
            parts.push(format_percent("Enc", eff.enc_delta));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!(" ({parts})", parts = parts.join(", "))
        }
    });

    let weather_state_name = i18n::t(today.i18n_key());
    let region_name = i18n::t(match game_state.region {
        Region::Heartland => "region.heartland",
        Region::RustBelt => "region.rustbelt",
        Region::Beltway => "region.beltway",
    });

    html! {
        <section class="weather-strip" role="region" aria-labelledby="weather-title">
            <h3 id="weather-title" class="sr-only">{ i18n::t("weather.title") }</h3>
            <p class="weather-info"
               tabindex="0"
               role="button"
               aria-expanded="false"
               onclick={Callback::from(move |_| {})}>
                <span class="weather-icon" aria-hidden="true">{ icon }</span>
                <span class="day-region">
                    { format!("Day {day} — Region: {region}", day = game_state.day, region = region_name) }
                </span>
                <br />
                <span class="weather-state">
                    { format!("Weather: {weather}{effects}", weather = weather_state_name, effects = effects_text) }
                </span>
            </p>

            <div aria-live="polite" aria-atomic="true" class="sr-only weather-announce">
                { format_weather_announcement(today, effect) }
            </div>
        </section>
    }
}

pub(super) fn render_weather_details(game_state: &GameState) -> Html {
    let weather_cfg = WeatherConfig::default_config();
    let today = game_state.weather_state.today;
    let weather_state_name = i18n::t(today.i18n_key());

    let effect = weather_cfg.effects.get(&today);
    let mitigation = weather_cfg.mitigation.get(&today);

    let announcement = format_weather_announcement(today, effect);

    html! {
        <section class="weather-details" aria-label={i18n::t("weather.details.header")}>
            <h4>{ weather_state_name }</h4>
            <ul>
                {
                    effect.map_or_else(
                        || html! { <li>{ i18n::t("weather.effects.none") }</li> },
                        |eff| html! {
                            <>
                                <li>{ format_delta(&i18n::t("stats.supplies"), eff.supplies) }</li>
                                <li>{ format_delta(&i18n::t("stats.sanity"), eff.sanity) }</li>
                                <li>{ format_delta(&i18n::t("stats.pants"), eff.pants) }</li>
                                <li>{ format_percent(&i18n::t("weather.details.encounter"), eff.enc_delta) }</li>
                                <li>{ format_percent(&i18n::t("weather.details.travel"), eff.travel_mult - 1.0) }</li>
                            </>
                        }
                    )
                }
            </ul>
            <p class="mitigation">
                { mitigation.map_or_else(
                    || html! { <span>{ i18n::t("weather.details.no_mitigation") }</span> },
                    |mit| {
                        let sanity = mit.sanity.unwrap_or(0);
                        let pants = mit.pants.unwrap_or(0);
                        html! { <span>{ format!("Mitigation tag: {} ({:+} sanity, {:+} pants)", mit.tag, sanity, pants) }</span> }
                    },
                )}
            </p>
            <p class="sr-only">{ announcement }</p>
        </section>
    }
}

pub(super) const fn pace_code(pace: PaceId) -> &'static str {
    match pace {
        PaceId::Steady => "S",
        PaceId::Heated => "H",
        PaceId::Blitz => "B",
    }
}

pub(super) const fn diet_code(diet: DietId) -> &'static str {
    match diet {
        DietId::Mixed => "M",
        DietId::Quiet => "Q",
        DietId::Doom => "D",
    }
}

pub(super) fn pace_preview(pacing_config: &PacingConfig, pace: PaceId) -> String {
    pacing_config
        .pace
        .iter()
        .find(|p| p.id == pace.as_str())
        .map_or_else(String::new, |p| {
            format!(
                "{} | {}: {} {}: {}",
                p.name,
                i18n::t("stats.pants"),
                p.pants,
                i18n::t("stats.sanity_short"),
                p.sanity
            )
        })
}

pub(super) fn diet_preview(pacing_config: &PacingConfig, diet: DietId) -> String {
    pacing_config
        .diet
        .iter()
        .find(|d| d.id == diet.as_str())
        .map_or_else(String::new, |d| {
            format!(
                "{} | {}: {} {}: {}",
                d.name,
                i18n::t("stats.pants"),
                d.pants,
                i18n::t("stats.sanity_short"),
                d.sanity
            )
        })
}

pub(super) fn format_delta(stat: &str, value: i32) -> String {
    format!("{stat} {value:+}")
}

pub(super) fn format_percent(stat: &str, value: f32) -> String {
    format!("{stat} {:+.0}%", value * 100.0)
}

pub(super) fn format_weather_announcement(
    today: Weather,
    effects: Option<&WeatherEffect>,
) -> String {
    let mut parts = Vec::new();
    parts.push(format!("Weather: {}", i18n::t(today.i18n_key())));

    if let Some(effect) = effects {
        if effect.supplies != 0 {
            parts.push(format_delta("Supplies", effect.supplies));
        }
        if effect.sanity != 0 {
            parts.push(format_delta("Sanity", effect.sanity));
        }
        if effect.pants != 0 {
            parts.push(format_delta("Pants", effect.pants));
        }
        if effect.enc_delta != 0.0 {
            parts.push(format_percent("Encounter", effect.enc_delta));
        }
        if (effect.travel_mult - 1.0).abs() > f32::EPSILON {
            parts.push(format_percent("Travel", effect.travel_mult - 1.0));
        }
    }

    parts.join(", ")
}
