use crate::app::state::AppState;
use crate::i18n;
use yew::prelude::*;

pub fn render_seed_footer(
    state: &AppState,
    open_save: &Callback<MouseEvent>,
    open_settings: &Callback<MouseEvent>,
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
                        onclick={open_save.clone()}
                    >
                        { i18n::t("save.header") }
                    </button>
                    <button class="retro-btn-secondary" onclick={open_settings.clone()}>
                        { i18n::t("menu.settings") }
                    </button>
                </crate::components::ui::seed_footer::SeedFooter>
            }
        })
        .unwrap_or_default()
}
