use super::*;
use futures::executor::block_on;
use yew::LocalServerRenderer;

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
