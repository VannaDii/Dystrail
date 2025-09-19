use crate::i18n;
use crate::game::{GameState, PacingConfig};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct Props {
    pub on_travel: Callback<()>,
    pub logs: Vec<String>,
    pub game_state: Option<Rc<GameState>>,
    pub pacing_config: Rc<PacingConfig>,
    pub on_pace_change: Callback<String>,
    pub on_diet_change: Callback<String>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        // Compare the relevant fields for re-rendering decisions
        self.logs == other.logs &&
        self.game_state.as_ref().map(|gs| (&gs.pace, &gs.diet)) ==
        other.game_state.as_ref().map(|gs| (&gs.pace, &gs.diet))
    }
}

#[function_component(TravelPanel)]
pub fn travel_panel(p: &Props) -> Html {
    let show_pace_diet = use_state(|| false);

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
        Callback::from(move |_| {
            show_pace_diet.set(false);
        })
    };

    // Check if travel is blocked due to vehicle breakdown
    let travel_blocked = p.game_state.as_ref()
        .map(|gs| gs.travel_blocked)
        .unwrap_or(false);

    // Prepare breakdown message if needed
    let breakdown_msg = if let Some(gs) = p.game_state.as_ref() {
        if let Some(breakdown) = &gs.breakdown {
            let part_name = i18n::t(breakdown.part.key());
            let mut vars = std::collections::HashMap::new();
            vars.insert("part", part_name.as_str());
            Some(i18n::tr("vehicle.breakdown", Some(&vars)))
        } else {
            None
        }
    } else {
        None
    };

    html! {
        <section class="panel">
            <h2>{ i18n::t("travel.title") }</h2>

            // Show breakdown alert banner if travel is blocked
            if travel_blocked {
                if let Some(msg) = &breakdown_msg {
                    <div class="alert breakdown-alert" role="alert" aria-live="assertive">
                        <p>{ msg }</p>
                        <p>{ i18n::t("vehicle.announce.blocked") }</p>
                    </div>
                }
            }

            if *show_pace_diet && p.game_state.is_some() {
                if let Some(game_state) = p.game_state.as_ref() {
                    <crate::components::ui::pace_diet_panel::PaceDietPanel
                        game_state={game_state.clone()}
                        pacing_config={p.pacing_config.clone()}
                        on_pace_change={p.on_pace_change.clone()}
                        on_diet_change={p.on_diet_change.clone()}
                        on_back={on_hide_pace_diet}
                    />
                } else {
                    <div class="error">{"Game state unavailable"}</div>
                }
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
                            <p>{"Current Pace: "}{&gs.pace}</p>
                            <p>{"Current Info Diet: "}{&gs.diet}</p>
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
