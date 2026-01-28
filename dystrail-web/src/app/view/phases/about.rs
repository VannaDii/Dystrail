use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::pages::about::AboutPage;
use yew::prelude::*;

pub fn render_about(state: &AppState) -> Html {
    let on_back = {
        let phase = state.phase.clone();
        Callback::from(move |()| phase.set(Phase::Menu))
    };
    html! { <AboutPage {on_back} /> }
}
