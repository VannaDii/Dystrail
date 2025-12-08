use crate::game::{DietId, GameState, PaceId, PacingConfig};
use crate::i18n;
use std::rc::Rc;
use yew::prelude::*;

mod helpers;
#[cfg(test)]
mod tests;

use helpers::{
    diet_code, diet_preview, pace_code, pace_preview, render_weather_details, render_weather_info,
};

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
        self.logs == other.logs
            && self.game_state.as_ref().map(|gs| (&gs.pace, &gs.diet))
                == other.game_state.as_ref().map(|gs| (&gs.pace, &gs.diet))
    }
}

/// Travel panel component displaying current travel status and progress
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

    let travel_blocked = p
        .game_state
        .as_ref()
        .is_some_and(|gs| gs.day_state.travel.travel_blocked);

    let breakdown_msg = p.game_state.as_ref().and_then(|gs| {
        gs.breakdown.as_ref().map(|breakdown| {
            let part_name = i18n::t(breakdown.part.key());
            let mut vars = std::collections::BTreeMap::new();
            vars.insert("part", part_name.as_str());
            i18n::tr("vehicle.breakdown", Some(&vars))
        })
    });

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
