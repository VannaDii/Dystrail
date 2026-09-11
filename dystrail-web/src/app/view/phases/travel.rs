use crate::app::phase::build_weather_badge;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::pages::travel::TravelPage;
use std::rc::Rc;
use yew::prelude::*;

pub fn render_travel(state: &AppState, handlers: &AppHandlers) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let snapshot = sess.state().clone();
        let weather_badge = build_weather_badge(&snapshot, &state.weather_config);
        let moving=state.pending_turn.is_some() || *state.travel_running;
        let progress=state.pending_turn.as_ref().map_or_else(||crate::game::route::physical_miles(&snapshot),|p|crate::game::route::physical_miles(p.session.state()));
        let on_travel=if moving {
            let app=state.clone();
            Callback::from(move |()|app.travel_running.set(false))
        }else{crate::app::flow::resume(state)};
        let state_rc = Rc::new(snapshot);
        let pacing_config_rc = Rc::new((*state.pacing_config).clone());
        html! { <>
            <TravelPage
                state={state_rc}
                receipt={state.last_turn.as_ref().map_or_else(Html::default,crate::app::turn::render_last_turn)}
                logs={(*state.logs).clone()}
                pacing_config={pacing_config_rc}
                weather_badge={weather_badge}
                data_ready={state.data_ready()}
                on_travel={on_travel}
                {moving}
                {progress}
                duration={state.travel_speed.duration()}
                transit={state.pending_turn.as_ref().map_or_else(Html::default,|p|crate::app::turn::render_transit(state,p))}
                activities={crate::app::activities::render(state)}
                repair={crate::app::repair::render(state)}
                controls={crate::app::flow::controls(state)}
                on_pace_change={handlers.pace_change.clone()}
                on_diet_change={handlers.diet_change.clone()}
            />
        </> }
    })
}
