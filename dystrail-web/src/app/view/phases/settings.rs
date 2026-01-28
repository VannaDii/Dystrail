use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::pages::settings::SettingsPage;
use yew::prelude::*;

pub fn render_settings(state: &AppState, handlers: &AppHandlers) -> Html {
    let on_back = {
        let phase = state.phase.clone();
        Callback::from(move |()| phase.set(Phase::Menu))
    };

    html! {
        <SettingsPage
            current_lang={(*state.current_language).clone()}
            high_contrast={*state.high_contrast}
            on_lang_change={handlers.lang_change.clone()}
            on_toggle_hc={handlers.toggle_hc.clone()}
            on_back={on_back}
        />
    }
}
