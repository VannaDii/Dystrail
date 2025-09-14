use crate::game::data::EncounterData;
use crate::game::seed::{decode_to_seed, encode_friendly, generate_code_from_entropy};
use crate::game::state::{GameMode, GameState, Region};
use crate::i18n;
use crate::routes::Route;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Boot,
    Menu,
    Travel,
    Encounter,
    Boss,
    Result,
}

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
    let state = use_state(|| None::<GameState>);
    let logs = use_state(Vec::<String>::new);
    let result = use_state(|| None::<(String, String)>);
    let run_seed = use_state(|| 0_u64);
    let show_save = use_state(|| false);
    let show_settings = use_state(|| false);
    let current_language = use_state(crate::i18n::current_lang);
    let data_ready = !data.encounters.is_empty();

    // Add routing hooks
    let navigator = use_navigator().unwrap();
    let route = use_route::<Route>().unwrap_or(Route::Home);

    // Sync route with phase (only when phase changes programmatically)
    {
        let navigator = navigator.clone();
        let phase = phase.clone();
        let route = route.clone();
        use_effect_with(phase.clone(), move |phase| {
            let new_route = Route::from_phase(phase);
            // Only update route if it's different to prevent circular updates
            if new_route != route {
                navigator.push(&new_route);
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
        use_effect_with((), move |()| {
            wasm_bindgen_futures::spawn_local(async move {
                let loaded = EncounterData::load_from_static().await;
                data.set(loaded);
                phase.set(Phase::Menu);
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
            if let Some((is_deep, seed)) = decode_to_seed(&code) {
                let mode = if is_deep {
                    GameMode::Deep
                } else {
                    GameMode::Classic
                };
                let gs = GameState::default().with_seed(seed, mode, (*data).clone());
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
                let gs = GameState::default().with_seed(seed, mode, (*data).clone());
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
        Callback::from(move |()| {
            if let Some(mut gs) = (*state).clone() {
                let (ended, info_key) = gs.travel_next_leg();
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
        Callback::from(move |_| {
            if let Some(mut gs) = (*state).clone() {
                let out = crate::game::boss::run_boss_minigame(&mut gs);
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
            if let Some(gs) = (*state).clone() {
                if let Ok(text) = serde_json::to_string(&gs) {
                    if let Some(win) = web_sys::window() {
                        let nav = win.navigator();
                        let cb = nav.clipboard();
                        let _ = cb.write_text(&text);
                    }
                }
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
        Phase::Menu => {
            // Main menu actions wiring
            let on_select = {
                let start_with_code = start_with_code.clone();
                let start_mode = start_mode.clone();
                let show_save = show_save.clone();
                let show_settings = show_settings.clone();
                let phase = phase.clone();
                Callback::from(move |idx: u8| {
                    match idx {
                        1 => {
                            // Travel: prefer start with code if valid else classic
                            if *code_valid {
                                start_with_code.emit(());
                            } else {
                                start_mode.emit(false);
                            }
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
                        2..=6 | 9..=u8::MAX => {}
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
                                            <p class="muted" aria-live="polite">{ format!("{} {}", i18n::t("game.seed_label"), (*code).clone()) }</p>
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
                html! {
                    <>
                        <crate::components::ui::stats_bar::StatsBar stats={gs.stats.clone()} day={gs.day} region={gs.region} exec_order={Some(gs.current_order)} />
                        <crate::components::ui::travel_panel::TravelPanel on_travel={do_travel} logs={(*logs).clone()} />
                        { if data_ready { Html::default() } else { html!{ <p class="muted" role="status">{ i18n::t("ui.loading_encounters") }</p> } } }
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
        Phase::Boss => html! {
            <section class="panel">
                <h2>{ i18n::t("boss.title") }</h2>
                <div class="encounter-desc">
                    <p>{ i18n::t("boss.phases_hint") }</p>
                </div>
                <div class="controls">
                    <button class="retro-btn-primary" onclick={boss_act}>{ i18n::t("boss.begin") }</button>
                </div>
            </section>
        },
        Phase::Result => {
            if let Some(gs) = (*state).clone() {
                let code_str = encode_friendly(gs.mode.is_deep(), *run_seed);
                let (title, summary) = if gs.stats.pants >= 100 {
                    (
                        i18n::t("result.pants_emergency"),
                        i18n::t("result.pants_emergency_desc"),
                    )
                } else if let Some((t, s)) = (*result).clone() {
                    (t, s)
                } else {
                    (i18n::t("result.title"), i18n::t("result.thanks"))
                };
                let to_copy = format!("Dystrail — {title}: {code_str}");
                let on_share = Callback::from(move |()| {
                    if let Some(win) = web_sys::window() {
                        let nav = win.navigator();
                        let cb = nav.clipboard();
                        let _ = cb.write_text(&to_copy);
                    }
                });
                html! { <crate::components::ui::result_screen::ResultScreen title={title} summary={summary} seed_code={AttrValue::from(code_str)} on_share={on_share} /> }
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
