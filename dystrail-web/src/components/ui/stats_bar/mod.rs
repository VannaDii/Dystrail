mod helpers;
#[cfg(test)]
mod tests;

use crate::game::exec_orders::ExecOrder;
use crate::game::state::{Region, Stats};
use crate::game::weather::Weather;
use crate::i18n;
use helpers::{
    exec_order_token, exec_sprite_class, persona_initial, persona_name_for, region_label,
    stat_chip, weather_sprite_class,
};
use std::collections::BTreeMap;
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
                                    <span class={classes!("sprite-badge", "sprite-weather", sprite_class)} aria-hidden="true">{ helpers::weather_symbol(w.weather) }</span>
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

pub use helpers::weather_symbol;
