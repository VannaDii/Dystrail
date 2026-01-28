use super::layout::{IntentActions, PanelContext, PanelMode, render_panel};
use super::weather::{render_weather_details, render_weather_info};
use crate::game::{GameState, MechanicalPolicyId, PacingConfig};
use crate::i18n;
use std::rc::Rc;
use web_sys::KeyboardEvent;
use web_sys::MouseEvent;
use yew::prelude::*;

#[cfg(test)]
pub(super) const fn next_flag_state(_current: bool, value: bool) -> bool {
    value
}

pub(super) const fn next_flag_toggle(current: bool) -> bool {
    !current
}

#[cfg(test)]
pub(super) fn show_flag_action(flag: UseStateHandle<bool>, value: bool) -> Callback<()> {
    Callback::from(move |()| flag.set(next_flag_state(*flag, value)))
}

pub(super) fn toggle_flag_action(flag: UseStateHandle<bool>) -> Callback<()> {
    Callback::from(move |()| flag.set(next_flag_toggle(*flag)))
}

pub(super) const fn compute_panel_mode(show_weather_details: bool) -> PanelMode {
    if show_weather_details {
        PanelMode::WeatherDetails
    } else {
        PanelMode::Main
    }
}

pub(super) fn build_weather_details(
    show_weather_details: bool,
    game_state: Option<&GameState>,
    on_toggle: &Callback<()>,
) -> Html {
    if show_weather_details {
        game_state.map_or_else(Html::default, |state| {
            let on_click = on_toggle.reform(|_e: MouseEvent| ());
            html! {
                <div class="weather-details-card" role="dialog" aria-labelledby="weather-details-header">
                    <h3 id="weather-details-header">{ i18n::t("weather.details.header") }</h3>
                    { render_weather_details(state) }
                    <button onclick={on_click} class="retro-btn-secondary weather-back-btn">
                        { i18n::t("weather.details.back") }
                    </button>
                </div>
            }
        })
    } else {
        Html::default()
    }
}

#[derive(Properties, Clone)]
pub struct Props {
    pub on_travel: Callback<()>,
    pub on_trade: Callback<()>,
    pub on_hunt: Callback<()>,
    pub on_open_inventory: Callback<()>,
    pub on_open_pace_diet: Callback<()>,
    pub on_open_map: Callback<()>,
    pub logs: Vec<String>,
    pub game_state: Option<Rc<GameState>>,
    pub pacing_config: Rc<PacingConfig>,
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
    let show_weather_details = use_state(|| false);

    let trigger_travel = p.on_travel.clone();
    let trigger_trade = p.on_trade.clone();
    let trigger_hunt = p.on_hunt.clone();
    let open_inventory_click = p.on_open_inventory.clone();
    let open_pace_diet_click = p.on_open_pace_diet.clone();
    let open_map_click = p.on_open_map.clone();
    let open_inventory_key = p.on_open_inventory.clone();
    let open_pace_diet_key = p.on_open_pace_diet.clone();
    let open_map_key = p.on_open_map.clone();

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

    let on_open_inventory = Callback::from(move |_e: MouseEvent| open_inventory_click.emit(()));
    let on_open_pace_diet = Callback::from(move |_e: MouseEvent| open_pace_diet_click.emit(()));
    let on_open_map = Callback::from(move |_e: MouseEvent| open_map_click.emit(()));

    let on_toggle_weather_details_action = toggle_flag_action(show_weather_details.clone());
    let on_toggle_weather_details = on_toggle_weather_details_action.reform(|_e: MouseEvent| ());

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
        .as_deref()
        .map_or_else(|| html! {}, render_weather_info);

    let weather_details = build_weather_details(
        *show_weather_details,
        p.game_state.as_deref(),
        &on_toggle_weather_details_action,
    );

    let panel_mode = compute_panel_mode(*show_weather_details);

    let intent_actions = show_otdeluxe_intents.then_some(IntentActions {
        on_trade: &on_trade_click,
        on_hunt: &on_hunt_click,
    });

    let on_keydown = {
        let intents_enabled = show_otdeluxe_intents;
        let open_inventory = open_inventory_key;
        let open_pace_diet = open_pace_diet_key;
        let open_map = open_map_key;
        #[cfg(target_arch = "wasm32")]
        {
            Callback::from(move |e: KeyboardEvent| match e.key().as_str() {
                "Enter" | " " => {
                    trigger_travel.emit(());
                    e.prevent_default();
                }
                "i" | "I" => {
                    open_inventory.emit(());
                    e.prevent_default();
                }
                "p" | "P" => {
                    open_pace_diet.emit(());
                    e.prevent_default();
                }
                "m" | "M" => {
                    open_map.emit(());
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
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (
                trigger_travel,
                trigger_trade,
                trigger_hunt,
                open_inventory,
                open_pace_diet,
                open_map,
                intents_enabled,
            );
            Callback::from(|_e: KeyboardEvent| {})
        }
    };

    html! {
        <section class="panel travel-shell" onkeydown={on_keydown}>
            { render_panel(&PanelContext {
                travel_blocked,
                breakdown_msg: breakdown_msg.as_deref(),
                mode: panel_mode,
                weather_details,
                weather_info,
                logs: &p.logs,
                game_state: p.game_state.as_deref(),
                pacing_config: &p.pacing_config,
                intent_actions,
                on_open_inventory: &on_open_inventory,
                on_open_pace_diet: &on_open_pace_diet,
                on_open_map: &on_open_map,
                on_toggle_weather_details: &on_toggle_weather_details,
                on_click: &on_click,
            }) }
        </section>
    }
}
