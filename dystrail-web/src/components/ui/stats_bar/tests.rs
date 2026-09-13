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
    };
    let props = Props {
        part: super::HudPart::All,
        stats,
        receipts: 3,
        moving: false,
        clock_hour: 8,
        clock_minute: 0,
        pace: Some(crate::game::PaceId::Steady),
        diet: Some(crate::game::DietId::Mixed),
        day: 9,
        region: Region::RustBelt,
        exec_order: None,
        persona_id: None,
        policy_readout: None,
        weather_readout: None,
        trip_resources: Html::default(),
        trip_destination: Html::default(),
        weather: Some(WeatherBadge {
            weather: Weather::Clear,
            mitigated: false,
        }),
    };

    let html = block_on(LocalServerRenderer::<StatsBar>::with_props(props).render());
    assert!(html.contains("Day 9"), "game day should appear: {html}");
    assert!(!html.contains("Pants"));
    assert!(html.contains("data-stat=\"ux.receipt\""));
    assert_eq!(html.matches("class=\"hud-stat\"").count(), 7);
    let evidence = html.split("data-stat=\"ux.receipt\"").nth(1).unwrap();
    assert!(
        evidence
            .split("</div>")
            .next()
            .unwrap()
            .contains("<dd>3</dd>")
    );
    assert!(
        html.contains("Health"),
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
        part: super::HudPart::All,
        stats: Stats::default(),
        receipts: 0,
        moving: false,
        clock_hour: 8,
        clock_minute: 0,
        pace: Some(crate::game::PaceId::Steady),
        diet: Some(crate::game::DietId::Mixed),
        day: 1,
        region: Region::Heartland,
        exec_order: Some(ExecOrder::TariffTsunami),
        persona_id: None,
        policy_readout: None,
        weather_readout: None,
        trip_resources: Html::default(),
        trip_destination: Html::default(),
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
