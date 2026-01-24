use dystrail_web::app::phase::Phase;
use dystrail_web::app::state::AppState;
use dystrail_web::app::view::{AppHandlers, render_crossing};
use dystrail_web::game::boss::BossConfig;
use dystrail_web::game::data::EncounterData;
use dystrail_web::game::endgame::EndgameTravelCfg;
use dystrail_web::game::pacing::PacingConfig;
use dystrail_web::game::state::GameState;
use dystrail_web::game::weather::WeatherConfig;
use dystrail_web::game::{
    CampConfig, CrossingConfig, GameMode, JourneySession, MechanicalPolicyId, ResultConfig,
    StrategyId,
};
use futures::executor::block_on;
use yew::LocalServerRenderer;
use yew::prelude::*;

fn noop_handlers() -> AppHandlers {
    AppHandlers {
        travel: Callback::noop(),
        trade: Callback::noop(),
        hunt: Callback::noop(),
        store_purchase: Callback::noop(),
        store_leave: Callback::noop(),
        pace_change: Callback::noop(),
        diet_change: Callback::noop(),
        encounter_choice: Callback::noop(),
        crossing_choice: Callback::noop(),
        otdeluxe_crossing_choice: Callback::noop(),
        route_prompt_choice: Callback::noop(),
        boss: Callback::noop(),
        save: Callback::noop(),
        load: Callback::noop(),
        export_state: Callback::noop(),
        import_state: Callback::noop(),
        lang_change: Callback::noop(),
        toggle_hc: Callback::noop(),
        settings_hc_change: Callback::noop(),
        go_home: Callback::noop(),
        begin_boot: Callback::noop(),
    }
}

fn base_state() -> GameState {
    GameState::default().with_seed(42, GameMode::Classic, EncounterData::empty())
}

#[function_component(CrossingHarness)]
fn crossing_harness() -> Html {
    dystrail_web::i18n::set_lang("en");
    let data = use_state(EncounterData::empty);
    let session = {
        let mut state = base_state();
        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        state.ot_deluxe.crossing.choice_pending = true;
        let session = JourneySession::from_state(
            state,
            StrategyId::Balanced,
            &EndgameTravelCfg::default_config(),
        );
        use_state(move || Some(session))
    };
    let app_state = AppState {
        phase: use_state(|| Phase::Crossing),
        code: use_state(|| AttrValue::from("CL-ORANGE42")),
        data,
        pacing_config: use_state(PacingConfig::default_config),
        endgame_config: use_state(EndgameTravelCfg::default_config),
        weather_config: use_state(WeatherConfig::default_config),
        camp_config: use_state(CampConfig::default_config),
        crossing_config: use_state(CrossingConfig::default),
        boss_config: use_state(BossConfig::load_from_static),
        result_config: use_state(ResultConfig::default),
        preload_progress: use_state(|| 0_u8),
        boot_ready: use_state(|| true),
        high_contrast: use_state(|| false),
        pending_state: use_state(|| None::<GameState>),
        session,
        logs: use_state(Vec::<String>::new),
        run_seed: use_state(|| 42_u64),
        show_save: use_state(|| false),
        save_focus_target: use_state(|| AttrValue::from("save-open-btn")),
        show_settings: use_state(|| false),
        current_language: use_state(|| String::from("en")),
    };
    render_crossing(&app_state, &noop_handlers())
}

#[test]
fn render_crossing_otdeluxe_choice_pending_renders_panel() {
    let html = block_on(LocalServerRenderer::<CrossingHarness>::new().render());
    assert!(html.contains("ot-crossing"));
}
