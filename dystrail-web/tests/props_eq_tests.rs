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
use std::rc::Rc;
use yew::Callback;

const fn weather_badge() -> WeatherBadge {
    WeatherBadge {
        weather: Weather::Clear,
        mitigated: false,
    }
}

#[rustfmt::skip]
fn camp_panel_props(state: Rc<GameState>, camp: Rc<CampConfig>, endgame: Rc<EndgameTravelCfg>) -> CampPanelProps { CampPanelProps { game_state: state, camp_config: camp, endgame_config: endgame, on_state_change: Callback::noop(), on_close: Callback::noop() } }

#[rustfmt::skip]
fn crossing_props(state: Rc<GameState>, config: Rc<CrossingConfig>, kind: CrossingKind) -> CrossingCardProps { CrossingCardProps { game_state: state, config, kind, on_choice: Callback::noop() } }

#[test]
#[rustfmt::skip]
fn component_props_equality_tracks_shared_pointers_and_variants() {
    let state = Rc::new(GameState::default());
    let camp_cfg = Rc::new(CampConfig::default());
    let endgame_cfg = Rc::new(EndgameTravelCfg::default_config());
    let crossing_cfg = Rc::new(CrossingConfig::default());
    let camp_a = camp_panel_props(state.clone(), camp_cfg.clone(), endgame_cfg.clone());
    let camp_b = camp_panel_props(state.clone(), camp_cfg, endgame_cfg);
    assert!(camp_a == camp_b);
    let base_crossing = crossing_props(state.clone(), crossing_cfg.clone(), CrossingKind::Checkpoint);
    let different_cfg = crossing_props(state.clone(), Rc::new(CrossingConfig::default()), CrossingKind::Checkpoint);
    let different_kind = crossing_props(state.clone(), crossing_cfg, CrossingKind::BridgeOut);
    assert!(base_crossing != different_cfg);
    assert!(base_crossing != different_kind);
    let otd_cross_a = OtDeluxeCrossingCardProps { game_state: state.clone(), on_choice: Callback::noop() };
    let otd_cross_b = OtDeluxeCrossingCardProps { game_state: state.clone(), on_choice: Callback::noop() };
    assert!(otd_cross_a == otd_cross_b);
    let store_a = OtDeluxeStorePanelProps { state: state.clone(), on_purchase: Callback::noop(), on_leave: Callback::noop() };
    let store_b = OtDeluxeStorePanelProps { state, on_purchase: Callback::noop(), on_leave: Callback::noop() };
    assert!(store_a == store_b);
}

#[test]
#[rustfmt::skip]
fn page_props_equality_for_core_routes() {
    let state = GameState::default();
    let state_rc = Rc::new(GameState::default());
    let shared_camp_cfg = Rc::new(CampConfig::default());
    let shared_endgame_cfg = Rc::new(EndgameTravelCfg::default_config());
    let shared_crossing_cfg = Rc::new(CrossingConfig::default());
    let boss_a = BossPageProps { state: state.clone(), config: BossConfig::load_from_static(), weather: weather_badge(), on_begin: Callback::noop() };
    let boss_b = BossPageProps { state, config: BossConfig::load_from_static(), weather: weather_badge(), on_begin: Callback::noop() };
    assert!(boss_a == boss_b);
    let camp_a = CampPageProps { state: state_rc.clone(), camp_config: shared_camp_cfg.clone(), endgame_config: shared_endgame_cfg.clone(), weather: weather_badge(), on_state_change: Callback::noop(), on_close: Callback::noop() };
    let camp_b = CampPageProps { state: state_rc.clone(), camp_config: shared_camp_cfg, endgame_config: shared_endgame_cfg, weather: weather_badge(), on_state_change: Callback::noop(), on_close: Callback::noop() };
    assert!(camp_a == camp_b);
    let crossing_a = CrossingPageProps { state: state_rc.clone(), config: shared_crossing_cfg.clone(), kind: CrossingKind::Checkpoint, weather: weather_badge(), on_choice: Callback::noop() };
    let crossing_b = CrossingPageProps { state: state_rc.clone(), config: shared_crossing_cfg, kind: CrossingKind::Checkpoint, weather: weather_badge(), on_choice: Callback::noop() };
    assert!(crossing_a == crossing_b);
    let encounter_a = EncounterPageProps { state: state_rc.clone(), weather: weather_badge(), on_choice: Callback::noop() };
    let encounter_b = EncounterPageProps { state: state_rc, weather: weather_badge(), on_choice: Callback::noop() };
    assert!(encounter_a == encounter_b);
}

#[test]
#[rustfmt::skip]
fn page_props_equality_for_otdeluxe_and_travel_routes() {
    let state_rc = Rc::new(GameState::default());
    let shared_pacing = Rc::new(PacingConfig::default());
    let otd_cross_a = OtDeluxeCrossingPageProps { state: state_rc.clone(), weather: weather_badge(), on_choice: Callback::noop() };
    let otd_cross_b = OtDeluxeCrossingPageProps { state: state_rc.clone(), weather: weather_badge(), on_choice: Callback::noop() };
    assert!(otd_cross_a == otd_cross_b);
    let otd_store_a = OtDeluxeStorePageProps { state: state_rc.clone(), weather: weather_badge(), on_purchase: Callback::<Vec<OtDeluxeStoreLineItem>>::noop(), on_leave: Callback::noop() };
    let otd_store_b = OtDeluxeStorePageProps { state: state_rc.clone(), weather: weather_badge(), on_purchase: Callback::<Vec<OtDeluxeStoreLineItem>>::noop(), on_leave: Callback::noop() };
    assert!(otd_store_a == otd_store_b);
    let route_a = RoutePromptPageProps { state: state_rc.clone(), prompt: OtDeluxeRoutePrompt::SubletteCutoff, weather: weather_badge(), on_choice: Callback::<OtDeluxeRouteDecision>::noop() };
    let route_b = RoutePromptPageProps { state: state_rc.clone(), prompt: OtDeluxeRoutePrompt::SubletteCutoff, weather: weather_badge(), on_choice: Callback::<OtDeluxeRouteDecision>::noop() };
    assert!(route_a == route_b);
    let travel_a = TravelPageProps { state: state_rc.clone(), logs: Vec::new(), pacing_config: shared_pacing.clone(), weather_badge: weather_badge(), data_ready: true, on_travel: Callback::noop(), on_trade: Callback::noop(), on_hunt: Callback::noop(), on_open_inventory: Callback::noop(), on_open_pace_diet: Callback::noop(), on_open_map: Callback::noop() };
    let travel_b = TravelPageProps { state: state_rc, logs: Vec::new(), pacing_config: shared_pacing, weather_badge: weather_badge(), data_ready: true, on_travel: Callback::noop(), on_trade: Callback::noop(), on_hunt: Callback::noop(), on_open_inventory: Callback::noop(), on_open_pace_diet: Callback::noop(), on_open_map: Callback::noop() };
    assert!(travel_a == travel_b);
}
