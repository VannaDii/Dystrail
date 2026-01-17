use super::layout::{IntentActions, PanelContext, PanelMode, render_panel};
use super::weather::{render_weather_details, render_weather_info};
use crate::game::{DietId, GameState, MechanicalPolicyId, PaceId, PacingConfig};
use crate::i18n;
use std::rc::Rc;
use web_sys::MouseEvent;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct Props {
    pub on_travel: Callback<()>,
    pub on_trade: Callback<()>,
    pub on_hunt: Callback<()>,
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

    let trigger_travel = p.on_travel.clone();
    let trigger_trade = p.on_trade.clone();
    let trigger_hunt = p.on_hunt.clone();
    let on_click: Callback<MouseEvent> = {
        let cb = trigger_travel.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_trade_click: Callback<MouseEvent> = {
        let cb = trigger_trade.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_hunt_click: Callback<MouseEvent> = {
        let cb = trigger_hunt.clone();
        Callback::from(move |_| cb.emit(()))
    };

    let on_show_pace_diet: Callback<MouseEvent> = {
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

    let on_toggle_weather_details: Callback<MouseEvent> = {
        let show_weather_details = show_weather_details.clone();
        Callback::from(move |_| {
            show_weather_details.set(!*show_weather_details);
        })
    };

    let travel_blocked = p
        .game_state
        .as_ref()
        .is_some_and(|gs| gs.day_state.travel.travel_blocked);
    let show_otdeluxe_intents = p
        .game_state
        .as_ref()
        .is_some_and(|gs| gs.mechanical_policy == MechanicalPolicyId::OtDeluxe90s);

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

    let panel_mode = if *show_weather_details {
        PanelMode::WeatherDetails
    } else if *show_pace_diet {
        PanelMode::PaceDiet
    } else {
        PanelMode::Main
    };

    let intent_actions = show_otdeluxe_intents.then_some(IntentActions {
        on_trade: &on_trade_click,
        on_hunt: &on_hunt_click,
    });

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

    let on_keydown = {
        let intents_enabled = show_otdeluxe_intents;
        Callback::from(move |e: KeyboardEvent| match e.key().as_str() {
            "Enter" | " " => {
                trigger_travel.emit(());
                e.prevent_default();
            }
            "p" | "P" => {
                show_pace_diet.set(true);
                e.prevent_default();
            }
            "t" | "T" if intents_enabled => {
                trigger_trade.emit(());
                e.prevent_default();
            }
            "h" | "H" if intents_enabled => {
                trigger_hunt.emit(());
                e.prevent_default();
            }
            _ => {}
        })
    };

    html! {
        <section class="panel travel-shell" onkeydown={on_keydown}>
            { render_panel(&PanelContext {
                travel_blocked,
                breakdown_msg: breakdown_msg.as_deref(),
                mode: panel_mode,
                weather_details,
                pace_diet_panel,
                weather_info,
                logs: &p.logs,
                game_state: p.game_state.as_deref(),
                pacing_config: &p.pacing_config,
                intent_actions,
                on_show_pace_diet: &on_show_pace_diet,
                on_toggle_weather_details: &on_toggle_weather_details,
                on_click: &on_click,
            }) }
        </section>
    }
}
