use crate::game::{GameState, ResultConfig};
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct ResultPageProps {
    pub state: GameState,
    pub result_config: ResultConfig,
    pub boss_won: bool,
    pub on_replay_seed: Callback<()>,
    pub on_new_run: Callback<()>,
    pub on_title: Callback<()>,
    pub on_export: Callback<()>,
}

impl PartialEq for ResultPageProps {
    fn eq(&self, other: &Self) -> bool {
        self.state.day == other.state.day
            && self.state.region == other.state.region
            && self.boss_won == other.boss_won
    }
}

#[function_component(ResultPage)]
pub fn result_page(props: &ResultPageProps) -> Html {
    html! {
        <crate::components::ui::result_screen::ResultScreen
            game_state={props.state.clone()}
            result_config={props.result_config.clone()}
            boss_won={props.boss_won}
            on_replay_seed={props.on_replay_seed.clone()}
            on_new_run={props.on_new_run.clone()}
            on_title={props.on_title.clone()}
            on_export={props.on_export.clone()}
        />
    }
}
