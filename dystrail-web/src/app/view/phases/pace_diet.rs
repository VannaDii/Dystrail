use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::pages::pace_diet::PaceDietPage;
use std::rc::Rc;
use yew::prelude::*;

fn render_pace_diet_page(
    snapshot: Rc<crate::game::GameState>,
    pacing: Rc<crate::game::pacing::PacingConfig>,
    handlers: &AppHandlers,
    on_back: Callback<()>,
) -> Html {
    html! {
        <PaceDietPage
            state={snapshot}
            pacing_config={pacing}
            on_pace_change={handlers.pace_change.clone()}
            on_diet_change={handlers.diet_change.clone()}
            on_back={on_back}
        />
    }
}

pub fn render_pace_diet(state: &AppState, handlers: &AppHandlers) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let snapshot = Rc::new(sess.state().clone());
        let pacing = Rc::new((*state.pacing_config).clone());
        let on_back = {
            let phase = state.phase.clone();
            Callback::from(move |()| phase.set(Phase::Travel))
        };
        render_pace_diet_page(snapshot, pacing, handlers, on_back)
    })
}
