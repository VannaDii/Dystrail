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
use crate::routes::Route;
use std::rc::Rc;
use wasm_bindgen::JsCast;
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
    html! {
        <BrowserRouter>
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
    let pending_state = use_state(|| None::<GameState>);
    let session = use_state(|| None::<JourneySession>);
    let logs = use_state(Vec::<String>::new);
    let result = use_state(|| None::<(String, String)>);
    let run_seed = use_state(|| 0_u64);
    let show_save = use_state(|| false);
    let show_settings = use_state(|| false);
    let current_language = use_state(crate::i18n::current_lang);
    let data_ready = !data.encounters.is_empty();

    // Add routing hooks - handle potential failures gracefully
    let navigator = use_navigator();
    let route = use_route::<Route>().unwrap_or(Route::Home);

    // Sync route with phase (only when phase changes programmatically)
    {
        let navigator_for_phase = navigator;
        let current_route = route.clone();
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
            if let Some(new_phase) = route.to_phase() {
                // Only update phase if it's different to prevent circular updates
                if new_phase != *phase {
                    phase.set(new_phase);
                }
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

    #[allow(unused_variables)]
    let on_code_change = {
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
        Callback::from(move |_| {
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
        Callback::from(move |e: web_sys::Event| {
            if let Some(select) = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
            {
                crate::i18n::set_lang(&select.value());
                current_language.set(select.value());
            }
        })
    };

    let main_view = match *phase {
        Phase::Boot => {
            let on_begin_click = {
                let phase = phase.clone();
                let ready = boot_ready.clone();
                Callback::from(move |_| {
                    if *ready {
                        phase.set(Phase::Persona);
                    }
                })
            };
            let on_begin_key = {
                let phase = phase.clone();
                let ready = boot_ready.clone();
                Callback::from(move |e: web_sys::KeyboardEvent| {
                    if *ready {
                        e.prevent_default();
                        phase.set(Phase::Persona);
                    }
                })
            };
            html! {
                <section class="panel boot-screen" aria-busy={(!*boot_ready).to_string()} aria-live="polite" onclick={on_begin_click} onkeydown={on_begin_key} tabindex="0">
                    <img src="/static/img/logo.png" alt="Dystrail" loading="eager" style="width:min(520px,80vw)"/>
                    <div class="bar-wrap" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={(*preload_progress).to_string()}>
                        <div class="bar-fill" style={format!("width:{}%", *preload_progress)}/>
                    </div>
                    <p class={classes!("muted", if *boot_ready { Some("cta-pulse") } else { None })}>
                        { if *boot_ready { i18n::t("ui.cta_start") } else { i18n::t("ui.loading") } }
                    </p>
                </section>
            }
        }
        Phase::Persona => {
            // On-persona selected callback
            #[allow(clippy::redundant_clone)]
            let on_selected = {
                let pending = pending_state.clone();
                Callback::from(move |per: crate::game::personas::Persona| {
                    let mut gs = (*pending).clone().unwrap_or_default();
                    gs.apply_persona(&per);
                    pending.set(Some(gs));
                })
            };
            #[allow(clippy::redundant_clone)]
            #[allow(clippy::redundant_clone)]
            let on_continue = {
                let phase = phase.clone();
                Callback::from(move |()| phase.set(Phase::Outfitting))
            };
            html! {
              <section class="panel retro-menu">
                <crate::components::ui::persona_select::PersonaSelect on_selected={Some(on_selected)} on_continue={Some(on_continue)} />
              </section>
            }
        }
        Phase::Outfitting => {
            // Outfitting Store
            let current_state = (*pending_state).clone().unwrap_or_default();
            let on_continue = {
                #[allow(clippy::redundant_clone)]
                let pending = pending_state.clone();
                #[allow(clippy::redundant_clone)]
                let phase = phase.clone();
                Callback::from(
                    move |(new_state, _grants, _tags): (
                        crate::game::GameState,
                        crate::game::store::Grants,
                        Vec<String>,
                    )| {
                        pending.set(Some(new_state));
                        phase.set(Phase::Menu);
                    },
                )
            };
            html! {
                <section class="panel retro-menu">
                    <crate::components::ui::outfitting_store::OutfittingStore
                        game_state={current_state}
                        on_continue={on_continue} />
                </section>
            }
        }
        Phase::Menu => {
            // Main menu actions wiring
            #[allow(clippy::redundant_clone)]
            let start_with_code_action = {
                let code = code.clone();
                let pending = pending_state.clone();
                let phase = phase.clone();
                let logs = logs.clone();
                let data = data.clone();
                let run_seed = run_seed.clone();
                let session_handle = session.clone();
                let endgame_cfg = (*endgame_config).clone();
                move || {
                    if let Some((is_deep, seed)) = decode_to_seed(&code) {
                        let mode = if is_deep {
                            GameMode::Deep
                        } else {
                            GameMode::Classic
                        };
                        let base = (*pending).clone().unwrap_or_default();
                        let gs = base.with_seed(seed, mode, (*data).clone());
                        let sess = session_from_state(gs, &endgame_cfg);
                        let mode_label = if is_deep {
                            crate::i18n::t("mode.deep")
                        } else {
                            crate::i18n::t("mode.classic")
                        };
                        let mut m = std::collections::HashMap::new();
                        m.insert("mode", mode_label.as_str());
                        logs.set(vec![crate::i18n::tr("log.run_begins", Some(&m))]);
                        run_seed.set(seed);
                        pending.set(Some(sess.state().clone()));
                        session_handle.set(Some(sess));
                        phase.set(Phase::Travel);
                    } else {
                        let entropy = js_sys::Date::now().to_bits();
                        let new_code = generate_code_from_entropy(false, entropy);
                        code.set(new_code.clone().into());
                        if let Some((_, seed)) = decode_to_seed(&new_code) {
                            let base = (*pending).clone().unwrap_or_default();
                            let gs = base.with_seed(seed, GameMode::Classic, (*data).clone());
                            let sess = session_from_state(gs, &endgame_cfg);
                            let mode_label = crate::i18n::t("mode.classic");
                            let mut m = std::collections::HashMap::new();
                            m.insert("mode", mode_label.as_str());
                            logs.set(vec![crate::i18n::tr("log.run_begins", Some(&m))]);
                            run_seed.set(seed);
                            pending.set(Some(sess.state().clone()));
                            session_handle.set(Some(sess));
                            phase.set(Phase::Travel);
                        }
                    }
                }
            };

            let on_select = {
                let show_save_handle = show_save.clone();
                let show_settings_handle = show_settings.clone();
                let phase_handle = phase.clone();
                Callback::from(move |idx: u8| match idx {
                    1 => start_with_code_action(),
                    2 => phase_handle.set(Phase::Camp),
                    7 => show_save_handle.set(true),
                    8 => show_settings_handle.set(true),
                    0 => phase_handle.set(Phase::Boot),
                    3..=6 | 9..=u8::MAX => {}
                })
            };
            html! {
                            <section class="panel retro-menu">
                    <header class="retro-header" role="banner">
                                    <div class="header-with-controls">
                                        <div class="header-center">
                                            <pre class="ascii-art">
            { "═══════════════════════════════" }<br/>
            { "D Y S T R A I L" }<br/>
            { "A Political Survival Adventure" }<br/>
            { "═══════════════════════════════" }
                                            </pre>
                                            <p class="muted" aria-live="polite">{ format!("{seed_label} {code}", seed_label = i18n::t("game.seed_label"), code = (*code).clone()) }</p>
                                        </div>
                                        <div class="header-controls-row">
                                            <div class="header-left">
                                                <nav aria-label={crate::i18n::t("nav.language")}>
                                                    <label for="menu-lang-select" class="sr-only">{crate::i18n::t("nav.language")}</label>
                                        <select id="menu-lang-select" onchange={on_lang_change} value={(*current_language).clone()}>
                                            <option value="en">{"English"}</option>
                                            <option value="zh">{"中文"}</option>
                                            <option value="hi">{"हिन्दी"}</option>
                                            <option value="es">{"Español"}</option>
                                            <option value="fr">{"Français"}</option>
                                            <option value="ar">{"العربية"}</option>
                                            <option value="bn">{"বাংলা"}</option>
                                            <option value="pt">{"Português"}</option>
                                            <option value="ru">{"Русский"}</option>
                                            <option value="ja">{"日本語"}</option>
                                            <option value="it">{"Italiano"}</option>
                                        </select>
                                                </nav>
                                            </div>
                                            <div class="header-right">
                                                <button id="menu-save-btn" onclick={{ let s=show_save.clone(); Callback::from(move |_| s.set(true)) }}>
                                                    {crate::i18n::t("save.header")}
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                            </header>
                                <img src="/static/img/logo.png" alt="Dystrail" loading="lazy" style="width:min(520px,80vw)"/>
                                <crate::components::ui::main_menu::MainMenu seed_text={Some((*code).to_string())} on_select={Some(on_select)} />
                            </section>
                        }
        }
        Phase::Travel => (*session).clone().map_or_else(Html::default, |sess| {
            let snapshot = sess.state().clone();
            let stats = snapshot.stats.clone();
            let day = snapshot.day;
            let region = snapshot.region;
            let exec_order = snapshot.current_order;
            let persona_id = snapshot.persona_id.clone();
            let weather_badge = build_weather_badge(&snapshot, &weather_config);
            let state_rc = Rc::new(snapshot);
            let pacing_config_rc = Rc::new((*pacing_config).clone());
            html! {
                <>
                    <crate::components::ui::stats_bar::StatsBar
                        stats={stats}
                        day={day}
                        region={region}
                        exec_order={exec_order}
                        persona_id={persona_id}
                        weather={Some(weather_badge)}
                    />
                    <crate::components::ui::travel_panel::TravelPanel
                        on_travel={do_travel}
                        logs={(*logs).clone()}
                        game_state={Some(state_rc)}
                        pacing_config={pacing_config_rc}
                        on_pace_change={on_pace_change}
                        on_diet_change={on_diet_change}
                    />
                    { if data_ready { Html::default() } else { html! { <p class="muted" role="status">{ i18n::t("ui.loading_encounters") }</p> } } }
                </>
            }
        }),
        Phase::Camp => (*session).clone().map_or_else(Html::default, |sess| {
            let snapshot = sess.state().clone();
            let stats = snapshot.stats.clone();
            let day = snapshot.day;
            let region = snapshot.region;
            let exec_order = snapshot.current_order;
            let persona_id = snapshot.persona_id.clone();
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
                <>
                    <crate::components::ui::stats_bar::StatsBar
                        stats={stats}
                        day={day}
                        region={region}
                        exec_order={exec_order}
                        persona_id={persona_id}
                        weather={Some(weather_badge)}
                    />
                    <crate::components::ui::camp_panel::CampPanel
                        game_state={camp_state}
                        camp_config={camp_config_rc}
                        on_state_change={{
                            let session_handle = session.clone();
                            let pending_state = pending_state.clone();
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
                </>
            }
        }),
        Phase::Encounter => (*session).clone().map_or_else(Html::default, |sess| {
            sess.state().current_encounter.as_ref().map_or_else(
                || {
                    if data_ready {
                        Html::default()
                    } else {
                        html! { <p class="muted" role="status">{ i18n::t("ui.loading_encounters") }</p> }
                    }
                },
                |enc| {
                    let snapshot = sess.state().clone();
                    let persona_id = snapshot.persona_id.clone();
                    let weather_badge = build_weather_badge(&snapshot, &weather_config);
                    html! {
                        <>
                            <crate::components::ui::stats_bar::StatsBar
                                stats={snapshot.stats.clone()}
                                day={snapshot.day}
                                region={snapshot.region}
                                exec_order={snapshot.current_order}
                                persona_id={persona_id}
                                weather={Some(weather_badge)}
                            />
                            <crate::components::ui::encounter_card::EncounterCard
                                encounter={enc.clone()}
                                on_choice={on_choice.clone()}
                            />
                        </>
                    }
                },
            )
        }),
        Phase::Boss => (*session).clone().map_or_else(Html::default, |sess| {
            let gs = sess.state().clone();
            let cfg = (*boss_config).clone();
            let persona_id = gs.persona_id.clone();
                let mut chance = f64::from(cfg.base_victory_chance);
                chance += f64::from(gs.stats.credibility) * f64::from(cfg.credibility_weight);
                chance += f64::from(gs.stats.sanity) * f64::from(cfg.sanity_weight);
                chance += f64::from(gs.stats.supplies) * f64::from(cfg.supplies_weight);
                chance += f64::from(gs.stats.allies) * f64::from(cfg.allies_weight);
                chance -= f64::from(gs.stats.pants) * f64::from(cfg.pants_penalty_weight);
                chance = chance.clamp(f64::from(cfg.min_chance), f64::from(cfg.max_chance));
                let chance_pct = format!("{:.1}", chance * 100.0);

                let mut rounds_map: std::collections::HashMap<&str, &str> =
                    std::collections::HashMap::new();
                let rounds_value = cfg.rounds.to_string();
                let passes_value = cfg.passes_required.to_string();
                rounds_map.insert("rounds", rounds_value.as_str());
                rounds_map.insert("passes", passes_value.as_str());
                let rounds_text = i18n::tr("boss.stats.rounds", Some(&rounds_map));

                let mut chance_map: std::collections::HashMap<&str, &str> =
                    std::collections::HashMap::new();
                chance_map.insert("chance", chance_pct.as_str());
                let chance_text = i18n::tr("boss.stats.chance", Some(&chance_map));

                let sanity_text = if cfg.sanity_loss_per_round > 0 {
                    let mut map: std::collections::HashMap<&str, &str> =
                        std::collections::HashMap::new();
                    let delta = format!("{:+}", -cfg.sanity_loss_per_round);
                    map.insert("sanity", delta.as_str());
                    Some(i18n::tr("boss.stats.sanity", Some(&map)))
                } else {
                    None
                };

                let pants_text = if cfg.pants_gain_per_round > 0 {
                    let mut map: std::collections::HashMap<&str, &str> =
                        std::collections::HashMap::new();
                    let delta = format!("{:+}", cfg.pants_gain_per_round);
                    map.insert("pants", delta.as_str());
                    Some(i18n::tr("boss.stats.pants", Some(&map)))
                } else {
                    None
                };

                let weather_badge = build_weather_badge(&gs, &weather_config);
            html! {
                <>
                    <crate::components::ui::stats_bar::StatsBar
                        stats={gs.stats.clone()}
                        day={gs.day}
                        region={gs.region}
                        exec_order={gs.current_order}
                        persona_id={persona_id}
                        weather={Some(weather_badge)}
                    />
                    <section class="panel boss-phase">
                        <h2>{ i18n::t("boss.title") }</h2>
                        <div class="encounter-desc">
                            <p>{ i18n::t("boss.phases_hint") }</p>
                            <ul class="boss-stats">
                                <li>{ rounds_text }</li>
                                { sanity_text.map_or_else(
                                    Html::default,
                                    |text| html! { <li>{ text }</li> },
                                ) }
                                { pants_text.map_or_else(
                                    Html::default,
                                    |text| html! { <li>{ text }</li> },
                                ) }
                                <li>{ chance_text }</li>
                            </ul>
                            <p class="muted">{ i18n::t("boss.reminder") }</p>
                        </div>
                        <div class="controls">
                            <button class="retro-btn-primary" onclick={boss_act}>{ i18n::t("boss.begin") }</button>
                        </div>
                    </section>
                </>
            }
        }),
        Phase::Result => (*session).clone().map_or_else(Html::default, |sess| {
            let result_state = sess.state().clone();
            let result_config_data = (*result_config).clone();
            let boss_won = result_state.boss_victory;

                let session_for_replay = session.clone();
                let pending_state_for_replay = pending_state.clone();
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
                let pending_state_for_new_run = pending_state.clone();
                let on_new_run = Callback::from(move |()| {
                    pending_state_for_new_run.set(Some(GameState::default()));
                    session_for_new_run.set(None);
                });

                let session_for_title = session.clone();
                let pending_state_for_title = pending_state.clone();
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

            html! { <crate::components::ui::result_screen::ResultScreen
                game_state={result_state}
                result_config={result_config_data}
                boss_won={boss_won}
                on_replay_seed={on_replay_seed}
                on_new_run={on_new_run}
                on_title={on_title}
                on_export={on_export}
            /> }
        }),
    };

    html! {
        <main id="main" role="main">
            <style>{ crate::a11y::visible_focus_css() }</style>
            { html!{ <crate::components::ui::save_drawer::SaveDrawer open={open_save} on_close={on_close_save} on_save={on_save_cb} on_load={on_load_cb} on_export={on_export_cb} on_import={on_import_cb} return_focus_id={Some(AttrValue::from("menu-save-btn"))} /> } }
            { html!{ <crate::components::ui::settings_dialog::SettingsDialog open={*show_settings} on_close={{ let s=show_settings.clone(); Callback::from(move |()| s.set(false)) }} /> } }
            { main_view }
            <crate::components::footer::Footer />
        </main>
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
        use crate::routes::Route;

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
