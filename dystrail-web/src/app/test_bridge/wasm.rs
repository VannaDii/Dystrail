use super::shared::seed_session_with_data;
use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::dom;
use crate::game::data::EncounterData;
use crate::game::state::{CollapseCause, Ending};
use crate::game::{
    CrossingKind, Encounter, MechanicalPolicyId, OtDeluxeRoutePrompt, PendingCrossing,
};
use serde::Serialize;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[derive(Serialize)]
struct BridgeGameState {
    screen: Option<String>,
    hp: Option<i64>,
    day: Option<i64>,
    pos: Option<serde_json::Value>,
}

struct BridgeBindings {
    _seed: Closure<dyn FnMut(JsValue)>,
    _speed: Closure<dyn FnMut(JsValue)>,
    _click: Closure<dyn FnMut(JsValue, JsValue)>,
    _key: Closure<dyn FnMut(JsValue)>,
    _state: Closure<dyn FnMut() -> JsValue>,
    _screen: Closure<dyn FnMut(JsValue)>,
}

impl BridgeBindings {
    fn keep(&self) {
        let _ = (
            &self._seed,
            &self._speed,
            &self._click,
            &self._key,
            &self._state,
            &self._screen,
        );
    }
}

fn test_mode_enabled() -> bool {
    dom::window()
        .and_then(|win| win.location().search().ok())
        .map(|search| search.contains("test=1"))
        .unwrap_or(false)
}

fn screen_label(phase: Phase) -> &'static str {
    match phase {
        Phase::Boot => "boot",
        Phase::Menu => "menu",
        Phase::About => "about",
        Phase::Settings => "settings",
        Phase::Persona => "persona",
        Phase::ModeSelect => "mode-select",
        Phase::Outfitting => "outfitting",
        Phase::Travel => "travel",
        Phase::Inventory => "inventory",
        Phase::PaceDiet => "pace-diet",
        Phase::Map => "map",
        Phase::Store => "store",
        Phase::Crossing => "crossing",
        Phase::RoutePrompt => "route",
        Phase::Camp => "camp",
        Phase::Encounter => "encounter",
        Phase::Boss => "boss",
        Phase::Result => "result",
    }
}

fn ensure_data_loaded(state: &AppState) -> EncounterData {
    if state.data.encounters.is_empty() {
        let loaded = crate::game::load_encounter_data().unwrap_or_else(|_| EncounterData::empty());
        state.data.set(loaded.clone());
        loaded
    } else {
        (*state.data).clone()
    }
}

fn seed_or_default(state: &AppState) -> u64 {
    let seed = *state.run_seed;
    if seed == 0 { 1337 } else { seed }
}

fn seed_session(state: &AppState, seed: u64) {
    let data = ensure_data_loaded(state);
    seed_session_with_data(state, seed, data, |_| {});
}

fn clear_run_state(state: &AppState) {
    state.session.set(None);
    state.pending_state.set(None);
    state.logs.set(Vec::new());
    state.run_seed.set(0);
    state.code.set(AttrValue::from(""));
    state.show_save.set(false);
    state.show_settings.set(false);
}

fn fixture_encounter(data: &EncounterData) -> Encounter {
    data.encounters
        .first()
        .cloned()
        .unwrap_or_else(|| Encounter {
            id: String::from("encounter-bridge"),
            name: String::from("Encounter"),
            desc: String::new(),
            weight: 1,
            regions: Vec::new(),
            modes: Vec::new(),
            choices: Vec::new(),
            hard_stop: false,
            major_repair: false,
            chainable: false,
        })
}

fn set_screen(state: &AppState, raw: &str) {
    let screen = raw.trim().to_ascii_lowercase();
    match screen.as_str() {
        "boot" => {
            clear_run_state(state);
            state.phase.set(Phase::Boot);
        }
        "menu" => {
            clear_run_state(state);
            state.phase.set(Phase::Menu);
        }
        "about" => {
            clear_run_state(state);
            state.phase.set(Phase::About);
        }
        "settings" => {
            clear_run_state(state);
            state.phase.set(Phase::Settings);
        }
        "persona" => {
            clear_run_state(state);
            state.phase.set(Phase::Persona);
        }
        "mode" | "mode-select" => {
            clear_run_state(state);
            state.phase.set(Phase::ModeSelect);
        }
        "outfitting" => {
            clear_run_state(state);
            state
                .pending_state
                .set(Some(crate::game::GameState::default()));
            state.phase.set(Phase::Outfitting);
        }
        "travel" => {
            seed_session(state, seed_or_default(state));
            state.phase.set(Phase::Travel);
        }
        "inventory" => {
            seed_session(state, seed_or_default(state));
            state.phase.set(Phase::Inventory);
        }
        "pace" | "pace-diet" => {
            seed_session(state, seed_or_default(state));
            state.phase.set(Phase::PaceDiet);
        }
        "map" => {
            seed_session(state, seed_or_default(state));
            state.phase.set(Phase::Map);
        }
        "camp" => {
            seed_session(state, seed_or_default(state));
            state.phase.set(Phase::Camp);
        }
        "route" | "route-prompt" => {
            let data = ensure_data_loaded(state);
            seed_session_with_data(state, seed_or_default(state), data, |gs| {
                gs.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
                gs.ot_deluxe.route.pending_prompt = Some(OtDeluxeRoutePrompt::SubletteCutoff);
            });
            state.phase.set(Phase::RoutePrompt);
        }
        "store" => {
            let data = ensure_data_loaded(state);
            seed_session_with_data(state, seed_or_default(state), data, |gs| {
                gs.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
                gs.ot_deluxe.store.pending_node = Some(1);
            });
            state.phase.set(Phase::Store);
        }
        "encounter" => {
            let data = ensure_data_loaded(state);
            let encounter = fixture_encounter(&data);
            seed_session_with_data(state, seed_or_default(state), data, |gs| {
                gs.current_encounter = Some(encounter);
            });
            state.phase.set(Phase::Encounter);
        }
        "crossing" => {
            let data = ensure_data_loaded(state);
            seed_session_with_data(state, seed_or_default(state), data, |gs| {
                gs.pending_crossing = Some(PendingCrossing {
                    kind: CrossingKind::BridgeOut,
                    computed_miles_today: 0.0,
                });
            });
            state.phase.set(Phase::Crossing);
        }
        "boss" => {
            let data = ensure_data_loaded(state);
            seed_session_with_data(state, seed_or_default(state), data, |gs| {
                gs.boss.readiness.ready = true;
                gs.boss.outcome.attempted = false;
            });
            state.phase.set(Phase::Boss);
        }
        "result" => {
            set_result_state(state, Ending::BossVictory);
        }
        "result-victory" | "victory" => {
            set_result_state(state, Ending::BossVictory);
        }
        "result-boss-loss" | "boss-loss" => {
            set_result_state(state, Ending::BossVoteFailed);
        }
        "result-pants" | "pants" => {
            set_result_state(
                state,
                Ending::Collapse {
                    cause: CollapseCause::Panic,
                },
            );
        }
        "result-sanity" | "sanity" => {
            set_result_state(state, Ending::SanityLoss);
        }
        "result-resource" | "resource" => {
            set_result_state(
                state,
                Ending::Collapse {
                    cause: CollapseCause::Hunger,
                },
            );
        }
        _ => {}
    }
}

fn set_result_state(state: &AppState, ending: Ending) {
    let data = ensure_data_loaded(state);
    seed_session_with_data(state, seed_or_default(state), data, |gs| {
        gs.ending = Some(ending);
        match ending {
            Ending::BossVictory => {
                gs.boss.outcome.victory = true;
                gs.boss.outcome.attempted = true;
            }
            Ending::BossVoteFailed => {
                gs.boss.outcome.victory = false;
                gs.boss.outcome.attempted = true;
                gs.boss.readiness.ready = true;
            }
            Ending::SanityLoss => {
                gs.stats.sanity = 0;
                gs.boss.outcome.attempted = false;
            }
            Ending::Collapse { cause } => {
                match cause {
                    CollapseCause::Panic => {
                        gs.stats.pants = 100;
                    }
                    CollapseCause::Hunger => {
                        gs.stats.supplies = 0;
                    }
                    _ => {}
                }
                gs.boss.outcome.attempted = false;
            }
            Ending::VehicleFailure { .. } | Ending::Exposure { .. } => {
                gs.boss.outcome.attempted = false;
            }
        }
    });
    state.phase.set(Phase::Result);
}

fn build_bridge(state: &AppState) -> BridgeBindings {
    let seed_state = state.clone();
    let seed = Closure::wrap(Box::new(move |value: JsValue| {
        if let Some(seed) = value.as_f64().map(|v| v as u64) {
            seed_state.run_seed.set(seed);
            seed_state
                .code
                .set(AttrValue::from(crate::game::encode_friendly(false, seed)));
        }
    }) as Box<dyn FnMut(JsValue)>);

    let speed = Closure::wrap(Box::new(move |_value: JsValue| {}) as Box<dyn FnMut(JsValue)>);

    let click_state = state.clone();
    let click = Closure::wrap(Box::new(move |_x: JsValue, _y: JsValue| {
        let phase = *click_state.phase;
        match phase {
            Phase::Boot => {
                if *click_state.boot_ready {
                    click_state.phase.set(Phase::Menu);
                }
            }
            Phase::Menu | Phase::Persona | Phase::ModeSelect | Phase::Outfitting => {
                seed_session(&click_state, seed_or_default(&click_state));
                click_state.phase.set(Phase::Travel);
            }
            Phase::Travel => {
                let handlers = crate::app::view::AppHandlers::new(&click_state, None);
                handlers.travel.emit(());
            }
            Phase::Inventory
            | Phase::PaceDiet
            | Phase::Map
            | Phase::Crossing
            | Phase::RoutePrompt
            | Phase::Camp
            | Phase::Encounter
            | Phase::Store => {
                click_state.phase.set(Phase::Travel);
            }
            Phase::Boss => {
                click_state.phase.set(Phase::Result);
            }
            Phase::Result => {
                click_state.phase.set(Phase::Menu);
            }
            Phase::About | Phase::Settings => {
                click_state.phase.set(Phase::Menu);
            }
        }
    }) as Box<dyn FnMut(JsValue, JsValue)>);

    let key_state = state.clone();
    let key = Closure::wrap(Box::new(move |value: JsValue| {
        if let Some(keys) = value.as_string() {
            let handlers = crate::app::view::AppHandlers::new(&key_state, None);
            for _ in keys.chars() {
                if *key_state.phase == Phase::Travel {
                    handlers.travel.emit(());
                }
            }
        }
    }) as Box<dyn FnMut(JsValue)>);

    let state_state = state.clone();
    let state_fn = Closure::wrap(Box::new(move || {
        let phase = *state_state.phase;
        let screen = Some(screen_label(phase).to_string());
        let snapshot = (*state_state.session)
            .clone()
            .map(|sess| sess.state().clone())
            .or_else(|| (*state_state.pending_state).clone());
        let (hp, day, pos) = snapshot.map_or((None, None, None), |gs| {
            let hp = Some(i64::from(gs.stats.hp));
            let day = Some(i64::from(gs.day));
            let pos = Some(serde_json::json!({ "miles": gs.miles_traveled }));
            (hp, day, pos)
        });
        serde_wasm_bindgen::to_value(&BridgeGameState {
            screen,
            hp,
            day,
            pos,
        })
        .unwrap_or(JsValue::NULL)
    }) as Box<dyn FnMut() -> JsValue>);

    let screen_state = state.clone();
    let screen = Closure::wrap(Box::new(move |value: JsValue| {
        if let Some(screen) = value.as_string() {
            set_screen(&screen_state, &screen);
        }
    }) as Box<dyn FnMut(JsValue)>);

    BridgeBindings {
        _seed: seed,
        _speed: speed,
        _click: click,
        _key: key,
        _state: state_fn,
        _screen: screen,
    }
}

fn attach_bridge(bindings: &BridgeBindings) {
    let Some(window) = dom::window() else {
        return;
    };
    let bridge = js_sys::Object::new();
    let _ = js_sys::Reflect::set(
        &bridge,
        &JsValue::from_str("seed"),
        bindings._seed.as_ref().unchecked_ref(),
    );
    let _ = js_sys::Reflect::set(
        &bridge,
        &JsValue::from_str("speed"),
        bindings._speed.as_ref().unchecked_ref(),
    );
    let _ = js_sys::Reflect::set(
        &bridge,
        &JsValue::from_str("click"),
        bindings._click.as_ref().unchecked_ref(),
    );
    let _ = js_sys::Reflect::set(
        &bridge,
        &JsValue::from_str("key"),
        bindings._key.as_ref().unchecked_ref(),
    );
    let _ = js_sys::Reflect::set(
        &bridge,
        &JsValue::from_str("state"),
        bindings._state.as_ref().unchecked_ref(),
    );
    let _ = js_sys::Reflect::set(
        &bridge,
        &JsValue::from_str("screen"),
        bindings._screen.as_ref().unchecked_ref(),
    );
    let _ = js_sys::Reflect::set(&window, &JsValue::from_str("__dystrailTest"), &bridge);
}

#[hook]
pub fn use_test_bridge(app_state: &AppState) {
    let bridge_handle = use_mut_ref(|| None::<BridgeBindings>);
    let installed = use_mut_ref(|| false);
    let state = app_state.clone();

    use_effect_with((), move |()| {
        let cleanup = || {};
        if *installed.borrow() {
            return cleanup;
        }
        *installed.borrow_mut() = true;
        if test_mode_enabled() {
            let bindings = build_bridge(&state);
            attach_bridge(&bindings);
            bindings.keep();
            *bridge_handle.borrow_mut() = Some(bindings);
        }
        cleanup
    });
}
