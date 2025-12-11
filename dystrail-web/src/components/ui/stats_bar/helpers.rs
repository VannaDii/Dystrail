use crate::game::exec_orders::ExecOrder;
use crate::game::personas::PersonasList;
use crate::game::state::Region;
use crate::game::weather::Weather;
use crate::i18n;
use std::collections::BTreeMap;
use std::sync::OnceLock;
use yew::prelude::*;

static PERSONA_NAMES: OnceLock<BTreeMap<String, String>> = OnceLock::new();

pub(super) fn persona_name_for(id: &str) -> String {
    let names = PERSONA_NAMES.get_or_init(|| {
        let json = include_str!("../../../../static/assets/data/personas.json");
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

pub(super) fn persona_initial(name: &str) -> String {
    name.chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect::<String>())
}

pub(super) fn region_label(region: Region) -> String {
    match region {
        Region::Heartland => i18n::t("region.heartland"),
        Region::RustBelt => i18n::t("region.rustbelt"),
        Region::Beltway => i18n::t("region.beltway"),
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

pub(super) const fn exec_order_token(order: ExecOrder) -> &'static str {
    match order {
        ExecOrder::Shutdown => "SD",
        ExecOrder::TravelBanLite => "TB",
        ExecOrder::BookPanic => "BP",
        ExecOrder::TariffTsunami => "TT",
        ExecOrder::DoEEliminated => "DE",
        ExecOrder::WarDeptReorg => "WR",
    }
}

pub(super) const fn weather_sprite_class(weather: Weather) -> &'static str {
    match weather {
        Weather::Clear => "sprite-weather-clear",
        Weather::Storm => "sprite-weather-storm",
        Weather::HeatWave => "sprite-weather-heat",
        Weather::ColdSnap => "sprite-weather-cold",
        Weather::Smoke => "sprite-weather-smoke",
    }
}

pub(super) const fn exec_sprite_class(order: ExecOrder) -> &'static str {
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

pub(super) fn stat_chip(label: String, value: i32, kind: &'static str) -> Html {
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
