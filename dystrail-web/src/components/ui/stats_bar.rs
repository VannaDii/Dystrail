use crate::game::exec_orders::ExecOrder;
use crate::game::personas::PersonasList;
use crate::game::state::{Region, Stats};
use crate::game::weather::Weather;
use crate::i18n;
use std::collections::BTreeMap;
use std::sync::OnceLock;
use yew::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct WeatherBadge {
    pub weather: Weather,
    pub mitigated: bool,
}

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct Props {
    pub stats: Stats,
    pub day: u32,
    pub region: Region,
    #[prop_or_default]
    pub exec_order: Option<ExecOrder>,
    #[prop_or_default]
    pub persona_id: Option<String>,
    #[prop_or_default]
    pub weather: Option<WeatherBadge>,
}

static PERSONA_NAMES: OnceLock<BTreeMap<String, String>> = OnceLock::new();

fn persona_name_for(id: &str) -> String {
    let names = PERSONA_NAMES.get_or_init(|| {
        let json = include_str!("../../../static/assets/data/personas.json");
        PersonasList::from_json(json)
            .map(|list| {
                list.0
                    .into_iter()
                    .map(|p| (p.id, p.name))
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default()
    });
    names.get(id).cloned().unwrap_or_else(|| id.to_uppercase())
}

fn persona_initial(name: &str) -> String {
    name.chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect::<String>())
}

fn region_label(region: Region) -> String {
    match region {
        crate::game::state::Region::Heartland => i18n::t("region.heartland"),
        crate::game::state::Region::RustBelt => i18n::t("region.rustbelt"),
        crate::game::state::Region::Beltway => i18n::t("region.beltway"),
    }
}

#[must_use]
pub const fn weather_symbol(weather: Weather) -> &'static str {
    match weather {
        Weather::Clear => "☼",
        Weather::Storm => "⛈",
        Weather::HeatWave => "☀",
        Weather::ColdSnap => "❄",
        Weather::Smoke => "☁",
    }
}

const fn exec_order_token(order: ExecOrder) -> &'static str {
    match order {
        ExecOrder::Shutdown => "SD",
        ExecOrder::TravelBanLite => "TB",
        ExecOrder::BookPanic => "BP",
        ExecOrder::TariffTsunami => "TT",
        ExecOrder::DoEEliminated => "DE",
        ExecOrder::WarDeptReorg => "WR",
    }
}

const fn weather_sprite_class(weather: Weather) -> &'static str {
    match weather {
        Weather::Clear => "sprite-weather-clear",
        Weather::Storm => "sprite-weather-storm",
        Weather::HeatWave => "sprite-weather-heat",
        Weather::ColdSnap => "sprite-weather-cold",
        Weather::Smoke => "sprite-weather-smoke",
    }
}

const fn exec_sprite_class(order: ExecOrder) -> &'static str {
    match order {
        ExecOrder::Shutdown => "sprite-eo-shutdown",
        ExecOrder::TravelBanLite => "sprite-eo-travelban",
        ExecOrder::BookPanic => "sprite-eo-book",
        ExecOrder::TariffTsunami => "sprite-eo-tariff",
        ExecOrder::DoEEliminated => "sprite-eo-doe",
        ExecOrder::WarDeptReorg => "sprite-eo-war",
    }
}

fn stat_icon(kind: &str) -> Html {
    let stroke = "#1A1000";
    let fill = "var(--text-bright)";
    match kind {
        "supplies" => html! {
            <svg width="16" height="16" viewBox="0 0 24 24" role="presentation" aria-hidden="true" class="stat-icon">
                <rect x="4" y="5" width="16" height="14" rx="2" fill={fill} stroke={stroke} stroke-width="1.5"/>
                <path d="M8 9h8M8 13h5" stroke={stroke} stroke-width="1.5" stroke-linecap="round"/>
            </svg>
        },
        "hp" => html! {
            <svg width="16" height="16" viewBox="0 0 24 24" role="presentation" aria-hidden="true" class="stat-icon">
                <path d="M12 20s7-4.2 7-10a5 5 0 0 0-9-2 5 5 0 0 0-9 2c0 5.8 11 10 11 10Z" fill={fill} stroke={stroke} stroke-width="1.5" stroke-linejoin="round"/>
            </svg>
        },
        "sanity" => html! {
            <svg width="16" height="16" viewBox="0 0 24 24" role="presentation" aria-hidden="true" class="stat-icon">
                <path d="M12 4c-3.5 0-6 2.7-6 6.4 0 4.3 3 6.9 6 9.6 3-2.7 6-5.3 6-9.6C18 6.7 15.5 4 12 4Zm0 3.2a2.8 2.8 0 1 1 0 5.6 2.8 2.8 0 0 1 0-5.6Z" fill={fill} stroke={stroke} stroke-width="1.2" />
            </svg>
        },
        "cred" => html! {
            <svg width="16" height="16" viewBox="0 0 24 24" role="presentation" aria-hidden="true" class="stat-icon">
                <path d="M4.5 8.5 12 4l7.5 4.5v7L12 20l-7.5-4.5Z" fill={fill} stroke={stroke} stroke-width="1.5" />
                <path d="M9.5 12.5 12 14l2.5-1.5" stroke={stroke} stroke-width="1.5" stroke-linecap="round" />
            </svg>
        },
        "morale" => html! {
            <svg width="16" height="16" viewBox="0 0 24 24" role="presentation" aria-hidden="true" class="stat-icon">
                <circle cx="12" cy="12" r="5.5" fill={fill} stroke={stroke} stroke-width="1.5" />
                <path d="M12 6v-2M12 20v-2M6 12H4M20 12h-2M7.8 7.8 6.4 6.4M17.6 17.6l-1.4-1.4M7.8 16.2 6.4 17.6M17.6 6.4l-1.4 1.4" stroke={stroke} stroke-width="1.2" stroke-linecap="round"/>
            </svg>
        },
        "allies" => html! {
            <svg width="16" height="16" viewBox="0 0 24 24" role="presentation" aria-hidden="true" class="stat-icon">
                <circle cx="8" cy="9" r="3" fill={fill} stroke={stroke} stroke-width="1.2"/>
                <circle cx="16" cy="9" r="3" fill={fill} stroke={stroke} stroke-width="1.2"/>
                <path d="M3.5 19a4.5 4.5 0 0 1 9 0M11.5 19a4.5 4.5 0 0 1 9 0" fill="none" stroke={stroke} stroke-width="1.4" stroke-linecap="round"/>
            </svg>
        },
        _ => Html::default(),
    }
}

fn stat_chip(label: String, value: i32, kind: &'static str) -> Html {
    let value_text = crate::i18n::fmt_number(f64::from(value));
    html! {
        <div class="stat-chip" role="listitem">
            { stat_icon(kind) }
            <div class="stat-copy">
                <span class="stat-label">{ label }</span>
                <span class="stat-value">{ value_text }</span>
            </div>
        </div>
    }
}

#[function_component(StatsBar)]
pub fn stats_bar(p: &Props) -> Html {
    let region_text = region_label(p.region);
    let day_str = crate::i18n::fmt_number(f64::from(p.day));
    let pants_raw = p.stats.pants;
    let pants_str = crate::i18n::fmt_number(f64::from(pants_raw));
    let persona_name = p
        .persona_id
        .as_deref()
        .map_or_else(|| i18n::t("persona.persona"), persona_name_for);
    let persona_initial = persona_initial(&persona_name);

    let day_region_text = {
        let mut m = BTreeMap::new();
        m.insert("day", day_str.as_str());
        m.insert("region", region_text.as_str());
        i18n::tr("stats.day_region", Some(&m))
    };

    let pants_text = {
        let mut m = BTreeMap::new();
        m.insert("pct", pants_str.as_str());
        i18n::tr("stats.pants", Some(&m))
    };

    let pants_meter_class = classes!(
        "pants-meter",
        if p.stats.pants >= 90 {
            Some("pants-critical")
        } else if p.stats.pants >= 70 {
            Some("pants-warn")
        } else {
            None
        }
    );

    html! {
        <section aria-label={i18n::t("stats.location")} class="panel stats-panel header-row" role="region">
            <div class="header-persona">
                <div class={classes!("persona-portrait", if p.stats.pants >= 90 { Some("portrait-pulse") } else { None })} aria-hidden="true">
                    <span class="portrait-initial">{ persona_initial }</span>
                </div>
                <div class="persona-copy">
                    <p class="persona-name">{ persona_name }</p>
                    <p class="muted">{ day_region_text }</p>
                </div>
            </div>
            <div class="stat-chip-grid" role="list" aria-label={i18n::t("stats.location")}>
                { stat_chip(i18n::t("stats.sup_short"), p.stats.supplies, "supplies") }
                { stat_chip(i18n::t("stats.hp_short"), p.stats.hp, "hp") }
                { stat_chip(i18n::t("stats.sanity_short"), p.stats.sanity, "sanity") }
                { stat_chip(i18n::t("stats.cred_short"), p.stats.credibility, "cred") }
                { stat_chip(i18n::t("stats.mor_short"), p.stats.morale, "morale") }
                { stat_chip(i18n::t("stats.allies_short"), p.stats.allies, "allies") }
            </div>
            <div class="conditions-stack">
                <div class={pants_meter_class} role="meter" aria-label={i18n::t("stats.pants_label")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={pants_raw.to_string()}>
                    <div class="meter-label">{ pants_text.clone() }</div>
                    <div class="bar-wrap slim">
                        <div class={classes!("bar-fill", if p.stats.pants >= 90 { Some("bar-fill-pulse") } else { Some("bar-fill-glow") })} style={format!("width: {pants}%", pants = p.stats.pants)}></div>
                    </div>
                </div>
                <div class="conditions-row">
                    {
                        p.weather.as_ref().map(|w| {
                            let label = i18n::t(w.weather.i18n_key());
                            let condition_label = format!("{} {}", i18n::t("weather.title"), label);
                            let sprite_class = weather_sprite_class(w.weather);
                            html! {
                                <div class={classes!("condition-pill", "weather-pill", if w.mitigated { Some("mitigated") } else { None })} aria-label={condition_label.clone()} title={condition_label}>
                                    <span class={classes!("sprite-badge", "sprite-weather", sprite_class)} aria-hidden="true">{ weather_symbol(w.weather) }</span>
                                    <span class="condition-label">{ label }</span>
                                </div>
                            }
                        }).unwrap_or_default()
                    }
                    <div class="exec-row" aria-live="polite">
                    {
                        p.exec_order.map_or_else(Html::default, |order| {
                            let order_label = i18n::t(order.name_key());
                            let abbr = exec_order_token(order);
                            let sprite_class = exec_sprite_class(order);
                            let full_label = format!("{} {}", i18n::t("eo.prefix"), order_label);
                            html! {
                                <div class="condition-pill exec-pill" aria-label={full_label.clone()} title={full_label}>
                                    <span class={classes!("sprite-badge", "sprite-eo", sprite_class)} aria-hidden="true">{ abbr }</span>
                                    <span class="condition-label">{ order_label }</span>
                                </div>
                            }
                        })
                    }
                    </div>
                </div>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
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
}
