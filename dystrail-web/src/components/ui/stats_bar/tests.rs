use super::*;
use futures::executor::block_on;
use yew::LocalServerRenderer;
use yew::prelude::*;

#[test]
fn stats_bar_renders_core_fields() {
    crate::i18n::set_lang("en");
    let stats = Stats {
        hp: 7,
        sanity: 5,
        credibility: 3,
        supplies: 12,
        morale: 6,
        allies: 2,
        pants: 42,
    };
    let props = Props {
        stats,
        day: 9,
        region: Region::RustBelt,
        exec_order: None,
        persona_id: None,
        weather: Some(WeatherBadge {
            weather: Weather::Clear,
            mitigated: false,
        }),
    };

    let html = block_on(LocalServerRenderer::<StatsBar>::with_props(props).render());
    assert!(
        html.contains("Rust Belt"),
        "region label should appear: {html}"
    );
    assert!(
        html.contains("42%"),
        "pants percentage should render: {html}"
    );
    assert!(
        html.contains("HP"),
        "stat abbreviations should be present: {html}"
    );
    assert!(
        html.contains("sprite-weather-clear"),
        "weather sprite class should render: {html}"
    );
}

#[test]
fn stats_bar_announces_exec_order() {
    crate::i18n::set_lang("en");
    let props = Props {
        stats: Stats::default(),
        day: 1,
        region: Region::Heartland,
        exec_order: Some(ExecOrder::TariffTsunami),
        persona_id: None,
        weather: None,
    };

    let html = block_on(LocalServerRenderer::<StatsBar>::with_props(props).render());
    assert!(
        html.contains("Tariff Tsunami"),
        "exec order should render announcement block: {html}"
    );
    assert!(
        html.contains("sprite-eo-tariff"),
        "exec order sprite class should render: {html}"
    );
}

#[test]
fn stats_bar_pants_classes_cover_warning_and_critical() {
    crate::i18n::set_lang("en");
    let critical_props = Props {
        stats: Stats {
            pants: 95,
            ..Stats::default()
        },
        day: 1,
        region: Region::Heartland,
        exec_order: None,
        persona_id: None,
        weather: None,
    };
    let html = block_on(LocalServerRenderer::<StatsBar>::with_props(critical_props).render());
    assert!(html.contains("pants-critical"));

    let warn_props = Props {
        stats: Stats {
            pants: 75,
            ..Stats::default()
        },
        day: 2,
        region: Region::Heartland,
        exec_order: None,
        persona_id: None,
        weather: None,
    };
    let html = block_on(LocalServerRenderer::<StatsBar>::with_props(warn_props).render());
    assert!(html.contains("pants-warn"));
}

#[test]
fn helper_functions_cover_persona_and_icons() {
    crate::i18n::set_lang("en");
    let unknown = helpers::persona_name_for("mystery");
    assert_eq!(unknown, "MYSTERY");
    assert_eq!(helpers::persona_initial(""), "?");
    assert_eq!(helpers::persona_initial("Crew"), "C");
    assert!(!helpers::weather_symbol(Weather::Smoke).is_empty());
    assert_ne!(
        helpers::weather_symbol(Weather::Smoke),
        helpers::weather_symbol(Weather::Clear)
    );
}

#[test]
fn stat_chip_skips_unknown_icon_kind() {
    #[function_component(StatChipHarness)]
    fn stat_chip_harness() -> Html {
        crate::i18n::set_lang("en");
        helpers::stat_chip(String::from("Label"), 3, "unknown")
    }

    let html = block_on(LocalServerRenderer::<StatChipHarness>::new().render());
    assert!(html.contains("stat-chip"));
    assert!(!html.contains("stat-icon"));
}

#[test]
fn helper_tokens_and_labels_cover_exec_and_regions() {
    crate::i18n::set_lang("en");
    assert_eq!(helpers::exec_order_token(ExecOrder::Shutdown), "SD");
    assert_eq!(helpers::exec_order_token(ExecOrder::DoEEliminated), "DE");
    assert_eq!(
        helpers::exec_sprite_class(ExecOrder::DoEEliminated),
        "sprite-eo-doe"
    );
    assert_eq!(
        helpers::weather_sprite_class(Weather::HeatWave),
        "sprite-weather-heat"
    );
    assert!(helpers::region_label(Region::Beltway).contains("Beltway"));
}

#[test]
fn helper_variants_cover_tokens_and_sprites() {
    crate::i18n::set_lang("en");
    assert_eq!(helpers::persona_name_for("journalist"), "Journalist");
    assert_eq!(helpers::exec_order_token(ExecOrder::TravelBanLite), "TB");
    assert_eq!(helpers::exec_order_token(ExecOrder::BookPanic), "BP");
    assert_eq!(helpers::exec_order_token(ExecOrder::TariffTsunami), "TT");
    assert_eq!(helpers::exec_order_token(ExecOrder::WarDeptReorg), "WR");
    assert_eq!(
        helpers::exec_sprite_class(ExecOrder::Shutdown),
        "sprite-eo-shutdown"
    );
    assert_eq!(
        helpers::exec_sprite_class(ExecOrder::TravelBanLite),
        "sprite-eo-travelban"
    );
    assert_eq!(
        helpers::exec_sprite_class(ExecOrder::BookPanic),
        "sprite-eo-book"
    );
    assert_eq!(
        helpers::exec_sprite_class(ExecOrder::TariffTsunami),
        "sprite-eo-tariff"
    );
    assert_eq!(
        helpers::exec_sprite_class(ExecOrder::WarDeptReorg),
        "sprite-eo-war"
    );
    assert_eq!(
        helpers::weather_sprite_class(Weather::Clear),
        "sprite-weather-clear"
    );
    assert_eq!(
        helpers::weather_sprite_class(Weather::Storm),
        "sprite-weather-storm"
    );
    assert_eq!(
        helpers::weather_sprite_class(Weather::ColdSnap),
        "sprite-weather-cold"
    );
    assert_eq!(
        helpers::weather_sprite_class(Weather::Smoke),
        "sprite-weather-smoke"
    );
    let clear = helpers::weather_symbol(Weather::Clear);
    let storm = helpers::weather_symbol(Weather::Storm);
    let heat = helpers::weather_symbol(Weather::HeatWave);
    let cold = helpers::weather_symbol(Weather::ColdSnap);
    let smoke = helpers::weather_symbol(Weather::Smoke);
    assert!(!clear.is_empty());
    assert!(!storm.is_empty());
    assert!(!heat.is_empty());
    assert!(!cold.is_empty());
    assert!(!smoke.is_empty());
    assert!(clear != storm);
    assert!(heat != cold);
    assert!(helpers::region_label(Region::Heartland).contains("Heartland"));
    assert!(helpers::region_label(Region::RustBelt).contains("Rust"));
}

#[test]
fn stat_icon_branches_render_all_variants() {
    #[function_component(StatIconHarness)]
    fn stat_icon_harness() -> Html {
        crate::i18n::set_lang("en");
        html! {
            <div>
                { helpers::stat_chip(String::from("Sup"), 1, "supplies") }
                { helpers::stat_chip(String::from("HP"), 2, "hp") }
                { helpers::stat_chip(String::from("San"), 3, "sanity") }
                { helpers::stat_chip(String::from("Cred"), 4, "cred") }
                { helpers::stat_chip(String::from("Mor"), 5, "morale") }
                { helpers::stat_chip(String::from("Allies"), 6, "allies") }
            </div>
        }
    }

    let html = block_on(LocalServerRenderer::<StatIconHarness>::new().render());
    assert!(html.contains("stat-icon"));
}
