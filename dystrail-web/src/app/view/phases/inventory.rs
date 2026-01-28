use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::pages::inventory::InventoryPage;
use std::rc::Rc;
use yew::prelude::*;

pub fn render_inventory(state: &AppState) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let snapshot = Rc::new(sess.state().clone());
        let on_back = {
            let phase = state.phase.clone();
            Callback::from(move |()| phase.set(Phase::Travel))
        };
        html! { <InventoryPage state={snapshot} {on_back} /> }
    })
}
