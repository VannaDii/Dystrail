mod handlers;
mod phases;

pub use handlers::AppHandlers;

use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers as Handlers;
use crate::router::Route;
use yew::prelude::*;
use yew_router::prelude::Navigator;

pub fn render_app(state: &AppState, route: Option<&Route>, navigator: Option<Navigator>) -> Html {
    let handlers: Handlers = AppHandlers::new(state, navigator);
    let main_view = phases::render_main_view(state, &handlers, route);

    let open_save_header = {
        let show_save = state.show_save.clone();
        let focus_target = state.save_focus_target.clone();
        Callback::from(move |()| {
            focus_target.set(AttrValue::from("save-open-btn"));
            show_save.set(true);
        })
    };

    let open_save_nav = {
        let show_save = state.show_save.clone();
        Callback::from(move |_| show_save.set(true))
    };

    let open_settings_nav = {
        let show_settings = state.show_settings.clone();
        Callback::from(move |_| show_settings.set(true))
    };

    let on_close_save = {
        let s = state.show_save.clone();
        Callback::from(move |()| s.set(false))
    };

    let on_close_settings = {
        let show_settings = state.show_settings.clone();
        Callback::from(move |()| show_settings.set(false))
    };

    let seed_footer = {
        let focus_target = state.save_focus_target.clone();
        let show_save = state.show_save.clone();
        let open_settings = {
            let show_settings = state.show_settings.clone();
            Callback::from(move |_| show_settings.set(true))
        };
        let open_save_footer = Callback::from(move |_| {
            focus_target.set(AttrValue::from("seed-save-btn"));
            show_save.set(true);
        });
        phases::render_seed_footer(state, &open_save_footer, &open_settings)
    };

    html! {
        <>
            <crate::components::header::Header
                on_open_save={open_save_header}
                on_lang_change={handlers.lang_change.clone()}
                current_lang={(*state.current_language).clone()}
                high_contrast={*state.high_contrast}
                on_toggle_hc={handlers.toggle_hc.clone()}
            />
            <main id="main" role="main">
                <style>{ crate::a11y::visible_focus_css() }</style>
                { html!{ <crate::components::ui::save_drawer::SaveDrawer open={*state.show_save} on_close={on_close_save} on_save={handlers.save.clone()} on_load={handlers.load.clone()} on_export={handlers.export_state.clone()} on_import={handlers.import_state.clone()} return_focus_id={Some((*state.save_focus_target).clone())} /> } }
                { html!{ <crate::components::ui::settings_dialog::SettingsDialog open={*state.show_settings} on_close={on_close_settings.clone()} on_hc_changed={handlers.settings_hc_change.clone()} /> } }
                { main_view }
                <div class="panel-footer nav-footer" role="navigation" aria-label={crate::i18n::t("menu.title")}>
                    <button class="retro-btn-secondary" onclick={open_settings_nav}>
                        { crate::i18n::t("menu.settings") }
                    </button>
                    <button class="retro-btn-secondary" onclick={open_save_nav}>
                        { crate::i18n::t("save.header") }
                    </button>
                </div>
                { seed_footer }
                <crate::components::footer::Footer />
            </main>
        </>
    }
}
