use crate::game::CampConfig;
use crate::game::boss::BossConfig;
use crate::game::data::EncounterData;
use crate::game::pacing::PacingConfig;
use crate::game::seed::{decode_to_seed, encode_friendly, generate_code_from_entropy};
use crate::game::state::{GameMode, GameState, Region};
use crate::game::weather::WeatherConfig;
use crate::game::{ResultConfig, load_result_config};
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
    let weather_config = use_state(WeatherConfig::default_config);
    let camp_config = use_state(CampConfig::default_config);
    let boss_config = use_state(BossConfig::load_from_static);
    let result_config = use_state(ResultConfig::default);
    let state = use_state(|| None::<GameState>);
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
        let nav_opt = navigator.clone();
        let route = route.clone();
        use_effect_with(phase.clone(), move |phase| {
            if let Some(nav) = nav_opt.as_ref() {
                let new_route = Route::from_phase(phase);
                // Only update route if it's different to prevent circular updates
                if new_route != route {
                    nav.push(&new_route);
                }
            }
        });
    }

    // Sync phase with route (only when route changes from URL navigation)
    {
        let phase = phase.clone();
        use_effect_with(route.clone(), move |route| {
            if let Some(new_phase) = route.to_phase() {
                // Only update phase if it's different to prevent circular updates
                if new_phase != *phase {
                    phase.set(new_phase);
                }
            }
        });
    }

    {
        let phase = phase.clone();
        let data = data.clone();
        let pacing_config = pacing_config.clone();
        let weather_config = weather_config.clone();
        let camp_config = camp_config.clone();
        let result_config = result_config.clone();
        use_effect_with((), move |()| {
            wasm_bindgen_futures::spawn_local(async move {
                let loaded_data = EncounterData::load_from_static();
                let loaded_pacing = PacingConfig::load_from_static();
                let loaded_weather = WeatherConfig::load_from_static();
                let loaded_camp = CampConfig::load_from_static();
                let loaded_result = load_result_config().unwrap_or_default();
                data.set(loaded_data);
                pacing_config.set(loaded_pacing);
                weather_config.set(loaded_weather);
                camp_config.set(loaded_camp);
                result_config.set(loaded_result);
                phase.set(Phase::Persona);
            });
            || {}
        });
    }

    #[allow(unused_variables)]
    let on_code_change = {
        let code = code.clone();
        let code_valid = code_valid.clone();
        Callback::from(move |v: String| {
            let v_up = v.trim().to_ascii_uppercase();
            let valid = regex::Regex::new(r"^(CL|DP)-[A-Z0-9]+\d{2}$")
                .map(|re| re.is_match(&v_up))
                .unwrap_or(false);
            code.set(v_up.into());
            code_valid.set(valid);
        })
    };

    let start_with_code = {
        let code = code.clone();
        let state = state.clone();
        let phase = phase.clone();
        let logs = logs.clone();
        let data = data.clone();
        let run_seed = run_seed.clone();
        Callback::from(move |()| {
            // Try to decode the share code first
            if let Some((is_deep, seed)) = decode_to_seed(&code) {
                let mode = if is_deep {
                    GameMode::Deep
                } else {
                    GameMode::Classic
                };
                let base = (*state).clone().unwrap_or_default();
                let gs = base.with_seed(seed, mode, (*data).clone());
                let mode_label = if is_deep {
                    crate::i18n::t("mode.deep")
                } else {
                    crate::i18n::t("mode.classic")
                };
                let mut m = std::collections::HashMap::new();
                m.insert("mode", mode_label.as_str());
                logs.set(vec![crate::i18n::tr("log.run_begins", Some(&m))]);
                state.set(Some(gs));
                run_seed.set(seed);
                phase.set(Phase::Travel);
            } else {
                // Fallback to classic mode if code decode fails
                let entropy = js_sys::Date::now().to_bits();
                let new_code = generate_code_from_entropy(false, entropy);
                code.set(new_code.clone().into());
                if let Some((_, seed)) = decode_to_seed(&new_code) {
                    let base = (*state).clone().unwrap_or_default();
                    let gs = base.with_seed(seed, GameMode::Classic, (*data).clone());
                    let mode_label = crate::i18n::t("mode.classic");
                    let mut m = std::collections::HashMap::new();
                    m.insert("mode", mode_label.as_str());
                    logs.set(vec![crate::i18n::tr("log.run_begins", Some(&m))]);
                    state.set(Some(gs));
                    run_seed.set(seed);
                    phase.set(Phase::Travel);
                }
            }
        })
    };

    let start_mode = {
        let code = code.clone();
        let state = state.clone();
        let phase = phase.clone();
        let logs = logs.clone();
        let data = data.clone();
        let run_seed = run_seed.clone();
        Callback::from(move |is_deep: bool| {
            let entropy = js_sys::Date::now().to_bits();
            let new_code = generate_code_from_entropy(is_deep, entropy);
            code.set(new_code.clone().into());
            if let Some((deep, seed)) = decode_to_seed(&new_code) {
                let mode = if deep {
                    GameMode::Deep
                } else {
                    GameMode::Classic
                };
                let base = (*state).clone().unwrap_or_default();
                let gs = base.with_seed(seed, mode, (*data).clone());
                let mode_label = if deep {
                    crate::i18n::t("mode.deep")
                } else {
                    crate::i18n::t("mode.classic")
                };
                let mut m = std::collections::HashMap::new();
                m.insert("mode", mode_label.as_str());
                logs.set(vec![crate::i18n::tr("log.run_begins", Some(&m))]);
                state.set(Some(gs));
                run_seed.set(seed);
                phase.set(Phase::Travel);
            }
        })
    };

    let do_travel = {
        let state = state.clone();
        let logs = logs.clone();
        let phase = phase.clone();
        let pacing_cfg = (*pacing_config).clone();
        Callback::from(move |()| {
            if let Some(mut gs) = (*state).clone() {
                // Apply pace and diet effects before traveling
                gs.apply_pace_and_diet(&pacing_cfg);
                let (ended, info_key, _) = gs.travel_next_leg();
                let mut lg = (*logs).clone();
                lg.push(crate::i18n::t(&info_key));
                if ended || gs.stats.pants >= 100 {
                    phase.set(Phase::Result);
                } else if gs.current_encounter.is_some() {
                    phase.set(Phase::Encounter);
                } else if matches!(gs.region, Region::Beltway) && gs.day > 12 {
                    phase.set(Phase::Boss);
                }
                logs.set(lg);
                state.set(Some(gs));
            }
        })
    };

    let on_pace_change = {
        let state = state.clone();
        Callback::from(move |new_pace: String| {
            if let Some(mut gs) = (*state).clone() {
                gs.pace = new_pace;
                state.set(Some(gs));
            }
        })
    };

    let on_diet_change = {
        let state = state.clone();
        Callback::from(move |new_diet: String| {
            if let Some(mut gs) = (*state).clone() {
                gs.diet = new_diet;
                state.set(Some(gs));
            }
        })
    };

    let on_choice = {
        let state = state.clone();
        let phase = phase.clone();
        let logs = logs.clone();
        Callback::from(move |idx: usize| {
            if let Some(mut gs) = (*state).clone() {
                let mut lg = (*logs).clone();
                gs.apply_choice(idx);
                lg.push(format!("Chose option {}", idx + 1));
                phase.set(Phase::Travel);
                logs.set(lg);
                state.set(Some(gs));
            }
        })
    };

    let boss_act = {
        let state = state.clone();
        let phase = phase.clone();
        let result = result.clone();
        let boss_config = boss_config.clone();
        Callback::from(move |_| {
            if let Some(mut gs) = (*state).clone() {
                let cfg = (*boss_config).clone();
                let out = crate::game::boss::run_boss_minigame(&mut gs, &cfg);
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
                result.set(Some((title, summary)));
                phase.set(Phase::Result);
                state.set(Some(gs));
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
        let state = state.clone();
        let logs = logs.clone();
        Callback::from(move |()| {
            if let Some(gs) = (*state).clone() {
                gs.save();
                let mut l = (*logs).clone();
                l.push(i18n::t("save.saved"));
                logs.set(l);
            }
        })
    };
    let on_load_cb = {
        let state = state.clone();
        let data = data.clone();
        let logs = logs.clone();
        let phase = phase.clone();
        let run_seed = run_seed.clone();
        Callback::from(move |()| {
            if let Some(mut gs) = GameState::load() {
                gs = gs.rehydrate((*data).clone());
                run_seed.set(gs.seed);
                state.set(Some(gs));
                let mut l = (*logs).clone();
                l.push(i18n::t("save.loaded"));
                logs.set(l);
                phase.set(Phase::Travel);
            }
        })
    };
    let on_export_cb = {
        let state = state.clone();
        Callback::from(move |()| {
            if let Some(gs) = (*state).clone()
                && let Ok(text) = serde_json::to_string(&gs)
                && let Some(win) = web_sys::window()
            {
                let nav = win.navigator();
                let cb = nav.clipboard();
                let _ = cb.write_text(&text);
            }
        })
    };
    let on_import_cb = {
        let state = state.clone();
        let data = data.clone();
        let logs = logs.clone();
        let run_seed = run_seed.clone();
        let phase = phase.clone();
        Callback::from(move |txt: String| {
            if let Ok(mut gs) = serde_json::from_str::<GameState>(&txt) {
                gs = gs.rehydrate((*data).clone());
                run_seed.set(gs.seed);
                state.set(Some(gs));
                let mut l = (*logs).clone();
                l.push(i18n::t("save.loaded"));
                logs.set(l);
                phase.set(Phase::Travel);
            } else {
                let mut l = (*logs).clone();
                l.push(i18n::t("save.error"));
                logs.set(l);
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
        Phase::Boot => html! {
            <section class="panel boot-screen" aria-busy="true" aria-live="polite">
                <img src="/static/img/logo.png" alt="Dystrail" loading="eager" style="width:min(520px,80vw)"/>
                <div class="bar-wrap" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow="100"><div class="bar-fill" style="width:100%"/></div>
                <p class="muted">{ i18n::t("ui.cta_start") }</p>
            </section>
        },
        Phase::Persona => {
            // On-persona selected callback
            let on_selected = {
                let state = state.clone();
                Callback::from(move |per: crate::game::personas::Persona| {
                    let mut gs = (*state).clone().unwrap_or_default();
                    gs.apply_persona(&per);
                    state.set(Some(gs));
                })
            };
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
            let current_state = (*state).clone().unwrap_or_default();
            let on_continue = {
                let state = state.clone();
                let phase = phase.clone();
                Callback::from(
                    move |(new_state, _grants, _tags): (
                        crate::game::GameState,
                        crate::game::store::Grants,
                        Vec<String>,
                    )| {
                        state.set(Some(new_state));
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
            let on_select = {
                let start_with_code = start_with_code.clone();
                let _start_mode = start_mode.clone();
                let show_save = show_save.clone();
                let show_settings = show_settings.clone();
                let phase = phase.clone();
                Callback::from(move |idx: u8| {
                    match idx {
                        1 => {
                            // Travel: always try start with code first, fall back to classic mode
                            start_with_code.emit(());
                        }
                        2 => {
                            // Camp
                            phase.set(Phase::Camp);
                        }
                        7 => {
                            show_save.set(true);
                        }
                        8 => {
                            show_settings.set(true);
                        }
                        0 => {
                            phase.set(Phase::Boot);
                        }
                        3..=6 | 9..=u8::MAX => {}
                    }
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
        Phase::Travel => {
            if let Some(gs) = (*state).clone() {
                let pacing_config_rc = Rc::new((*pacing_config).clone());
                html! {
                    <>
                        <crate::components::ui::stats_bar::StatsBar stats={gs.stats.clone()} day={gs.day} region={gs.region} exec_order={Some(gs.current_order)} />
                        <crate::components::ui::travel_panel::TravelPanel
                            on_travel={do_travel}
                            logs={(*logs).clone()}
                            game_state={(*state).clone().map(Rc::new)}
                            pacing_config={pacing_config_rc}
                            on_pace_change={on_pace_change}
                            on_diet_change={on_diet_change}
                        />
                        { if data_ready { Html::default() } else { html!{ <p class="muted" role="status">{ i18n::t("ui.loading_encounters") }</p> } } }
                    </>
                }
            } else {
                Html::default()
            }
        }
        Phase::Camp => {
            if let Some(gs) = (*state).clone() {
                let camp_config_rc = Rc::new((*camp_config).clone());
                html! {
                    <>
                        <crate::components::ui::stats_bar::StatsBar stats={gs.stats.clone()} day={gs.day} region={gs.region} exec_order={Some(gs.current_order)} />
                        <crate::components::ui::camp_panel::CampPanel
                            game_state={Rc::new(gs.clone())}
                            camp_config={camp_config_rc}
                            on_state_change={{
                                let state = state.clone();
                                Callback::from(move |new_state| {
                                    state.set(Some(new_state));
                                })
                            }}
                            on_close={{
                                let phase = phase.clone();
                                Callback::from(move |()| phase.set(Phase::Menu))
                            }}
                        />
                    </>
                }
            } else {
                Html::default()
            }
        }
        Phase::Encounter => {
            if let Some(gs) = (*state).clone() {
                if let Some(enc) = gs.current_encounter.clone() {
                    html! { <crate::components::ui::encounter_card::EncounterCard encounter={enc} on_choice={on_choice} /> }
                } else if data_ready {
                    Html::default()
                } else {
                    html! { <p class="muted" role="status">{ i18n::t("ui.loading_encounters") }</p> }
                }
            } else {
                Html::default()
            }
        }
        Phase::Boss => {
            if let Some(gs) = (*state).clone() {
                let cfg = (*boss_config).clone();
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

                html! {
                    <section class="panel boss-phase">
                        <h2>{ i18n::t("boss.title") }</h2>
                        <div class="encounter-desc">
                            <p>{ i18n::t("boss.phases_hint") }</p>
                            <ul class="boss-stats">
                                <li>{ rounds_text }</li>
                                { if let Some(text) = sanity_text {
                                    html! { <li>{ text }</li> }
                                } else { Html::default() } }
                                { if let Some(text) = pants_text {
                                    html! { <li>{ text }</li> }
                                } else { Html::default() } }
                                <li>{ chance_text }</li>
                            </ul>
                            <p class="muted">{ i18n::t("boss.reminder") }</p>
                        </div>
                        <div class="controls">
                            <button class="retro-btn-primary" onclick={boss_act}>{ i18n::t("boss.begin") }</button>
                        </div>
                    </section>
                }
            } else {
                Html::default()
            }
        }
        Phase::Result => {
            if let Some(gs) = (*state).clone() {
                let result_config_data = (*result_config).clone();
                let boss_won = gs.boss_victory;

                let on_replay_seed = {
                    let seed = *run_seed;
                    let state = state.clone();
                    Callback::from(move |()| {
                        // Use default and set seed
                        let new_game = GameState {
                            seed,
                            ..GameState::default()
                        };
                        state.set(Some(new_game));
                    })
                };

                let on_new_run = {
                    let state = state.clone();
                    Callback::from(move |()| {
                        state.set(Some(GameState::default()));
                    })
                };

                let on_title = {
                    let phase = phase.clone();
                    Callback::from(move |()| {
                        phase.set(Phase::Boot);
                    })
                };

                let on_export = {
                    let seed = *run_seed;
                    let game_state = gs.clone();
                    Callback::from(move |()| {
                        let code_str = encode_friendly(game_state.mode.is_deep(), seed);
                        if let Some(win) = web_sys::window() {
                            let nav = win.navigator();
                            let cb = nav.clipboard();
                            let _ = cb.write_text(&code_str);
                        }
                    })
                };

                html! { <crate::components::ui::result_screen::ResultScreen
                    game_state={gs}
                    result_config={result_config_data}
                    boss_won={boss_won}
                    on_replay_seed={on_replay_seed}
                    on_new_run={on_new_run}
                    on_title={on_title}
                    on_export={on_export}
                /> }
            } else {
                Html::default()
            }
        }
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
