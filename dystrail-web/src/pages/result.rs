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

#[cfg(test)]
mod tests {
    use super::ResultPageProps;
    use crate::game::{GameState, Region, ResultConfig};
    use yew::Callback;

    #[test]
    fn props_eq_compares_day_region_and_boss() {
        let state = GameState {
            day: 10,
            region: Region::RustBelt,
            ..GameState::default()
        };
        let props_a = ResultPageProps {
            state: state.clone(),
            result_config: ResultConfig::default(),
            boss_won: false,
            on_replay_seed: Callback::noop(),
            on_new_run: Callback::noop(),
            on_title: Callback::noop(),
            on_export: Callback::noop(),
        };
        let props_b = ResultPageProps {
            state,
            result_config: ResultConfig::default(),
            boss_won: false,
            on_replay_seed: Callback::noop(),
            on_new_run: Callback::noop(),
            on_title: Callback::noop(),
            on_export: Callback::noop(),
        };
        assert!(props_a == props_b);

        let changed = GameState {
            day: 11,
            region: Region::RustBelt,
            ..GameState::default()
        };
        let props_c = ResultPageProps {
            state: changed,
            result_config: ResultConfig::default(),
            boss_won: false,
            on_replay_seed: Callback::noop(),
            on_new_run: Callback::noop(),
            on_title: Callback::noop(),
            on_export: Callback::noop(),
        };
        assert!(props_a != props_c);
    }
}
