use crate::components::ui::stats_bar::WeatherBadge;
use crate::game::CampConfig;
use crate::game::boss::BossConfig;
use crate::game::data::EncounterData;
use crate::game::endgame::EndgameTravelCfg;
use crate::game::pacing::PacingConfig;
use crate::game::seed::{decode_to_seed, encode_friendly, generate_code_from_entropy};
use crate::game::state::{DietId, GameMode, GameState, PaceId, Region};
use crate::game::weather::WeatherConfig;
use crate::game::{JourneySession, ResultConfig, StrategyId, load_result_config};
use crate::i18n;
use crate::pages::{
    boot::BootPage,
    boss::BossPage,
    camp::CampPage,
    encounter::EncounterPage,
    menu::{MenuAction, MenuPage},
    not_found::NotFound,
    outfitting::OutfittingPage,
    persona::PersonaPage,
    result::ResultPage,
    travel::TravelPage,
};
use crate::router::Route;
use std::rc::Rc;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Boot,
    Persona,
    Outfitting,
    Menu,
    Travel,
    Camp,
    Encounter,
    Boss,
    Result,
}

fn is_seed_code_valid(code: &str) -> bool {
    regex::Regex::new(r"^(CL|DP)-[A-Z0-9]+\d{2}$")
        .map(|re| re.is_match(code))
        .unwrap_or(false)
}

const fn default_strategy_for(mode: GameMode) -> StrategyId {
    match mode {
        GameMode::Classic | GameMode::Deep => StrategyId::Balanced,
    }
}

fn strategy_for_state(state: &GameState) -> StrategyId {
    state
        .policy
        .map_or_else(|| default_strategy_for(state.mode), StrategyId::from)
}

fn session_from_state(state: GameState, endgame_cfg: &EndgameTravelCfg) -> JourneySession {
    let strategy = strategy_for_state(&state);
    JourneySession::from_state(state, strategy, endgame_cfg)
}

fn build_weather_badge(state: &GameState, cfg: &WeatherConfig) -> WeatherBadge {
    let weather_today = state.weather_state.today;
    let mitigated = cfg
        .mitigation
        .get(&weather_today)
        .is_some_and(|mit| state.inventory.tags.contains(&mit.tag));
    WeatherBadge {
        weather: weather_today,
        mitigated,
    }
}

/// Main application component providing browser routing
///
/// Sets up the router context for the entire application and renders the main `AppInner` component.
/// This is the top-level component that gets mounted to the DOM.
#[function_component(App)]
pub fn app() -> Html {
    let router_base = crate::paths::router_base().map(AttrValue::from);
    html! {
        <BrowserRouter basename={router_base}>
            <AppInner />
        </BrowserRouter>
    }
}

#[function_component(AppInner)]
pub fn app_inner() -> Html {
    let phase = use_state(|| Phase::Boot);
    let code = use_state(|| AttrValue::from("CL-ORANGE42"));
    let code_valid = use_state(|| true);
    let data = use_state(EncounterData::empty);
    let pacing_config = use_state(PacingConfig::default_config);
    let endgame_config = use_state(EndgameTravelCfg::default_config);
    let weather_config = use_state(WeatherConfig::default_config);
    let camp_config = use_state(CampConfig::default_config);
    let boss_config = use_state(BossConfig::load_from_static);
    let result_config = use_state(ResultConfig::default);
    let preload_progress = use_state(|| 0_u8);
    let boot_ready = use_state(|| false);
    let high_contrast = use_state(crate::a11y::high_contrast_enabled);
    let pending_state = use_state(|| None::<GameState>);
    let session = use_state(|| None::<JourneySession>);
    let logs = use_state(Vec::<String>::new);
    let result = use_state(|| None::<(String, String)>);
    let run_seed = use_state(|| 0_u64);
    let show_save = use_state(|| false);
    let save_focus_target = use_state(|| AttrValue::from("save-open-btn"));
    let show_settings = use_state(|| false);
    let current_language = use_state(crate::i18n::current_lang);
    let data_ready = !data.encounters.is_empty();
    let seed_footer_seed = run_seed.clone();

    // Add routing hooks - handle potential failures gracefully
    let navigator = use_navigator();
    let navigator_for_phase = navigator.clone();
    let route = use_route::<Route>();
    let active_route = route.clone().unwrap_or(Route::Home);
    let not_found = matches!(route.as_ref(), None | Some(Route::NotFound));

    // Sync route with phase (only when phase changes programmatically)
    {
        let current_route = active_route;
        use_effect_with(
            (phase.clone(), current_route),
            move |(phase, current_route)| {
                if let Some(nav) = navigator_for_phase.as_ref() {
                    let new_route = Route::from_phase(phase);
                    // Only update route if it's different to prevent circular updates
                    if &new_route != current_route {
                        nav.push(&new_route);
                    }
                }
            },
        );
    }

    // Sync phase with route (only when route changes from URL navigation)
    {
        let phase = phase.clone();
        use_effect_with(route, move |route| {
            if let Some(route) = route
                && let Some(new_phase) = route.to_phase()
                && new_phase != *phase
            {
                phase.set(new_phase);
            }
        });
    }

    {
        let data = data.clone();
        let pacing_config = pacing_config.clone();
        let endgame_config = endgame_config.clone();
        let weather_config = weather_config.clone();
        let preload_progress = preload_progress.clone();
        let boot_ready = boot_ready.clone();
        let camp_config = camp_config.clone();
        let result_config = result_config.clone();
        use_effect_with((), move |()| {
            #[cfg(not(test))]
            {
                wasm_bindgen_futures::spawn_local(async move {
                    let mut progress = 0_u8;
                    let mut bump = |p: &UseStateHandle<u8>| {
                        progress = progress.saturating_add(9);
                        p.set(progress.min(99));
                    };
                    let loaded_data = EncounterData::load_from_static();
                    bump(&preload_progress);
                    let loaded_pacing = PacingConfig::load_from_static();
                    bump(&preload_progress);
                    let loaded_endgame = EndgameTravelCfg::default_config();
                    bump(&preload_progress);
                    let loaded_weather = WeatherConfig::load_from_static();
                    bump(&preload_progress);
                    let loaded_camp = CampConfig::load_from_static();
                    bump(&preload_progress);
                    let loaded_result = load_result_config().unwrap_or_default();
                    bump(&preload_progress);
                    // Preload remaining JSON assets to honor boot spec
                    let _ = serde_json::from_str::<crate::game::store::Store>(include_str!(
                        "../static/assets/data/store.json"
                    ));
                    bump(&preload_progress);
                    let _ = crate::game::personas::PersonasList::from_json(include_str!(
                        "../static/assets/data/personas.json"
                    ));
                    bump(&preload_progress);
                    let _ = serde_json::from_str::<crate::game::crossings::CrossingConfig>(
                        include_str!("../static/assets/data/crossings.json"),
                    );
                    bump(&preload_progress);
                    let _ = serde_json::from_str::<crate::game::vehicle::VehicleConfig>(
                        include_str!("../static/assets/data/vehicle.json"),
                    );
                    bump(&preload_progress);
                    let _ = serde_json::from_str::<crate::game::boss::BossConfig>(include_str!(
                        "../static/assets/data/boss.json"
                    ));
                    bump(&preload_progress);
                    data.set(loaded_data);
                    pacing_config.set(loaded_pacing);
                    endgame_config.set(loaded_endgame);
                    weather_config.set(loaded_weather);
                    camp_config.set(loaded_camp);
                    result_config.set(loaded_result);
                    preload_progress.set(100);
                    boot_ready.set(true);
                });
            }
            #[cfg(test)]
            {
                let loaded_data = EncounterData::load_from_static();
                let loaded_pacing = PacingConfig::load_from_static();
                let loaded_endgame = EndgameTravelCfg::default_config();
                let loaded_weather = WeatherConfig::load_from_static();
                let loaded_camp = CampConfig::load_from_static();
                let loaded_result = load_result_config().unwrap_or_default();
                data.set(loaded_data);
                pacing_config.set(loaded_pacing);
                endgame_config.set(loaded_endgame);
                weather_config.set(loaded_weather);
                camp_config.set(loaded_camp);
                result_config.set(loaded_result);
                preload_progress.set(100);
                boot_ready.set(true);
            }
            || {}
        });
    }

    let _on_code_change = {
        let code_handle = code.clone();
        let code_valid_handle = code_valid;
        Callback::from(move |v: String| {
            let v_up = v.trim().to_ascii_uppercase();
            let valid = is_seed_code_valid(&v_up);
            code_handle.set(v_up.into());
            code_valid_handle.set(valid);
        })
    };

    let do_travel = {
        let session_handle = session.clone();
        let logs = logs.clone();
        let phase = phase.clone();
        let pacing_cfg = (*pacing_config).clone();
        Callback::from(move |()| {
            let Some(mut sess) = (*session_handle).clone() else {
                return;
            };
            sess.with_state_mut(|state| state.apply_pace_and_diet(&pacing_cfg));
            let outcome = sess.tick_day();

            let mut lg = (*logs).clone();
            lg.push(crate::i18n::t(&outcome.log_key));
            let state_ref = sess.state();
            if outcome.ended || state_ref.stats.pants >= 100 {
                phase.set(Phase::Result);
            } else if state_ref.current_encounter.is_some() {
                phase.set(Phase::Encounter);
            } else if matches!(state_ref.region, Region::Beltway) && state_ref.day > 12 {
                phase.set(Phase::Boss);
            }

            logs.set(lg);
            session_handle.set(Some(sess));
        })
    };

    let on_pace_change = {
        let session_handle = session.clone();
        Callback::from(move |new_pace: PaceId| {
            if let Some(mut sess) = (*session_handle).clone() {
                sess.with_state_mut(|state| state.pace = new_pace);
                session_handle.set(Some(sess));
            }
        })
    };

    let on_diet_change = {
        let session_handle = session.clone();
        Callback::from(move |new_diet: DietId| {
            if let Some(mut sess) = (*session_handle).clone() {
                sess.with_state_mut(|state| state.diet = new_diet);
                session_handle.set(Some(sess));
            }
        })
    };

    let on_choice = {
        let session_handle = session.clone();
        let phase_handle = phase.clone();
        let logs_handle = logs.clone();
        Callback::from(move |idx: usize| {
            if let Some(mut sess) = (*session_handle).clone() {
                sess.with_state_mut(|state| state.apply_choice(idx));
                let mut lg = (*logs_handle).clone();
                lg.push(format!("Chose option {}", idx + 1));
                phase_handle.set(Phase::Travel);
                logs_handle.set(lg);
                session_handle.set(Some(sess));
            }
        })
    };

    let boss_act = {
        let session_handle = session.clone();
        let phase_handle = phase.clone();
        let result_handle = result;
        let boss_config_handle = boss_config.clone();
        Callback::from(move |()| {
            if let Some(mut sess) = (*session_handle).clone() {
                let cfg = (*boss_config_handle).clone();
                let out =
                    sess.with_state_mut(|state| crate::game::boss::run_boss_minigame(state, &cfg));
                let (title_key, summary_key) = match out {
                    crate::game::boss::BossOutcome::PassedCloture => {
                        ("result.passed_cloture", "result.passed_cloture_desc")
                    }
                    crate::game::boss::BossOutcome::SurvivedFlood => (
                        "result.survived_filibuster",
                        "result.survived_filibuster_desc",
                    ),
                    crate::game::boss::BossOutcome::PantsEmergency => {
                        ("result.pants_emergency", "result.pants_emergency_desc")
                    }
                    crate::game::boss::BossOutcome::Exhausted => {
                        ("result.exhausted", "result.exhausted_desc")
                    }
                };
                let title = crate::i18n::t(title_key);
                let summary = crate::i18n::t(summary_key);
                result_handle.set(Some((title, summary)));
                phase_handle.set(Phase::Result);
                session_handle.set(Some(sess));
            }
        })
    };

    // Save/Load drawer callbacks
    let open_save = *show_save;
    let on_close_save = {
        let s = show_save.clone();
        Callback::from(move |()| s.set(false))
    };
    let on_save_cb = {
        let session_handle = session.clone();
        let logs_handle = logs.clone();
        Callback::from(move |()| {
            if let Some(sess) = (*session_handle).clone() {
                sess.state().save();
                let mut l = (*logs_handle).clone();
                l.push(i18n::t("save.saved"));
                logs_handle.set(l);
            }
        })
    };
    let on_load_cb = {
        let session_handle = session.clone();
        let pending_handle = pending_state.clone();
        let data_handle = data.clone();
        let logs_handle = logs.clone();
        let phase_handle = phase.clone();
        let run_seed_handle = run_seed.clone();
        let endgame_cfg = (*endgame_config).clone();
        Callback::from(move |()| {
            if let Some(mut gs) = GameState::load() {
                gs = gs.rehydrate((*data_handle).clone());
                let sess = session_from_state(gs, &endgame_cfg);
                run_seed_handle.set(sess.state().seed);
                pending_handle.set(Some(sess.state().clone()));
                session_handle.set(Some(sess));
                let mut l = (*logs_handle).clone();
                l.push(i18n::t("save.loaded"));
                logs_handle.set(l);
                phase_handle.set(Phase::Travel);
            }
        })
    };
    let on_export_cb = {
        let session_handle = session.clone();
        Callback::from(move |()| {
            let Some(sess) = (*session_handle).clone() else {
                return;
            };
            let Ok(text) = serde_json::to_string(sess.state()) else {
                return;
            };
            let Some(win) = web_sys::window() else {
                return;
            };
            let nav = win.navigator();
            let cb = nav.clipboard();
            let _ = cb.write_text(&text);
        })
    };
    let on_import_cb = {
        let session_handle = session.clone();
        let pending_handle = pending_state.clone();
        let data_handle = data.clone();
        let logs_handle = logs.clone();
        let run_seed_handle = run_seed.clone();
        let phase_handle = phase.clone();
        let endgame_cfg = (*endgame_config).clone();
        Callback::from(move |txt: String| {
            if let Ok(mut gs) = serde_json::from_str::<GameState>(&txt) {
                gs = gs.rehydrate((*data_handle).clone());
                let sess = session_from_state(gs, &endgame_cfg);
                run_seed_handle.set(sess.state().seed);
                pending_handle.set(Some(sess.state().clone()));
                session_handle.set(Some(sess));
                let mut l = (*logs_handle).clone();
                l.push(i18n::t("save.loaded"));
                logs_handle.set(l);
                phase_handle.set(Phase::Travel);
            } else {
                let mut l = (*logs_handle).clone();
                l.push(i18n::t("save.error"));
                logs_handle.set(l);
            }
        })
    };

    // Language change callback
    let on_lang_change = {
        let current_language = current_language.clone();
        Callback::from(move |code: String| {
            crate::i18n::set_lang(&code);
            current_language.set(code);
        })
    };
    let on_hc_toggle = {
        let high_contrast = high_contrast.clone();
        Callback::from(move |next: bool| {
            crate::a11y::set_high_contrast(next);
            high_contrast.set(next);
        })
    };
    let on_settings_hc_changed = {
        let high_contrast = high_contrast.clone();
        Callback::from(move |next: bool| {
            high_contrast.set(next);
        })
    };
    let on_open_save_header = {
        let show_save = show_save.clone();
        let focus_target = save_focus_target.clone();
        Callback::from(move |()| {
            focus_target.set(AttrValue::from("save-open-btn"));
            show_save.set(true);
        })
    };

    let go_home = {
        let phase = phase.clone();
        Callback::from(move |()| {
            if let Some(nav) = navigator.as_ref() {
                nav.push(&Route::Home);
            }
            phase.set(Phase::Menu);
        })
    };

    let begin_boot = {
        let phase = phase.clone();
        let ready = boot_ready.clone();
        Callback::from(move |()| {
            if *ready {
                phase.set(Phase::Persona);
            }
        })
    };

    let persona_pending = pending_state.clone();
    let outfitting_pending = pending_state.clone();
    let camp_pending = pending_state.clone();
    let result_pending_replay = pending_state.clone();
    let result_pending_new = pending_state.clone();
    let result_pending_title = pending_state.clone();
    let menu_pending = pending_state.clone();
    let menu_logs = logs.clone();
    let menu_run_seed = run_seed.clone();
    let menu_session = session.clone();
    let menu_phase = phase.clone();
    let menu_code = code.clone();
    let menu_endgame_cfg = (*endgame_config).clone();

    let main_view = if not_found {
        html! { <NotFound on_go_home={go_home} /> }
    } else {
        match *phase {
            Phase::Boot => {
                let boot_logo_src: AttrValue =
                    crate::paths::asset_path("static/img/logo.png").into();
                html! {
                    <BootPage
                        logo_src={boot_logo_src}
                        ready={*boot_ready}
                        preload_progress={*preload_progress}
                        on_begin={begin_boot}
                    />
                }
            }
            Phase::Persona => {
                let on_selected = {
                    let pending = persona_pending;
                    Callback::from(move |per: crate::game::personas::Persona| {
                        let mut gs = (*pending).clone().unwrap_or_default();
                        gs.apply_persona(&per);
                        pending.set(Some(gs));
                    })
                };
                let on_continue = {
                    let phase = phase.clone();
                    Callback::from(move |()| phase.set(Phase::Outfitting))
                };
                html! { <PersonaPage {on_selected} {on_continue} /> }
            }
            Phase::Outfitting => {
                let current_state = (*pending_state).clone().unwrap_or_default();
                let on_continue = {
                    let pending_handle = outfitting_pending;
                    let phase_handle = phase.clone();
                    Callback::from(
                        move |(new_state, _grants, _tags): (
                            crate::game::GameState,
                            crate::game::store::Grants,
                            Vec<String>,
                        )| {
                            pending_handle.set(Some(new_state));
                            phase_handle.set(Phase::Menu);
                        },
                    )
                };
                html! {
                    <OutfittingPage game_state={current_state} {on_continue} />
                }
            }
            Phase::Menu => {
                let start_with_code_action = {
                    let code_handle = menu_code;
                    let pending_handle = menu_pending;
                    let phase_handle = menu_phase;
                    let logs_handle = menu_logs;
                    let data_handle = data;
                    let run_seed_handle = menu_run_seed;
                    let session_handle = menu_session;
                    let endgame_cfg = menu_endgame_cfg;
                    move || {
                        if let Some((is_deep, seed)) = decode_to_seed(&code_handle) {
                            let mode = if is_deep {
                                GameMode::Deep
                            } else {
                                GameMode::Classic
                            };
                            let base = (*pending_handle).clone().unwrap_or_default();
                            let gs = base.with_seed(seed, mode, (*data_handle).clone());
                            let sess = session_from_state(gs, &endgame_cfg);
                            let mode_label = if is_deep {
                                crate::i18n::t("mode.deep")
                            } else {
                                crate::i18n::t("mode.classic")
                            };
                            let mut m = std::collections::BTreeMap::new();
                            m.insert("mode", mode_label.as_str());
                            logs_handle.set(vec![crate::i18n::tr("log.run_begins", Some(&m))]);
                            run_seed_handle.set(seed);
                            pending_handle.set(Some(sess.state().clone()));
                            session_handle.set(Some(sess));
                            phase_handle.set(Phase::Travel);
                        } else {
                            let entropy = js_sys::Date::now().to_bits();
                            let new_code = generate_code_from_entropy(false, entropy);
                            code_handle.set(new_code.clone().into());
                            if let Some((_, seed)) = decode_to_seed(&new_code) {
                                let base = (*pending_handle).clone().unwrap_or_default();
                                let gs =
                                    base.with_seed(seed, GameMode::Classic, (*data_handle).clone());
                                let sess = session_from_state(gs, &endgame_cfg);
                                let mode_label = crate::i18n::t("mode.classic");
                                let mut m = std::collections::BTreeMap::new();
                                m.insert("mode", mode_label.as_str());
                                logs_handle.set(vec![crate::i18n::tr("log.run_begins", Some(&m))]);
                                run_seed_handle.set(seed);
                                pending_handle.set(Some(sess.state().clone()));
                                session_handle.set(Some(sess));
                                phase_handle.set(Phase::Travel);
                            }
                        }
                    }
                };

                let on_action = {
                    let show_save_handle = show_save.clone();
                    let show_settings_handle = show_settings.clone();
                    let save_focus = save_focus_target.clone();
                    let phase_handle = phase.clone();
                    Callback::from(move |action: MenuAction| match action {
                        MenuAction::StartRun => start_with_code_action(),
                        MenuAction::CampPreview => phase_handle.set(Phase::Camp),
                        MenuAction::OpenSave => {
                            save_focus.set(AttrValue::from("save-open-btn"));
                            show_save_handle.set(true);
                        }
                        MenuAction::OpenSettings => show_settings_handle.set(true),
                        MenuAction::Reset => phase_handle.set(Phase::Boot),
                    })
                };

                let menu_logo_src: AttrValue =
                    crate::paths::asset_path("static/img/logo.png").into();
                html! { <MenuPage code={(*code).clone()} logo_src={menu_logo_src} on_action={on_action} /> }
            }
            Phase::Travel => (*session).clone().map_or_else(Html::default, |sess| {
                let snapshot = sess.state().clone();
                let weather_badge = build_weather_badge(&snapshot, &weather_config);
                let state_rc = Rc::new(snapshot);
                let pacing_config_rc = Rc::new((*pacing_config).clone());
                html! {
                    <TravelPage
                        state={state_rc}
                        logs={(*logs).clone()}
                        pacing_config={pacing_config_rc}
                        weather_badge={weather_badge}
                        data_ready={data_ready}
                        on_travel={do_travel.clone()}
                        on_pace_change={on_pace_change.clone()}
                        on_diet_change={on_diet_change.clone()}
                    />
                }
            }),
            Phase::Camp => (*session).clone().map_or_else(Html::default, |sess| {
                let snapshot = sess.state().clone();
                let weather_cfg = (*weather_config).clone();
                let weather_today = snapshot.weather_state.today;
                let weather_mitigated = weather_cfg
                    .mitigation
                    .get(&weather_today)
                    .is_some_and(|mit| snapshot.inventory.tags.contains(&mit.tag));
                let weather_badge = WeatherBadge {
                    weather: weather_today,
                    mitigated: weather_mitigated,
                };
                let camp_state = Rc::new(snapshot);
                let camp_config_rc = Rc::new((*camp_config).clone());
                html! {
                    <CampPage
                        state={camp_state}
                        camp_config={camp_config_rc}
                        weather={weather_badge}
                        on_state_change={{
                            let session_handle = session.clone();
                            let pending_state = camp_pending.clone();
                            let endgame_cfg = (*endgame_config).clone();
                            Callback::from(move |new_state: GameState| {
                                let snapshot = new_state.clone();
                                let updated = session_from_state(new_state, &endgame_cfg);
                                pending_state.set(Some(snapshot));
                                session_handle.set(Some(updated));
                            })
                        }}
                        on_close={{
                            let phase_handle = phase.clone();
                            Callback::from(move |()| phase_handle.set(Phase::Menu))
                        }}
                    />
                }
            }),
            Phase::Encounter => (*session).clone().map_or_else(Html::default, |sess| {
                let snapshot = sess.state().clone();
                let weather_badge = build_weather_badge(&snapshot, &weather_config);
                html! {
                    <EncounterPage
                        state={Rc::new(snapshot)}
                        weather={weather_badge}
                        on_choice={on_choice.clone()}
                    />
                }
            }),
            Phase::Boss => (*session).clone().map_or_else(Html::default, |sess| {
                let gs = sess.state().clone();
                let cfg = (*boss_config).clone();
                let weather_badge = build_weather_badge(&gs, &weather_config);
                html! {
                    <BossPage
                        state={gs}
                        config={cfg}
                        weather={weather_badge}
                        on_begin={boss_act.clone()}
                    />
                }
            }),
            Phase::Result => (*session).clone().map_or_else(Html::default, |sess| {
                let result_state = sess.state().clone();
                let result_config_data = (*result_config).clone();
                let boss_won = result_state.boss.outcome.victory;

                let session_for_replay = session.clone();
                let pending_state_for_replay = result_pending_replay.clone();
                let seed_for_replay = *run_seed;
                let on_replay_seed = Callback::from(move |()| {
                    let new_game = GameState {
                        seed: seed_for_replay,
                        ..GameState::default()
                    };
                    pending_state_for_replay.set(Some(new_game));
                    session_for_replay.set(None);
                });

                let session_for_new_run = session.clone();
                let pending_state_for_new_run = result_pending_new.clone();
                let on_new_run = Callback::from(move |()| {
                    pending_state_for_new_run.set(Some(GameState::default()));
                    session_for_new_run.set(None);
                });

                let session_for_title = session.clone();
                let pending_state_for_title = result_pending_title.clone();
                let on_title = {
                    let phase = phase.clone();
                    Callback::from(move |()| {
                        pending_state_for_title.set(None);
                        session_for_title.set(None);
                        phase.set(Phase::Boot);
                    })
                };

                let on_export = {
                    let seed = *run_seed;
                    let is_deep = result_state.mode.is_deep();
                    Callback::from(move |()| {
                        let code_str = encode_friendly(is_deep, seed);
                        if let Some(win) = web_sys::window() {
                            let nav = win.navigator();
                            let cb = nav.clipboard();
                            let _ = cb.write_text(&code_str);
                        }
                    })
                };

                html! {
                    <ResultPage
                        state={result_state}
                        result_config={result_config_data}
                        boss_won={boss_won}
                        on_replay_seed={on_replay_seed}
                        on_new_run={on_new_run}
                        on_title={on_title}
                        on_export={on_export}
                    />
                }
            }),
        }
    };

    let seed_footer = (*session)
        .as_ref()
        .map(|sess| {
            let seed_value = if *seed_footer_seed == 0 {
                sess.state().seed
            } else {
                *seed_footer_seed
            };
            let is_deep = sess.state().mode.is_deep();
            let open_save_from_footer = {
                let focus_target = save_focus_target.clone();
                let save_open = show_save.clone();
                Callback::from(move |_| {
                    focus_target.set(AttrValue::from("seed-save-btn"));
                    save_open.set(true);
                })
            };
            let open_settings_from_footer = {
                let show_settings = show_settings.clone();
                Callback::from(move |_| show_settings.set(true))
            };
            html! {
                <crate::components::ui::seed_footer::SeedFooter seed={seed_value} is_deep_mode={is_deep}>
                    <button
                        id="seed-save-btn"
                        class="retro-btn-secondary"
                        onclick={open_save_from_footer}
                    >
                        { i18n::t("save.header") }
                    </button>
                    <button class="retro-btn-secondary" onclick={open_settings_from_footer}>
                        { i18n::t("menu.settings") }
                    </button>
                </crate::components::ui::seed_footer::SeedFooter>
            }
        })
        .unwrap_or_default();

    html! {
        <>
            <crate::components::header::Header
                on_open_save={on_open_save_header}
                on_lang_change={on_lang_change}
                current_lang={(*current_language).clone()}
                high_contrast={*high_contrast}
                on_toggle_hc={on_hc_toggle}
            />
            <main id="main" role="main">
                <style>{ crate::a11y::visible_focus_css() }</style>
                { html!{ <crate::components::ui::save_drawer::SaveDrawer open={open_save} on_close={on_close_save} on_save={on_save_cb} on_load={on_load_cb} on_export={on_export_cb} on_import={on_import_cb} return_focus_id={Some((*save_focus_target).clone())} /> } }
                { html!{ <crate::components::ui::settings_dialog::SettingsDialog open={*show_settings} on_close={{ let s=show_settings.clone(); Callback::from(move |()| s.set(false)) }} on_hc_changed={on_settings_hc_changed.clone()} /> } }
                { main_view }
                <div class="panel-footer nav-footer" role="navigation" aria-label={i18n::t("menu.title")}>
                    <button class="retro-btn-secondary" onclick={{ let s=show_settings.clone(); Callback::from(move |_| s.set(true)) }}>
                        { i18n::t("menu.settings") }
                    </button>
                    <button class="retro-btn-secondary" onclick={{ let s=show_save.clone(); Callback::from(move |_| s.set(true)) }}>
                        { i18n::t("save.header") }
                    </button>
                </div>
                { seed_footer }
                <crate::components::footer::Footer />
            </main>
        </>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_code_validation_handles_expected_formats() {
        assert!(is_seed_code_valid("CL-ORANGE42"));
        assert!(is_seed_code_valid("DP-SIGNAL99"));
        assert!(!is_seed_code_valid("CL-ORANGE4"));
        assert!(!is_seed_code_valid("INVALID"));
        assert!(!is_seed_code_valid("XY-TOOLATE00"));
    }

    #[test]
    fn route_phase_mappings_cover_all_states() {
        use crate::router::Route;

        let phases = [
            Phase::Boot,
            Phase::Persona,
            Phase::Outfitting,
            Phase::Menu,
            Phase::Travel,
            Phase::Camp,
            Phase::Encounter,
            Phase::Boss,
            Phase::Result,
        ];

        for phase in phases {
            let route = Route::from_phase(&phase);
            let round_trip = route.to_phase();
            match (phase, round_trip) {
                (Phase::Boot | Phase::Menu, Some(mapped)) => {
                    assert!(mapped == Phase::Menu);
                }
                (_, Some(mapped)) => assert!(mapped == phase),
                (_, None) => panic!("Route should map to a phase"),
            }
        }
    }
}
