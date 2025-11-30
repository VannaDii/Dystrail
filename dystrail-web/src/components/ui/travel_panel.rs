use crate::components::ui::stats_bar::weather_symbol;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Inventory;
    use crate::game::Region;
    use crate::game::vehicle::{Breakdown, Part};
    use crate::game::weather::{Weather, WeatherConfig, WeatherState};
    use futures::executor::block_on;
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use std::rc::Rc;
    use yew::LocalServerRenderer;

    fn sample_game_state() -> Rc<GameState> {
        Rc::new(GameState {
            day: 3,
            region: Region::RustBelt,
            travel_blocked: true,
            breakdown: Some(Breakdown {
                part: Part::Battery,
                day_started: 2,
            }),
            logs: vec!["Log booting".into(), "Arrived in Rust Belt".into()],
            weather_state: WeatherState {
                today: Weather::Storm,
                yesterday: Weather::ColdSnap,
                extreme_streak: 1,
                heatwave_streak: 0,
                coldsnap_streak: 0,
                neutral_buffer: 0,
            },
            inventory: Inventory {
                tags: HashSet::from_iter([String::from("rain_resist")]),
                ..Inventory::default()
            },
            ..GameState::default()
        })
    }

    #[test]
    fn travel_panel_render_includes_weather_and_breakdown() {
        crate::i18n::set_lang("en");

        let html = block_on(
            LocalServerRenderer::<TravelPanel>::with_props(Props {
                on_travel: Callback::noop(),
                logs: vec!["Welcome back".into()],
                game_state: Some(sample_game_state()),
                pacing_config: Rc::new(PacingConfig::default_config()),
                on_pace_change: Callback::noop(),
                on_diet_change: Callback::noop(),
            })
            .render(),
        );

        assert!(
            html.contains("Weather: Storm"),
            "SSR output should include weather state: {html}"
        );
        assert!(
            html.contains("Travel blocked until repaired."),
            "Breakdown banner should be present when travel is blocked: {html}"
        );
        assert!(
            html.contains("Log booting") || html.contains("Welcome back"),
            "Rendered log entries should appear: {html}"
        );
    }

    #[test]
    fn helpers_format_readable_effects() {
        let pos = format_delta("Supplies", 3);
        let neg = format_delta("Supplies", -2);
        assert_eq!(pos, "Supplies +3");
        assert_eq!(neg, "Supplies -2");

        let pct_pos = format_percent("Encounter", 0.25);
        let pct_neg = format_percent("Encounter", -0.5);
        assert_eq!(pct_pos, "Encounter +25%");
        assert_eq!(pct_neg, "Encounter -50%");

        let weather_cfg = WeatherConfig::default_config();
        let announcement =
            format_weather_announcement(Weather::Storm, weather_cfg.effects.get(&Weather::Storm));
        assert!(
            announcement.contains("Weather: Storm"),
            "Announcement should mention current weather: {announcement}"
        );
    }
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
            let mut vars = std::collections::BTreeMap::new();
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
        <section class="panel travel-shell">
            <header class="section-header">
                <h2>{ i18n::t("travel.title") }</h2>
                { weather_info.clone() }
            </header>

            if travel_blocked {
                if let Some(msg) = &breakdown_msg {
                    <div class="alert breakdown-alert" role="alert" aria-live="assertive">
                        <p>{ msg }</p>
                        <p>{ i18n::t("vehicle.announce.blocked") }</p>
                    </div>
                }
            }

            { if *show_weather_details { weather_details } else if *show_pace_diet { pace_diet_panel } else {
                html! {
                    <div class="travel-body">
                        if travel_blocked {
                            <p id="breakdown-notice" class="help-text">
                                { i18n::t("vehicle.announce.blocked") }
                            </p>
                        }
                        if let Some(gs) = p.game_state.as_ref() {
                            <div class="current-settings" role="status" aria-live="polite">
                                <div class="pace-diet-row" role="list">
                                    <div class="condition-pill pace-pill" role="listitem" title={pace_preview(&p.pacing_config, gs.pace)}>
                                        <span class="sprite-badge sprite-pace" aria-hidden="true">{ pace_code(gs.pace) }</span>
                                        <span class="condition-label">{ format!("{} {}", i18n::t("menu.pace"), gs.pace) }</span>
                                    </div>
                                    <div class="condition-pill diet-pill" role="listitem" title={diet_preview(&p.pacing_config, gs.diet)}>
                                        <span class="sprite-badge sprite-diet" aria-hidden="true">{ diet_code(gs.diet) }</span>
                                        <span class="condition-label">{ format!("{} {}", i18n::t("menu.diet"), gs.diet) }</span>
                                    </div>
                                </div>
                            </div>
                        }
                        if !p.logs.is_empty() {
                            <div class="log" role="log" aria-live="polite">
                                { for p.logs.iter().map(|l| html!{ <p>{l}</p> }) }
                            </div>
                        }
                    </div>
                }
            }}

            <footer class="panel-footer">
                <button onclick={on_show_pace_diet} aria-label={i18n::t("pacediet.title")} class="retro-btn-secondary">
                    { i18n::t("pacediet.title") }
                </button>
                <button
                    onclick={on_toggle_weather_details.clone()}
                    aria-label={i18n::t("weather.details.header")}
                    class="retro-btn-secondary"
                >
                    { i18n::t("weather.details.header") }
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
            </footer>
        </section>
    }
}

/// Render weather header strip with current weather and effects
fn render_weather_info(game_state: &GameState) -> Html {
    use crate::game::weather::WeatherConfig;

    let weather_cfg = WeatherConfig::default_config();
    let today = game_state.weather_state.today;
    let icon = weather_symbol(today);

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
                <span class="weather-icon" aria-hidden="true">{ icon }</span>
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

const fn pace_code(pace: PaceId) -> &'static str {
    match pace {
        PaceId::Steady => "ST",
        PaceId::Heated => "HT",
        PaceId::Blitz => "BZ",
    }
}

const fn diet_code(diet: DietId) -> &'static str {
    match diet {
        DietId::Quiet => "QT",
        DietId::Mixed => "MX",
        DietId::Doom => "DS",
    }
}

fn pace_preview(pacing_config: &PacingConfig, pace: PaceId) -> String {
    let cfg = pacing_config.get_pace_safe(pace.as_str());
    format!(
        "San {san:+} | Pants {pants:+} | Enc {enc:+.0}%",
        san = cfg.sanity,
        pants = cfg.pants,
        enc = cfg.encounter_chance_delta * 100.0
    )
}

fn diet_preview(pacing_config: &PacingConfig, diet: DietId) -> String {
    let cfg = pacing_config.get_diet_safe(diet.as_str());
    format!(
        "San {san:+} | Pants {pants:+} | Receipts {rcpt:+}%",
        san = cfg.sanity,
        pants = cfg.pants,
        rcpt = cfg.receipt_find_pct_delta
    )
}
