pub(crate) mod handlers;
pub(crate) mod phases;

pub use handlers::AppHandlers;

use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers as Handlers;
use crate::router::Route;
use yew::prelude::*;
use yew_router::prelude::Navigator;

pub fn render_app(state: &AppState, route: Option<&Route>, navigator: Option<Navigator>) -> Html {
    let handlers: Handlers = AppHandlers::new(state, navigator);
    let screen = if state.pending_turn.is_none() && crate::app::policy_bulletin::is_pending(state) {
        "policy-bulletin"
    } else if *state.phase == crate::app::Phase::Map {
        "map"
    } else if state.pending_turn.is_some() {
        "traveling"
    } else if state.aftermath.is_some() {
        "aftermath"
    } else if *state.phase == crate::app::Phase::Town
        && state
            .session
            .as_ref()
            .is_some_and(|s| s.state().continuity.route_services.trading)
    {
        "trade"
    } else {
        state.phase.screen_name()
    };
    let main_view = phases::render_main_view(state, &handlers, route);

    let open_save_header = {
        let running = state.travel_running.clone();
        let show_save = state.show_save.clone();
        let focus_target = state.save_focus_target.clone();
        Callback::from(move |()| {
            running.set(false);
            focus_target.set(AttrValue::from("game-menu-button"));
            show_save.set(true);
        })
    };

    let on_close_save = {
        let s = state.show_save.clone();
        Callback::from(move |()| s.set(false))
    };

    let on_close_settings = {
        let show_settings = state.show_settings.clone();
        Callback::from(move |()| show_settings.set(false))
    };

    html! {
        <ContextProvider<crate::i18n::Language> context={crate::i18n::Language((*state.current_language).clone())}>
        <ContextProvider<crate::app::journey_panel::PanelContext> context={crate::app::journey_panel::context(state,&handlers)}>
        <ContextProvider<crate::app::help::HelpPreference> context={crate::app::help::HelpPreference(*state.help_enabled)}>
        <crate::app::weather_status::WeatherStatus weather={state.session.as_ref().map(|s|s.state().weather_state.today)} policy={state.session.as_ref().and_then(|s|s.state().current_order)}>
        <div class={classes!("game-shell", (state.session.as_ref().is_some_and(|s|s.state().mode.is_deep()) || state.code.starts_with("DP-")).then_some("deep-mode"))}>
            <crate::components::header::Header
                on_open_save={open_save_header}
                can_abandon={state.session.is_some() && *state.phase!=crate::app::Phase::Result}
                on_abandon={{let app=state.clone();Callback::from(move |()|{app.travel_running.set(false);app.show_abandon.set(true);})}}
                on_save={handlers.save.clone()}
                status={(*state.save_status).clone()}
                on_status_clear={{let status=state.save_status.clone();Callback::from(move |()|status.set(String::new()))}}
                on_lang_change={handlers.lang_change.clone()}
                current_lang={(*state.current_language).clone()}
                high_contrast={*state.high_contrast}
                on_toggle_hc={handlers.toggle_hc.clone()}
                help_enabled={*state.help_enabled}
                on_toggle_help={crate::app::help::change(state.help_enabled.clone())}
            />
            <main id="main" role="main" tabindex="-1" data-screen={screen}>
                <style>{ crate::a11y::visible_focus_css() }</style>
                { html!{ <crate::components::ui::save_drawer::SaveDrawer status={(*state.save_status).clone()} open={*state.show_save} on_close={on_close_save} on_save={handlers.save.clone()} on_load={handlers.load.clone()} on_export={handlers.export_state.clone()} on_download={handlers.download_state.clone()} can_save={state.session.is_some() || state.pending_turn.is_some()} on_import={handlers.import_state.clone()} return_focus_id={Some((*state.save_focus_target).clone())} /> } }
                { html!{ <crate::components::ui::settings_dialog::SettingsDialog open={*state.show_settings} on_close={on_close_settings.clone()} on_hc_changed={handlers.settings_hc_change.clone()} /> } }

                { main_view }
                if *state.show_abandon {{crate::app::abandon::render(state)}}
                <crate::components::footer::Footer />
            </main>
        </div>
        </crate::app::weather_status::WeatherStatus>
        </ContextProvider<crate::app::help::HelpPreference>>
        </ContextProvider<crate::app::journey_panel::PanelContext>>
        </ContextProvider<crate::i18n::Language>>
    }
}
