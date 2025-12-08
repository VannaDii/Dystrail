use crate::game::{GameState, store::Grants};
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct OutfittingPageProps {
    pub game_state: GameState,
    pub on_continue: Callback<(GameState, Grants, Vec<String>)>,
}

impl PartialEq for OutfittingPageProps {
    fn eq(&self, other: &Self) -> bool {
        self.game_state.budget_cents == other.game_state.budget_cents
            && self.game_state.persona_id == other.game_state.persona_id
    }
}

#[function_component(OutfittingPage)]
pub fn outfitting_page(props: &OutfittingPageProps) -> Html {
    html! {
        <section class="panel retro-menu">
            <crate::components::ui::outfitting_store::OutfittingStore
                game_state={props.game_state.clone()}
                on_continue={props.on_continue.clone()}
            />
        </section>
    }
}
