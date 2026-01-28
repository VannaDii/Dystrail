use crate::game::{DietId, GameState, PaceId, PacingConfig};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct PaceDietPageProps {
    pub state: Rc<GameState>,
    pub pacing_config: Rc<PacingConfig>,
    pub on_pace_change: Callback<PaceId>,
    pub on_diet_change: Callback<DietId>,
    pub on_back: Callback<()>,
}

impl PartialEq for PaceDietPageProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
            && Rc::ptr_eq(&self.pacing_config, &other.pacing_config)
    }
}

#[function_component(PaceDietPage)]
pub fn pace_diet_page(props: &PaceDietPageProps) -> Html {
    html! {
        <section class="panel retro-menu" data-testid="pace-diet-screen">
            <crate::components::ui::pace_diet_panel::PaceDietPanel
                game_state={props.state.clone()}
                pacing_config={props.pacing_config.clone()}
                on_pace_change={props.on_pace_change.clone()}
                on_diet_change={props.on_diet_change.clone()}
                on_back={props.on_back.clone()}
            />
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pace_diet_props_equality_tracks_state_and_config() {
        let state = Rc::new(GameState::default());
        let pacing = Rc::new(PacingConfig::default_config());
        let props_a = PaceDietPageProps {
            state: state.clone(),
            pacing_config: pacing.clone(),
            on_pace_change: Callback::from(|_pace| ()),
            on_diet_change: Callback::from(|_diet| ()),
            on_back: Callback::from(|()| ()),
        };
        let props_b = PaceDietPageProps {
            state,
            pacing_config: pacing,
            on_pace_change: Callback::from(|_pace| ()),
            on_diet_change: Callback::from(|_diet| ()),
            on_back: Callback::from(|()| ()),
        };
        assert!(props_a == props_b);

        let props_c = PaceDietPageProps {
            state: Rc::new(GameState::default()),
            pacing_config: Rc::new(PacingConfig::default_config()),
            on_pace_change: Callback::from(|_pace| ()),
            on_diet_change: Callback::from(|_diet| ()),
            on_back: Callback::from(|()| ()),
        };
        assert!(props_a != props_c);
    }
}
