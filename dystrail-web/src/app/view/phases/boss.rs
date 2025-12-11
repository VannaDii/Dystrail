use crate::app::phase::build_weather_badge;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::pages::boss::BossPage;
use yew::prelude::*;

pub fn render_boss(state: &AppState, handlers: &AppHandlers) -> Html {
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
