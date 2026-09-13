use crate::game::{GameState, ResultConfig};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct ResultPageProps {
    pub state: Rc<GameState>,
    pub result_config: ResultConfig,
    pub boss_won: bool,
    pub on_replay_seed: Callback<()>,
    pub on_new_run: Callback<()>,
    pub on_title: Callback<()>,
    pub on_export: Callback<()>,
}

impl PartialEq for ResultPageProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
            && self.boss_won == other.boss_won
            && self.result_config == other.result_config
            && self.on_replay_seed == other.on_replay_seed
            && self.on_new_run == other.on_new_run
            && self.on_title == other.on_title
            && self.on_export == other.on_export
    }
}

#[function_component(ResultPage)]
pub fn result_page(props: &ResultPageProps) -> Html {
    crate::i18n::use_language();
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
