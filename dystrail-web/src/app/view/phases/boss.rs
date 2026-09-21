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
                key={crate::i18n::current_lang()}
                state={gs}
                config={cfg}
                weather={weather_badge}
                on_begin={handlers.boss.clone()}
                on_hearing={handlers.hearing.clone()}
                fast={*state.travel_speed == crate::app::flow::TravelSpeed::Fast}
                paused={*state.show_save || *state.show_settings || *state.show_abandon}
                on_fast={{let speed=state.travel_speed.clone();Callback::from(move |fast|speed.set(if fast {crate::app::flow::TravelSpeed::Fast} else {crate::app::flow::TravelSpeed::Normal}))}}
                on_camp={{let phase=state.phase.clone();Callback::from(move |()|phase.set(crate::app::Phase::Camp))}}
            />
        }
    })
}
