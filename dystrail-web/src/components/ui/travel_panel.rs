use crate::game::{DietId, GameState, PaceId, PacingConfig};
use crate::i18n;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct Props {
    pub on_travel: Callback<()>,
    pub logs: Vec<String>,
    pub game_state: Option<Rc<GameState>>,
    pub pacing_config: Rc<PacingConfig>,
    pub on_pace_change: Callback<PaceId>,
    pub on_diet_change: Callback<DietId>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        // Compare the relevant fields for re-rendering decisions
        self.logs == other.logs
            && self.game_state.as_ref().map(|gs| (&gs.pace, &gs.diet))
                == other.game_state.as_ref().map(|gs| (&gs.pace, &gs.diet))
    }
}

/// Travel panel component displaying current travel status and progress
///
/// Shows day, region, weather, and provides controls for pace/diet and weather details.
/// Includes accessibility features and keyboard navigation support.
#[function_component(TravelPanel)]
pub fn travel_panel(p: &Props) -> Html {
    let show_pace_diet = use_state(|| false);
    let show_weather_details = use_state(|| false);

    let on_click = {
        let cb = p.on_travel.clone();
        Callback::from(move |_| cb.emit(()))
    };

    let on_show_pace_diet = {
        let show_pace_diet = show_pace_diet.clone();
        Callback::from(move |_| {
            show_pace_diet.set(true);
        })
    };

    let on_hide_pace_diet = {
        let show_pace_diet = show_pace_diet.clone();
        Callback::from(move |()| {
            show_pace_diet.set(false);
        })
    };

    let on_toggle_weather_details = {
        let show_weather_details = show_weather_details.clone();
        Callback::from(move |_| {
            show_weather_details.set(!*show_weather_details);
        })
    };

    // Check if travel is blocked due to vehicle breakdown
    let travel_blocked = p.game_state.as_ref().is_some_and(|gs| gs.travel_blocked);

    // Prepare breakdown message if needed
    let breakdown_msg = p.game_state.as_ref().and_then(|gs| {
        gs.breakdown.as_ref().map(|breakdown| {
            let part_name = i18n::t(breakdown.part.key());
            let mut vars = std::collections::HashMap::new();
            vars.insert("part", part_name.as_str());
            i18n::tr("vehicle.breakdown", Some(&vars))
        })
    });

    // Weather information display
    let weather_info = p
        .game_state
        .as_ref()
        .map_or_else(|| html! {}, |gs| render_weather_info(gs.as_ref()));

    let weather_details = if *show_weather_details {
        p.game_state.as_ref().map_or_else(
            Html::default,
            |game_state| {
                html! {
                    <div class="weather-details-card" role="dialog" aria-labelledby="weather-details-header">
                        <h3 id="weather-details-header">{ i18n::t("weather.details.header") }</h3>
                        { render_weather_details(game_state.as_ref()) }
                        <button onclick={on_toggle_weather_details.clone()} class="retro-btn-secondary weather-back-btn">
                            { i18n::t("weather.details.back") }
                        </button>
                    </div>
                }
            },
        )
    } else {
        Html::default()
    };

    let pace_diet_panel = if *show_pace_diet {
        p.game_state.as_ref().map_or_else(
            || html! { <div class="error">{"Game state unavailable"}</div> },
            |game_state| {
                html! {
                    <crate::components::ui::pace_diet_panel::PaceDietPanel
                        game_state={game_state.clone()}
                        pacing_config={p.pacing_config.clone()}
                        on_pace_change={p.on_pace_change.clone()}
                        on_diet_change={p.on_diet_change.clone()}
                        on_back={on_hide_pace_diet.clone()}
                    />
                }
            },
        )
    } else {
        Html::default()
    };

    html! {
        <section class="panel">
            <h2>{ i18n::t("travel.title") }</h2>

            // Weather header strip
            { weather_info.clone() }

            // Show breakdown alert banner if travel is blocked
            if travel_blocked {
                if let Some(msg) = &breakdown_msg {
                    <div class="alert breakdown-alert" role="alert" aria-live="assertive">
                        <p>{ msg }</p>
                        <p>{ i18n::t("vehicle.announce.blocked") }</p>
                    </div>
                }
            }

            // Weather details card if shown
            if *show_weather_details {
                { weather_details }
            } else if *show_pace_diet {
                { pace_diet_panel }
            } else {
                <>
                    <div class="controls">
                        <button onclick={on_show_pace_diet} aria-label={i18n::t("pacediet.title")} class="retro-btn-secondary">
                            { i18n::t("pacediet.title") }
                        </button>
                        <button
                            onclick={on_click}
                            aria-label={i18n::t("travel.next")}
                            class="retro-btn-primary"
                            disabled={travel_blocked}
                            aria-describedby={if travel_blocked { "breakdown-notice" } else { "" }}
                        >
                            { i18n::t("travel.next") }
                        </button>
                    </div>

                    if travel_blocked {
                        <p id="breakdown-notice" class="help-text">
                            { i18n::t("vehicle.announce.blocked") }
                        </p>
                    }

                    if let Some(gs) = p.game_state.as_ref() {
                        <div class="current-settings" role="status" aria-live="polite">
                            <p>{"Current Pace: "}{gs.pace}</p>
                            <p>{"Current Info Diet: "}{gs.diet}</p>
                        </div>
                    }
                </>
            }

            if !p.logs.is_empty() {
                <div class="log" role="log" aria-live="polite">
                    { for p.logs.iter().map(|l| html!{ <p>{l}</p> }) }
                </div>
            }
        </section>
    }
}

/// Render weather header strip with current weather and effects
fn render_weather_info(game_state: &GameState) -> Html {
    use crate::game::weather::WeatherConfig;

    let weather_cfg = WeatherConfig::default_config();
    let today = game_state.weather_state.today;

    // Get weather effects for display
    let effect = weather_cfg.effects.get(&today);

    // Format effects for display
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
        crate::game::Region::Heartland => "region.heartland",
        crate::game::Region::RustBelt => "region.rustbelt",
        crate::game::Region::Beltway => "region.beltway",
    });

    html! {
        <section class="weather-strip" role="region" aria-labelledby="weather-title">
            <h3 id="weather-title" class="sr-only">{ i18n::t("weather.title") }</h3>
            <p class="weather-info"
               tabindex="0"
               role="button"
               aria-expanded="false"
               onclick={Callback::from(move |_| {})}>
                <span class="day-region">
                    { format!("Day {day} — Region: {region}", day = game_state.day, region = region_name) }
                </span>
                <br />
                <span class="weather-state">
                    { format!("Weather: {weather}{effects}", weather = weather_state_name, effects = effects_text) }
                </span>
            </p>

            // Live announcement for screen readers (polite)
            <div aria-live="polite" aria-atomic="true" class="sr-only weather-announce">
                { format_weather_announcement(today, effect) }
            </div>
        </section>
    }
}

/// Render detailed weather information card
fn render_weather_details(game_state: &GameState) -> Html {
    use crate::game::weather::WeatherConfig;

    let weather_cfg = WeatherConfig::default_config();
    let today = game_state.weather_state.today;
    let weather_state_name = i18n::t(today.i18n_key());

    // Get effects and format them
    let effect = weather_cfg.effects.get(&today);
    let effects_list = effect.map_or_else(
        || "None".to_string(),
        |eff| {
            let mut parts = Vec::new();

            if eff.supplies != 0 {
                parts.push(format_delta("Supplies", eff.supplies));
            }
            if eff.sanity != 0 {
                parts.push(format_delta("Sanity", eff.sanity));
            }
            if eff.pants != 0 {
                parts.push(format_delta("Pants", eff.pants));
            }
            if eff.enc_delta != 0.0 {
                parts.push(format_percent("Encounter", eff.enc_delta));
            }

            parts.join(", ")
        },
    );

    // Get mitigation text if applicable
    let mitigation_text =
        weather_cfg
            .mitigation
            .get(&today)
            .map_or_else(String::new, |mitigation| {
                if game_state.inventory.tags.contains(&mitigation.tag) {
                    match today {
                        crate::game::weather::Weather::Storm => i18n::t("weather.gear.storm"),
                        crate::game::weather::Weather::Smoke => i18n::t("weather.gear.smoke"),
                        crate::game::weather::Weather::ColdSnap => i18n::t("weather.gear.cold"),
                        _ => String::new(),
                    }
                } else {
                    String::new()
                }
            });

    // Get special notes
    let notes_text = match today {
        crate::game::weather::Weather::Storm => i18n::t("weather.notes.storm_crossings"),
        _ => String::new(),
    };

    html! {
        <div class="weather-details">
            <p>{ format!("• State: {state}", state = weather_state_name) }</p>
            <p>{ format!("• Effects today: {effects}", effects = effects_list) }</p>
            if !mitigation_text.is_empty() {
                <p>{ format!("• Gear mitigation: {mitigation}", mitigation = mitigation_text) }</p>
            }
            if !notes_text.is_empty() {
                <p>{ format!("• Notes: {notes_text}") }</p>
            }
        </div>
    }
}

/// Format a stat delta with proper sign
fn format_delta(stat: &str, value: i32) -> String {
    if value >= 0 {
        format!("{stat} +{value}")
    } else {
        format!("{stat} {value}") // negative sign already included
    }
}

/// Format a percentage delta with proper sign
fn format_percent(stat: &str, value: f32) -> String {
    if value >= 0.0 {
        format!("{stat} +{value:.0}%", stat = stat, value = value * 100.0)
    } else {
        format!("{stat} {value:.0}%", stat = stat, value = value * 100.0) // negative sign already included
    }
}

/// Format weather announcement for screen readers
fn format_weather_announcement(
    weather: crate::game::weather::Weather,
    effect: Option<&crate::game::weather::WeatherEffect>,
) -> String {
    let weather_name = i18n::t(weather.i18n_key());

    effect.map_or_else(
        || format!("Weather: {weather_name}"),
        |eff| {
            let mut parts = Vec::new();

            if eff.supplies != 0 {
                parts.push(format_delta("Supplies", eff.supplies));
            }
            if eff.sanity != 0 {
                parts.push(format_delta("Sanity", eff.sanity));
            }
            if eff.pants != 0 {
                parts.push(format_delta("Pants", eff.pants));
            }
            if eff.enc_delta != 0.0 {
                parts.push(format_percent("Encounter", eff.enc_delta));
            }

            let effects_text = if parts.is_empty() {
                "No effects".to_string()
            } else {
                parts.join(", ")
            };

            format!("Weather: {weather_name}. {effects_text}")
        },
    )
}
