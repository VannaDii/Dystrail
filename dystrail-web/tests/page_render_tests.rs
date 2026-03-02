use dystrail_web::components::ui::stats_bar::WeatherBadge;
use dystrail_web::game::{
    BossConfig, CampConfig, CrossingConfig, CrossingKind, EncounterData, EndgameTravelCfg,
    GameMode, GameState, MechanicalPolicyId, OtDeluxeRoutePrompt, PacingConfig, ResultConfig,
    Stats,
};
use dystrail_web::pages::{
    about::{AboutPage, AboutPageProps},
    boot::{BootPage, BootPageProps},
    boss::{BossPage, BossPageProps},
    camp::{CampPage, CampPageProps},
    crossing::{CrossingPage, CrossingPageProps},
    encounter::{EncounterPage, EncounterPageProps},
    inventory::{InventoryPage, InventoryPageProps},
    menu::{MenuPage, MenuPageProps},
    not_found::{NotFound, Props as NotFoundProps},
    otdeluxe_crossing::{OtDeluxeCrossingPage, OtDeluxeCrossingPageProps},
    otdeluxe_store::{OtDeluxeStorePage, OtDeluxeStorePageProps},
    result::{ResultPage, ResultPageProps},
    route_prompt::{RoutePromptPage, RoutePromptPageProps},
    settings::{SettingsPage, SettingsPageProps},
    travel::{TravelPage, TravelPageProps},
};
use futures::executor::block_on;
use std::rc::Rc;
use yew::{Callback, LocalServerRenderer};

const fn weather_badge() -> WeatherBadge {
    WeatherBadge {
        weather: dystrail_web::game::Weather::Clear,
        mitigated: false,
    }
}
fn base_state() -> GameState {
    GameState {
        mode: GameMode::Classic,
        persona_id: Some("organizer".to_string()),
        stats: Stats {
            supplies: 12,
            sanity: 9,
            pants: 20,
            ..Stats::default()
        },
        day: 3,
        region: dystrail_web::game::Region::Heartland,
        ..GameState::default()
    }
}

#[test]
#[rustfmt::skip]
fn boot_menu_and_settings_render_expected_ui() {
    dystrail_web::i18n::set_lang("en");
    let boot = block_on(LocalServerRenderer::<BootPage>::with_props(BootPageProps { logo_src: "logo.png".into(), ready: false, preload_progress: 25, on_begin: Callback::noop() }).render());
    let menu = block_on(LocalServerRenderer::<MenuPage>::with_props(MenuPageProps { logo_src: "logo.png".into(), on_action: Callback::noop() }).render());
    let settings = block_on(LocalServerRenderer::<SettingsPage>::with_props(SettingsPageProps { current_lang: "en".to_string(), high_contrast: false, on_lang_change: Callback::noop(), on_toggle_hc: Callback::noop(), on_back: Callback::noop() }).render());
    assert!(boot.contains(&dystrail_web::i18n::t("boot.loading_label")));
    assert!(menu.contains(&dystrail_web::i18n::t("menu.start_journey")));
    assert!(settings.contains("settings-language"));
}

#[test]
#[rustfmt::skip]
fn about_not_found_and_result_render_expected_ui() {
    dystrail_web::i18n::set_lang("en");
    let about = block_on(LocalServerRenderer::<AboutPage>::with_props(AboutPageProps { on_back: Callback::noop() }).render());
    let not_found = block_on(LocalServerRenderer::<NotFound>::with_props(NotFoundProps { on_go_home: Callback::noop() }).render());
    let result = block_on(LocalServerRenderer::<ResultPage>::with_props(ResultPageProps { state: base_state(), result_config: ResultConfig::default(), boss_won: false, on_replay_seed: Callback::noop(), on_new_run: Callback::noop(), on_title: Callback::noop(), on_export: Callback::noop() }).render());
    assert!(about.contains(&dystrail_web::i18n::t("about.title")));
    assert!(not_found.contains("not-found"));
    assert!(result.contains("result-screen"));
}

#[test]
#[rustfmt::skip]
fn travel_camp_encounter_and_boss_render_expected_ui() {
    dystrail_web::i18n::set_lang("en");
    let travel = block_on(LocalServerRenderer::<TravelPage>::with_props(TravelPageProps { state: Rc::new(base_state()), logs: vec!["log.booting".to_string()], pacing_config: Rc::new(PacingConfig::default()), weather_badge: weather_badge(), data_ready: true, on_travel: Callback::noop(), on_trade: Callback::noop(), on_hunt: Callback::noop(), on_open_inventory: Callback::noop(), on_open_pace_diet: Callback::noop(), on_open_map: Callback::noop() }).render());
    let camp = block_on(LocalServerRenderer::<CampPage>::with_props(CampPageProps { state: Rc::new(base_state()), camp_config: Rc::new(CampConfig::default()), endgame_config: Rc::new(EndgameTravelCfg::default_config()), weather: weather_badge(), on_state_change: Callback::noop(), on_close: Callback::noop() }).render());
    let encounter = block_on(LocalServerRenderer::<EncounterPage>::with_props(EncounterPageProps { state: Rc::new(base_state()), weather: weather_badge(), on_choice: Callback::noop() }).render());
    let boss = block_on(LocalServerRenderer::<BossPage>::with_props(BossPageProps { state: base_state(), config: BossConfig::load_from_static(), weather: weather_badge(), on_begin: Callback::noop() }).render());
    assert!(travel.contains("travel-shell"));
    assert!(camp.contains("camp-modal"));
    assert!(encounter.contains("Loading encounters"));
    assert!(boss.contains("boss-panel"));
}

#[test]
#[rustfmt::skip]
fn crossing_route_prompt_and_otdeluxe_render_expected_ui() {
    dystrail_web::i18n::set_lang("en");
    let crossing = block_on(LocalServerRenderer::<CrossingPage>::with_props(CrossingPageProps { state: Rc::new(base_state()), config: Rc::new(CrossingConfig::default()), kind: CrossingKind::Checkpoint, weather: weather_badge(), on_choice: Callback::noop() }).render());
    let route = block_on(LocalServerRenderer::<RoutePromptPage>::with_props(RoutePromptPageProps { state: Rc::new(base_state()), prompt: OtDeluxeRoutePrompt::SubletteCutoff, weather: weather_badge(), on_choice: Callback::noop() }).render());
    let mut cross_state = base_state(); cross_state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s; cross_state.ot_deluxe.oxen.healthy = 4;
    let otd_cross = block_on(LocalServerRenderer::<OtDeluxeCrossingPage>::with_props(OtDeluxeCrossingPageProps { state: Rc::new(cross_state), weather: weather_badge(), on_choice: Callback::noop() }).render());
    let mut store_state = base_state(); store_state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s; store_state.ot_deluxe.inventory.cash_cents = 1_000; store_state.ot_deluxe.inventory.food_lbs = 200; store_state.ot_deluxe.oxen.healthy = 4;
    let otd_store = block_on(LocalServerRenderer::<OtDeluxeStorePage>::with_props(OtDeluxeStorePageProps { state: Rc::new(store_state), weather: weather_badge(), on_purchase: Callback::noop(), on_leave: Callback::noop() }).render());
    assert!(crossing.contains("ot-crossing"));
    assert!(route.contains("route-prompt-title"));
    assert!(otd_cross.contains("ot-crossing"));
    assert!(otd_store.contains("otdeluxe-store-title"));
}

#[test]
#[rustfmt::skip]
fn encounter_with_loaded_data_renders_encounter_panel() {
    dystrail_web::i18n::set_lang("en");
    let data = EncounterData::from_json(include_str!("../../dystrail-web/static/assets/data/game.json")).expect("encounter data");
    let mut state = base_state();
    state.current_encounter = data.encounters.first().cloned();
    let html = block_on(LocalServerRenderer::<EncounterPage>::with_props(EncounterPageProps { state: Rc::new(state), weather: weather_badge(), on_choice: Callback::noop() }).render());
    assert!(html.contains("encounter-panel"));
}

#[test]
#[rustfmt::skip]
fn inventory_page_renders_empty_and_present_tags() {
    dystrail_web::i18n::set_lang("en");

    let empty = block_on(LocalServerRenderer::<InventoryPage>::with_props(InventoryPageProps {
        state: Rc::new(base_state()),
        on_back: Callback::noop(),
    }).render());
    assert!(empty.contains(&dystrail_web::i18n::t("inventory.tags_none")));

    let mut tagged_state = base_state();
    let _ = tagged_state.inventory.tags.insert("permit".to_string());
    let tagged = block_on(LocalServerRenderer::<InventoryPage>::with_props(InventoryPageProps {
        state: Rc::new(tagged_state),
        on_back: Callback::noop(),
    }).render());
    assert!(tagged.contains("permit"));
}
