use dystrail_web::components::ui::stats_bar::WeatherBadge;
use dystrail_web::game::{
    BossConfig, CampConfig, CrossingConfig, CrossingKind, EncounterData, EndgameTravelCfg,
    GameMode, GameState, MechanicalPolicyId, OtDeluxeRouteDecision, OtDeluxeRoutePrompt,
    PacingConfig, ResultConfig, Stats,
};
use dystrail_web::pages::{
    boot::{BootPage, BootPageProps},
    boss::{BossPage, BossPageProps},
    camp::{CampPage, CampPageProps},
    crossing::{CrossingPage, CrossingPageProps},
    encounter::{EncounterPage, EncounterPageProps},
    menu::{MenuPage, MenuPageProps},
    not_found::{NotFound, Props as NotFoundProps},
    otdeluxe_crossing::{OtDeluxeCrossingPage, OtDeluxeCrossingPageProps},
    otdeluxe_store::{OtDeluxeStorePage, OtDeluxeStorePageProps},
    outfitting::{OutfittingPage, OutfittingPageProps},
    persona::{PersonaPage, PersonaPageProps},
    result::{ResultPage, ResultPageProps},
    route_prompt::{RoutePromptPage, RoutePromptPageProps},
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
fn boot_page_renders_loading_and_ready() {
    dystrail_web::i18n::set_lang("en");
    let props_loading = BootPageProps {
        logo_src: "logo.png".into(),
        ready: false,
        preload_progress: 25,
        on_begin: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<BootPage>::with_props(props_loading).render());
    assert!(html.contains("Loading"));

    let props_ready = BootPageProps {
        logo_src: "logo.png".into(),
        ready: true,
        preload_progress: 100,
        on_begin: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<BootPage>::with_props(props_ready).render());
    assert!(html.contains("PRESS ANY KEY TO BEGIN"));
}

#[test]
fn menu_page_renders_seed_and_menu() {
    dystrail_web::i18n::set_lang("en");
    let props = MenuPageProps {
        code: "CL-ORANGE42".into(),
        logo_src: "logo.png".into(),
        on_action: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<MenuPage>::with_props(props).render());
    assert!(html.contains("D Y S T R A I L"));
    assert!(html.contains("CL-ORANGE42"));
}

#[test]
fn persona_page_renders_selector() {
    dystrail_web::i18n::set_lang("en");
    let props = PersonaPageProps {
        on_selected: Callback::noop(),
        on_continue: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<PersonaPage>::with_props(props).render());
    assert!(html.contains("persona-select"));
}

#[test]
fn outfitting_page_renders_store() {
    dystrail_web::i18n::set_lang("en");
    let props = OutfittingPageProps {
        game_state: base_state(),
        on_continue: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<OutfittingPage>::with_props(props).render());
    assert!(html.contains("outfitting-store"));
}

#[test]
fn travel_page_renders_panel_and_stats() {
    dystrail_web::i18n::set_lang("en");
    let props = TravelPageProps {
        state: Rc::new(base_state()),
        logs: vec!["log.booting".to_string()],
        pacing_config: Rc::new(PacingConfig::default()),
        weather_badge: weather_badge(),
        data_ready: true,
        on_travel: Callback::noop(),
        on_trade: Callback::noop(),
        on_hunt: Callback::noop(),
        on_pace_change: Callback::noop(),
        on_diet_change: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<TravelPage>::with_props(props).render());
    assert!(html.contains("stats-panel"));
    assert!(html.contains("travel-shell"));
}

#[test]
fn camp_page_renders_camp_panel() {
    dystrail_web::i18n::set_lang("en");
    let props = CampPageProps {
        state: Rc::new(base_state()),
        camp_config: Rc::new(CampConfig::default()),
        endgame_config: Rc::new(EndgameTravelCfg::default_config()),
        weather: weather_badge(),
        on_state_change: Callback::noop(),
        on_close: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<CampPage>::with_props(props).render());
    assert!(html.contains("camp-modal"));
}

#[test]
fn encounter_page_renders_loading_and_encounter() {
    dystrail_web::i18n::set_lang("en");
    let props = EncounterPageProps {
        state: Rc::new(base_state()),
        weather: weather_badge(),
        on_choice: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<EncounterPage>::with_props(props).render());
    assert!(html.contains("Loading encounters"));

    let mut state = base_state();
    let data = EncounterData::from_json(include_str!(
        "../../dystrail-web/static/assets/data/game.json"
    ))
    .expect("encounter data");
    state.current_encounter = data.encounters.first().cloned();

    let props = EncounterPageProps {
        state: Rc::new(state),
        weather: weather_badge(),
        on_choice: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<EncounterPage>::with_props(props).render());
    assert!(html.contains("encounter-panel"));
}

#[test]
fn boss_page_renders_stats() {
    dystrail_web::i18n::set_lang("en");
    let props = BossPageProps {
        state: base_state(),
        config: BossConfig::load_from_static(),
        weather: weather_badge(),
        on_begin: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<BossPage>::with_props(props).render());
    assert!(html.contains("boss-panel"));
}

#[test]
fn crossing_page_renders_options() {
    dystrail_web::i18n::set_lang("en");
    let props = CrossingPageProps {
        state: Rc::new(base_state()),
        config: Rc::new(CrossingConfig::default()),
        kind: CrossingKind::Checkpoint,
        weather: weather_badge(),
        on_choice: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<CrossingPage>::with_props(props).render());
    assert!(html.contains("ot-crossing"));
}

#[test]
fn otdeluxe_crossing_page_renders_panel() {
    dystrail_web::i18n::set_lang("en");
    let mut state = base_state();
    state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
    state.ot_deluxe.oxen.healthy = 4;
    let props = OtDeluxeCrossingPageProps {
        state: Rc::new(state),
        weather: weather_badge(),
        on_choice: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<OtDeluxeCrossingPage>::with_props(props).render());
    assert!(html.contains("ot-crossing"));
}

#[test]
fn otdeluxe_store_page_renders_panel() {
    dystrail_web::i18n::set_lang("en");
    let mut state = base_state();
    state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
    state.ot_deluxe.inventory.cash_cents = 1_000;
    state.ot_deluxe.inventory.food_lbs = 200;
    state.ot_deluxe.oxen.healthy = 4;
    let props = OtDeluxeStorePageProps {
        state: Rc::new(state),
        weather: weather_badge(),
        on_purchase: Callback::noop(),
        on_leave: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<OtDeluxeStorePage>::with_props(props).render());
    assert!(html.contains("otdeluxe-store-title"));
}

#[test]
fn route_prompt_page_renders_prompt() {
    dystrail_web::i18n::set_lang("en");
    let props = RoutePromptPageProps {
        state: Rc::new(base_state()),
        prompt: OtDeluxeRoutePrompt::SubletteCutoff,
        weather: weather_badge(),
        on_choice: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<RoutePromptPage>::with_props(props).render());
    assert!(html.contains("route-prompt-title"));
}

#[test]
fn result_page_renders_summary() {
    dystrail_web::i18n::set_lang("en");
    let props = ResultPageProps {
        state: base_state(),
        result_config: ResultConfig::default(),
        boss_won: false,
        on_replay_seed: Callback::noop(),
        on_new_run: Callback::noop(),
        on_title: Callback::noop(),
        on_export: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<ResultPage>::with_props(props).render());
    assert!(html.contains("result-screen"));
}

#[test]
fn not_found_page_renders_message() {
    dystrail_web::i18n::set_lang("en");
    let props = NotFoundProps {
        on_go_home: Callback::noop(),
    };
    let html = block_on(LocalServerRenderer::<NotFound>::with_props(props).render());
    assert!(html.contains("not-found"));
}

#[test]
fn route_prompt_decision_callback_smoke() {
    let cb = Callback::noop();
    cb.emit(OtDeluxeRouteDecision::StayOnTrail);
}
