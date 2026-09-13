use crate::game::{DietId, GameState, PaceId, PacingConfig};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct Props {
    pub detail: u8,
    pub logs: Vec<String>,
    #[prop_or_default]
    pub receipt: Html,
    pub game_state: Option<Rc<GameState>>,
    pub pacing_config: Rc<PacingConfig>,
    pub on_pace_change: Callback<PaceId>,
    pub on_diet_change: Callback<DietId>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        self.detail == other.detail
            && self.logs == other.logs
            && match (&self.game_state, &other.game_state) {
                (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}

/// Action controls keep the available choices and their effects visible together.
#[function_component(TravelPanel)]
pub fn travel_panel(p: &Props) -> Html {
    crate::i18n::use_language();
    let Some(gs) = p.game_state.as_ref() else {
        return Html::default();
    };
    html! {<section class="travel-controls" role="tabpanel" id="travel-detail-panel" aria-labelledby={format!("travel-tab-{}",p.detail)} tabindex="0">
        if p.detail==2 {{super::pace::render_settings(gs,&p.pacing_config,&p.on_pace_change,&p.on_diet_change)}}
        if p.detail==0 {{super::status::render_status(gs)}} else if p.detail==1 {{super::journal::render(gs,&p.logs)}}
        if p.detail==3 {{p.receipt.clone()}}
    </section>}
}
