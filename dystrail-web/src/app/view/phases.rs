use crate::app::phase::{Phase, build_weather_badge, session_from_state};
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::game::encode_friendly;
use crate::game::seed::{decode_to_seed, generate_code_from_entropy};
use crate::game::state::{GameMode, GameState};
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

pub fn render_main_view(state: &AppState, handlers: &AppHandlers, route: Option<&Route>) -> Html {
    let not_found = matches!(route, None | Some(Route::NotFound));
    if not_found {
        return html! { <NotFound on_go_home={handlers.go_home.clone()} /> };
    }

    match *state.phase {
        Phase::Boot => {
            let boot_logo_src: AttrValue = crate::paths::asset_path("static/img/logo.png").into();
            html! {
                <BootPage
                    logo_src={boot_logo_src}
                    ready={*state.boot_ready}
                    preload_progress={*state.preload_progress}
                    on_begin={handlers.begin_boot.clone()}
                />
            }
        }
        Phase::Persona => render_persona(state),
        Phase::Outfitting => render_outfitting(state),
        Phase::Menu => render_menu(state),
        Phase::Travel => render_travel(state, handlers),
        Phase::Camp => render_camp(state),
        Phase::Encounter => render_encounter(state, handlers),
        Phase::Boss => render_boss(state, handlers),
        Phase::Result => render_result(state),
    }
}

fn render_persona(state: &AppState) -> Html {
    let on_selected = {
        let pending = state.pending_state.clone();
        Callback::from(move |per: crate::game::personas::Persona| {
            let mut gs = (*pending).clone().unwrap_or_default();
            gs.apply_persona(&per);
            pending.set(Some(gs));
        })
    };
    let on_continue = {
        let phase = state.phase.clone();
        Callback::from(move |()| phase.set(Phase::Outfitting))
    };
    html! { <PersonaPage {on_selected} {on_continue} /> }
}

fn render_outfitting(state: &AppState) -> Html {
    let current_state = (*state.pending_state).clone().unwrap_or_default();
    let on_continue = {
        let pending_handle = state.pending_state.clone();
        let phase_handle = state.phase.clone();
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

fn render_menu(state: &AppState) -> Html {
    let start_with_code_action = {
        let code_handle = state.code.clone();
        let pending_handle = state.pending_state.clone();
        let phase_handle = state.phase.clone();
        let logs_handle = state.logs.clone();
        let data_handle = state.data.clone();
        let run_seed_handle = state.run_seed.clone();
        let session_handle = state.session.clone();
        let endgame_cfg = (*state.endgame_config).clone();
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
                    let gs = base.with_seed(seed, GameMode::Classic, (*data_handle).clone());
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
        let phase_handle = state.phase.clone();
        let show_save_handle = state.show_save.clone();
        let show_settings_handle = state.show_settings.clone();
        let save_focus = state.save_focus_target.clone();
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

    let menu_logo_src: AttrValue = crate::paths::asset_path("static/img/logo.png").into();
    html! {
        <MenuPage
            code={(*state.code).clone()}
            logo_src={menu_logo_src}
            {on_action}
        />
    }
}

fn render_travel(state: &AppState, handlers: &AppHandlers) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let snapshot = sess.state().clone();
        let weather_badge = build_weather_badge(&snapshot, &state.weather_config);
        let state_rc = Rc::new(snapshot);
        let pacing_config_rc = Rc::new((*state.pacing_config).clone());
        html! {
            <TravelPage
                state={state_rc}
                logs={(*state.logs).clone()}
                pacing_config={pacing_config_rc}
                weather_badge={weather_badge}
                data_ready={state.data_ready()}
                on_travel={handlers.travel.clone()}
                on_pace_change={handlers.pace_change.clone()}
                on_diet_change={handlers.diet_change.clone()}
            />
        }
    })
}

fn render_camp(state: &AppState) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let snapshot = sess.state().clone();
        let weather_cfg = (*state.weather_config).clone();
        let weather_today = snapshot.weather_state.today;
        let weather_mitigated = weather_cfg
            .mitigation
            .get(&weather_today)
            .is_some_and(|mit| snapshot.inventory.tags.contains(&mit.tag));
        let weather_badge = crate::components::ui::stats_bar::WeatherBadge {
            weather: weather_today,
            mitigated: weather_mitigated,
        };
        let camp_state = Rc::new(snapshot);
        let camp_config_rc = Rc::new((*state.camp_config).clone());
        html! {
            <CampPage
                state={camp_state}
                camp_config={camp_config_rc}
                weather={weather_badge}
                on_state_change={{
                    let session_handle = state.session.clone();
                    let pending_state = state.pending_state.clone();
                    let endgame_cfg = (*state.endgame_config).clone();
                    Callback::from(move |new_state: GameState| {
                        let snapshot = new_state.clone();
                        let updated = session_from_state(new_state, &endgame_cfg);
                        pending_state.set(Some(snapshot));
                        session_handle.set(Some(updated));
                    })
                }}
                on_close={{
                    let phase_handle = state.phase.clone();
                    Callback::from(move |()| phase_handle.set(Phase::Menu))
                }}
            />
        }
    })
}

fn render_encounter(state: &AppState, handlers: &AppHandlers) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let snapshot = sess.state().clone();
        let weather_badge = build_weather_badge(&snapshot, &state.weather_config);
        html! {
            <EncounterPage
                state={Rc::new(snapshot)}
                weather={weather_badge}
                on_choice={handlers.encounter_choice.clone()}
            />
        }
    })
}

fn render_boss(state: &AppState, handlers: &AppHandlers) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let gs = sess.state().clone();
        let cfg = (*state.boss_config).clone();
        let weather_badge = build_weather_badge(&gs, &state.weather_config);
        html! {
            <BossPage
                state={gs}
                config={cfg}
                weather={weather_badge}
                on_begin={handlers.boss.clone()}
            />
        }
    })
}

fn render_result(state: &AppState) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let result_state = sess.state().clone();
        let result_config_data = (*state.result_config).clone();
        let boss_won = result_state.boss.outcome.victory;

        let session_for_replay = state.session.clone();
        let pending_state_for_replay = state.pending_state.clone();
        let seed_for_replay = *state.run_seed;
        let on_replay_seed = Callback::from(move |()| {
            let new_game = GameState {
                seed: seed_for_replay,
                ..GameState::default()
            };
            pending_state_for_replay.set(Some(new_game));
            session_for_replay.set(None);
        });

        let session_for_new_run = state.session.clone();
        let pending_state_for_new_run = state.pending_state.clone();
        let on_new_run = Callback::from(move |()| {
            pending_state_for_new_run.set(Some(GameState::default()));
            session_for_new_run.set(None);
        });

        let session_for_title = state.session.clone();
        let pending_state_for_title = state.pending_state.clone();
        let phase_for_title = state.phase.clone();
        let on_title = Callback::from(move |()| {
            pending_state_for_title.set(None);
            session_for_title.set(None);
            phase_for_title.set(Phase::Boot);
        });

        let on_export = {
            let seed = *state.run_seed;
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
    })
}

pub fn render_seed_footer(
    state: &AppState,
    open_save: Callback<MouseEvent>,
    open_settings: Callback<MouseEvent>,
) -> Html {
    state
        .session
        .as_ref()
        .as_ref()
        .map(|sess| {
            let seed_value = if *state.run_seed == 0 {
                sess.state().seed
            } else {
                *state.run_seed
            };
            let is_deep = sess.state().mode.is_deep();
            html! {
                <crate::components::ui::seed_footer::SeedFooter seed={seed_value} is_deep_mode={is_deep}>
                    <button
                        id="seed-save-btn"
                        class="retro-btn-secondary"
                        onclick={open_save}
                    >
                        { i18n::t("save.header") }
                    </button>
                    <button class="retro-btn-secondary" onclick={open_settings}>
                        { i18n::t("menu.settings") }
                    </button>
                </crate::components::ui::seed_footer::SeedFooter>
            }
        })
        .unwrap_or_default()
}
