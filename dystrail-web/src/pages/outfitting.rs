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

#[cfg(test)]
mod tests {
    use super::OutfittingPageProps;
    use crate::game::GameState;
    use yew::Callback;

    #[test]
    fn props_eq_compares_budget_and_persona() {
        let state = GameState {
            budget_cents: 1200,
            persona_id: Some("organizer".to_string()),
            ..GameState::default()
        };
        let props_a = OutfittingPageProps {
            game_state: state.clone(),
            on_continue: Callback::noop(),
        };

        let other = GameState {
            day: state.day + 1,
            ..state.clone()
        };
        let props_b = OutfittingPageProps {
            game_state: other,
            on_continue: Callback::noop(),
        };
        assert!(props_a == props_b);

        let changed = GameState {
            budget_cents: 999,
            ..state
        };
        let props_c = OutfittingPageProps {
            game_state: changed,
            on_continue: Callback::noop(),
        };
        assert!(props_a != props_c);
    }
}
