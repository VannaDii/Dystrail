mod handlers;
mod phases;

pub use handlers::AppHandlers;
pub use phases::render_crossing;

use crate::app::Phase;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers as Handlers;
use crate::router::Route;
use yew::prelude::*;
use yew_router::prelude::Navigator;

fn build_open_save(
    show_save: UseStateHandle<bool>,
    focus_target: UseStateHandle<AttrValue>,
    focus_id: AttrValue,
) -> Callback<()> {
    Callback::from(move |()| {
        focus_target.set(focus_id.clone());
        show_save.set(true);
    })
}

const fn is_meta_phase(phase: Phase) -> bool {
    matches!(
        phase,
        Phase::Boot | Phase::Menu | Phase::About | Phase::Settings
    )
}

fn render_header_section(
    show_header: bool,
    state: &AppState,
    handlers: &Handlers,
    open_save_header: Callback<()>,
) -> Html {
    if !show_header {
        return Html::default();
    }
    html! {
        <crate::components::header::Header
            on_open_save={open_save_header}
            on_lang_change={handlers.lang_change.clone()}
            current_lang={(*state.current_language).clone()}
            high_contrast={*state.high_contrast}
            on_toggle_hc={handlers.toggle_hc.clone()}
        />
    }
}

fn render_drawers(
    show_header: bool,
    state: &AppState,
    handlers: &Handlers,
    on_close_save: Callback<()>,
    on_close_settings: Callback<()>,
) -> Html {
    if !show_header {
        return Html::default();
    }
    html! {
        <>
            <crate::components::ui::save_drawer::SaveDrawer
                open={*state.show_save}
                on_close={on_close_save}
                on_save={handlers.save.clone()}
                on_load={handlers.load.clone()}
                on_export={handlers.export_state.clone()}
                on_import={handlers.import_state.clone()}
                return_focus_id={Some((*state.save_focus_target).clone())}
            />
            <crate::components::ui::settings_dialog::SettingsDialog
                open={*state.show_settings}
                on_close={on_close_settings}
                on_hc_changed={handlers.settings_hc_change.clone()}
            />
        </>
    }
}

fn render_nav_footer(
    show_nav: bool,
    open_settings_nav: Callback<MouseEvent>,
    open_save_nav: Callback<MouseEvent>,
) -> Html {
    if !show_nav {
        return Html::default();
    }
    html! {
        <div class="panel-footer nav-footer" role="navigation" aria-label={crate::i18n::t("menu.title")}>
            <button class="retro-btn-secondary" onclick={open_settings_nav}>
                { crate::i18n::t("menu.settings") }
            </button>
            <button class="retro-btn-secondary" onclick={open_save_nav}>
                { crate::i18n::t("save.header") }
            </button>
        </div>
    }
}

fn render_footer(show_header: bool) -> Html {
    if show_header {
        html! { <crate::components::footer::Footer /> }
    } else {
        Html::default()
    }
}

fn render_seed_footer(show_nav: bool, state: &AppState) -> Html {
    if !show_nav {
        return Html::default();
    }
    let show_save = state.show_save.clone();
    let open_settings = {
        let show_settings = state.show_settings.clone();
        Callback::from(move |_event: MouseEvent| show_settings.set(true))
    };
    let open_save_footer = build_open_save(
        show_save,
        state.save_focus_target.clone(),
        AttrValue::from("seed-save-btn"),
    )
    .reform(|_event: MouseEvent| ());
    phases::render_seed_footer(state, &open_save_footer, &open_settings)
}

pub fn render_app(state: &AppState, route: Option<&Route>, navigator: Option<Navigator>) -> Html {
    let handlers: Handlers = AppHandlers::new(state, navigator);
    let main_view = phases::render_main_view(state, &handlers, route);
    let phase = *state.phase;
    let is_meta = is_meta_phase(phase);
    let show_header = !is_meta;
    let has_session = (*state.session).is_some();
    let show_nav = has_session && !is_meta;

    let open_save_header = {
        build_open_save(
            state.show_save.clone(),
            state.save_focus_target.clone(),
            AttrValue::from("save-open-btn"),
        )
    };

    let open_save_nav = {
        let show_save = state.show_save.clone();
        Callback::from(move |_event: MouseEvent| show_save.set(true))
    };

    let open_settings_nav = {
        let show_settings = state.show_settings.clone();
        Callback::from(move |_event: MouseEvent| show_settings.set(true))
    };

    let on_close_save = {
        let s = state.show_save.clone();
        Callback::from(move |()| s.set(false))
    };

    let on_close_settings = {
        let show_settings = state.show_settings.clone();
        Callback::from(move |()| show_settings.set(false))
    };

    let seed_footer = render_seed_footer(show_nav, state);

    html! {
        <>
            { render_header_section(show_header, state, &handlers, open_save_header) }
            <main id="main" role="main">
                <style>{ crate::a11y::visible_focus_css() }</style>
                { render_drawers(show_header, state, &handlers, on_close_save, on_close_settings) }
                { main_view }
                { render_nav_footer(show_nav, open_settings_nav, open_save_nav) }
                { seed_footer }
                { render_footer(show_header) }
            </main>
        </>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::phase::Phase;
    use crate::game::data::EncounterData;
    use crate::game::state::GameMode;
    use crate::game::{EndgameTravelCfg, JourneySession, StrategyId};
    use futures::executor::block_on;
    use std::cell::Cell;
    use std::rc::Rc;
    use yew::LocalServerRenderer;

    #[function_component(RenderAppHarness)]
    fn render_app_harness() -> Html {
        crate::i18n::set_lang("en");
        let data = EncounterData::load_from_static();
        let base = crate::game::GameState::default().with_seed(7, GameMode::Classic, data.clone());
        let session = JourneySession::from_state(
            base.clone(),
            StrategyId::Balanced,
            &EndgameTravelCfg::default_config(),
        );
        let state = AppState {
            phase: use_state(|| Phase::Travel),
            code: use_state(|| AttrValue::from("CL-ORANGE42")),
            data: use_state(move || data),
            pacing_config: use_state(crate::game::pacing::PacingConfig::default_config),
            endgame_config: use_state(EndgameTravelCfg::default_config),
            weather_config: use_state(crate::game::weather::WeatherConfig::default_config),
            camp_config: use_state(crate::game::CampConfig::default_config),
            crossing_config: use_state(crate::game::CrossingConfig::default),
            boss_config: use_state(crate::game::boss::BossConfig::load_from_static),
            result_config: use_state(crate::game::ResultConfig::default),
            preload_progress: use_state(|| 100_u8),
            boot_ready: use_state(|| true),
            high_contrast: use_state(|| false),
            pending_state: use_state(|| Some(base.clone())),
            session: use_state(|| Some(session)),
            logs: use_state(Vec::<String>::new),
            run_seed: use_state(|| 7_u64),
            show_save: use_state(|| false),
            save_focus_target: use_state(|| AttrValue::from("save-open-btn")),
            show_settings: use_state(|| false),
            current_language: use_state(|| String::from("en")),
        };
        render_app(&state, Some(&Route::Travel), None)
    }

    #[test]
    fn render_app_builds_shell() {
        let html = block_on(LocalServerRenderer::<RenderAppHarness>::new().render());
        assert!(html.contains("nav-footer"));
        assert!(html.contains("save-open-btn"));
    }

    #[function_component(OpenSaveHarness)]
    fn open_save_harness() -> Html {
        let show_save = use_state(|| false);
        let focus_target = use_state(|| AttrValue::from("initial"));
        let invoked = use_mut_ref(|| false);
        let called = Rc::new(Cell::new(false));
        let called_ref = called.clone();
        let open_save = build_open_save(show_save, focus_target, AttrValue::from("focus-target"));
        let invoke = Callback::from(move |()| {
            called_ref.set(true);
            open_save.emit(());
        });

        if !*invoked.borrow() {
            *invoked.borrow_mut() = true;
            invoke.emit(());
        }

        html! { <div data-called={called.get().to_string()} /> }
    }

    #[test]
    fn build_open_save_executes_callback() {
        let html = block_on(LocalServerRenderer::<OpenSaveHarness>::new().render());
        assert!(html.contains("data-called=\"true\""));
    }

    #[function_component(RenderAppMetaHarness)]
    fn render_app_meta_harness() -> Html {
        crate::i18n::set_lang("en");
        let data = crate::game::data::EncounterData::empty();
        let state = AppState {
            phase: use_state(|| Phase::Menu),
            code: use_state(|| AttrValue::from("CL-ORANGE42")),
            data: use_state(move || data),
            pacing_config: use_state(crate::game::pacing::PacingConfig::default_config),
            endgame_config: use_state(crate::game::EndgameTravelCfg::default_config),
            weather_config: use_state(crate::game::weather::WeatherConfig::default_config),
            camp_config: use_state(crate::game::CampConfig::default_config),
            crossing_config: use_state(crate::game::CrossingConfig::default),
            boss_config: use_state(crate::game::boss::BossConfig::load_from_static),
            result_config: use_state(crate::game::ResultConfig::default),
            preload_progress: use_state(|| 0_u8),
            boot_ready: use_state(|| false),
            high_contrast: use_state(|| false),
            pending_state: use_state(|| None::<crate::game::GameState>),
            session: use_state(|| None::<crate::game::JourneySession>),
            logs: use_state(Vec::<String>::new),
            run_seed: use_state(|| 0_u64),
            show_save: use_state(|| false),
            save_focus_target: use_state(|| AttrValue::from("save-open-btn")),
            show_settings: use_state(|| false),
            current_language: use_state(|| String::from("en")),
        };
        render_app(&state, Some(&Route::Menu), None)
    }

    #[test]
    fn render_app_hides_shell_on_meta_routes() {
        let html = block_on(LocalServerRenderer::<RenderAppMetaHarness>::new().render());
        assert!(html.contains("menu-screen"));
        assert!(!html.contains("nav-footer"));
        assert!(!html.contains("save-open-btn"));
    }
}
