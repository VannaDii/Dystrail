use std::rc::Rc;

use dystrail_web::components::ui::camp_panel::Props as CampPanelProps;
use dystrail_web::components::ui::crossing_card::CrossingCardProps;
use dystrail_web::components::ui::otdeluxe_crossing_card::OtDeluxeCrossingCardProps;
use dystrail_web::components::ui::otdeluxe_store_panel::OtDeluxeStorePanelProps;
use dystrail_web::components::ui::stats_bar::WeatherBadge;
use dystrail_web::game::{
    BossConfig, CampConfig, CrossingConfig, CrossingKind, EndgameTravelCfg, GameState,
    OtDeluxeRouteDecision, OtDeluxeRoutePrompt, OtDeluxeStoreLineItem, PacingConfig, Weather,
};
use dystrail_web::pages::{
    boss::BossPageProps, camp::CampPageProps, crossing::CrossingPageProps,
    encounter::EncounterPageProps, otdeluxe_crossing::OtDeluxeCrossingPageProps,
    otdeluxe_store::OtDeluxeStorePageProps, route_prompt::RoutePromptPageProps,
    travel::TravelPageProps,
};
use yew::Callback;

const fn weather_badge() -> WeatherBadge {
    WeatherBadge {
        weather: Weather::Clear,
        mitigated: false,
    }
}

#[test]
fn ui_props_use_pointer_equality() {
    let state = Rc::new(GameState::default());
    let other_state = Rc::new(GameState::default());
    let camp_cfg = Rc::new(CampConfig::default());
    let endgame_cfg = Rc::new(EndgameTravelCfg::default_config());
    let crossing_cfg = Rc::new(CrossingConfig::default());

    let camp_panel_a = CampPanelProps {
        game_state: state.clone(),
        camp_config: camp_cfg.clone(),
        endgame_config: endgame_cfg.clone(),
        on_state_change: Callback::noop(),
        on_close: Callback::noop(),
    };
    let camp_panel_b = CampPanelProps {
        game_state: state.clone(),
        camp_config: camp_cfg.clone(),
        endgame_config: endgame_cfg.clone(),
        on_state_change: Callback::noop(),
        on_close: Callback::noop(),
    };
    assert!(camp_panel_a == camp_panel_b);

    let camp_panel_c = CampPanelProps {
        game_state: other_state.clone(),
        camp_config: camp_cfg,
        endgame_config: endgame_cfg,
        on_state_change: Callback::noop(),
        on_close: Callback::noop(),
    };
    assert!(camp_panel_a != camp_panel_c);

    let crossing_props_a = CrossingCardProps {
        game_state: state.clone(),
        config: crossing_cfg.clone(),
        kind: CrossingKind::Checkpoint,
        on_choice: Callback::noop(),
    };
    let crossing_props_b = CrossingCardProps {
        game_state: state.clone(),
        config: crossing_cfg.clone(),
        kind: CrossingKind::Checkpoint,
        on_choice: Callback::noop(),
    };
    assert!(crossing_props_a == crossing_props_b);

    let crossing_props_c = CrossingCardProps {
        game_state: other_state,
        config: crossing_cfg,
        kind: CrossingKind::Checkpoint,
        on_choice: Callback::noop(),
    };
    assert!(crossing_props_a != crossing_props_c);

    let otdeluxe_crossing_props_a = OtDeluxeCrossingCardProps {
        game_state: state.clone(),
        on_choice: Callback::noop(),
    };
    let otdeluxe_crossing_props_b = OtDeluxeCrossingCardProps {
        game_state: state.clone(),
        on_choice: Callback::noop(),
    };
    assert!(otdeluxe_crossing_props_a == otdeluxe_crossing_props_b);

    let otdeluxe_store_props_a = OtDeluxeStorePanelProps {
        state: state.clone(),
        on_purchase: Callback::noop(),
        on_leave: Callback::noop(),
    };
    let otdeluxe_store_props_b = OtDeluxeStorePanelProps {
        state,
        on_purchase: Callback::noop(),
        on_leave: Callback::noop(),
    };
    assert!(otdeluxe_store_props_a == otdeluxe_store_props_b);
}

#[test]
fn page_props_compare_boss_and_camp() {
    let weather = weather_badge();
    let state = GameState::default();
    let state_rc = Rc::new(GameState::default());
    let config = BossConfig::load_from_static();
    let camp_cfg = Rc::new(CampConfig::default());
    let endgame_cfg = Rc::new(EndgameTravelCfg::default_config());

    let boss_a = BossPageProps {
        state: state.clone(),
        config: config.clone(),
        weather: weather.clone(),
        on_begin: Callback::noop(),
    };
    let boss_b = BossPageProps {
        state,
        config,
        weather: weather.clone(),
        on_begin: Callback::noop(),
    };
    assert!(boss_a == boss_b);

    let camp_a = CampPageProps {
        state: state_rc.clone(),
        camp_config: camp_cfg.clone(),
        endgame_config: endgame_cfg.clone(),
        weather: weather.clone(),
        on_state_change: Callback::noop(),
        on_close: Callback::noop(),
    };
    let camp_b = CampPageProps {
        state: state_rc,
        camp_config: camp_cfg,
        endgame_config: endgame_cfg,
        weather,
        on_state_change: Callback::noop(),
        on_close: Callback::noop(),
    };
    assert!(camp_a == camp_b);
}

#[test]
fn page_props_compare_crossing_and_encounter() {
    let weather = weather_badge();
    let state_rc = Rc::new(GameState::default());
    let crossing_cfg = Rc::new(CrossingConfig::default());

    let crossing_a = CrossingPageProps {
        state: state_rc.clone(),
        config: crossing_cfg.clone(),
        kind: CrossingKind::Checkpoint,
        weather: weather.clone(),
        on_choice: Callback::noop(),
    };
    let crossing_b = CrossingPageProps {
        state: state_rc.clone(),
        config: crossing_cfg,
        kind: CrossingKind::Checkpoint,
        weather: weather.clone(),
        on_choice: Callback::noop(),
    };
    assert!(crossing_a == crossing_b);

    let encounter_a = EncounterPageProps {
        state: state_rc.clone(),
        weather: weather.clone(),
        on_choice: Callback::noop(),
    };
    let encounter_b = EncounterPageProps {
        state: state_rc,
        weather,
        on_choice: Callback::noop(),
    };
    assert!(encounter_a == encounter_b);
}

#[test]
fn page_props_compare_otdeluxe_and_travel() {
    let weather = weather_badge();
    let state_rc = Rc::new(GameState::default());

    let otdeluxe_cross_a = OtDeluxeCrossingPageProps {
        state: state_rc.clone(),
        weather: weather.clone(),
        on_choice: Callback::noop(),
    };
    let otdeluxe_cross_b = OtDeluxeCrossingPageProps {
        state: state_rc.clone(),
        weather: weather.clone(),
        on_choice: Callback::noop(),
    };
    assert!(otdeluxe_cross_a == otdeluxe_cross_b);

    let otdeluxe_store_a = OtDeluxeStorePageProps {
        state: state_rc.clone(),
        weather: weather.clone(),
        on_purchase: Callback::<Vec<OtDeluxeStoreLineItem>>::noop(),
        on_leave: Callback::noop(),
    };
    let otdeluxe_store_b = OtDeluxeStorePageProps {
        state: state_rc.clone(),
        weather: weather.clone(),
        on_purchase: Callback::<Vec<OtDeluxeStoreLineItem>>::noop(),
        on_leave: Callback::noop(),
    };
    assert!(otdeluxe_store_a == otdeluxe_store_b);

    let prompt = OtDeluxeRoutePrompt::SubletteCutoff;
    let route_a = RoutePromptPageProps {
        state: state_rc.clone(),
        prompt,
        weather: weather.clone(),
        on_choice: Callback::<OtDeluxeRouteDecision>::noop(),
    };
    let route_b = RoutePromptPageProps {
        state: state_rc.clone(),
        prompt,
        weather: weather.clone(),
        on_choice: Callback::<OtDeluxeRouteDecision>::noop(),
    };
    assert!(route_a == route_b);

    let pacing = Rc::new(PacingConfig::default());
    let travel_a = TravelPageProps {
        state: state_rc.clone(),
        logs: Vec::new(),
        pacing_config: pacing.clone(),
        weather_badge: weather.clone(),
        data_ready: true,
        on_travel: Callback::noop(),
        on_trade: Callback::noop(),
        on_hunt: Callback::noop(),
        on_pace_change: Callback::noop(),
        on_diet_change: Callback::noop(),
    };
    let travel_b = TravelPageProps {
        state: state_rc,
        logs: Vec::new(),
        pacing_config: pacing,
        weather_badge: weather,
        data_ready: true,
        on_travel: Callback::noop(),
        on_trade: Callback::noop(),
        on_hunt: Callback::noop(),
        on_pace_change: Callback::noop(),
        on_diet_change: Callback::noop(),
    };
    assert!(travel_a == travel_b);
}
