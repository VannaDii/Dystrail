//! Compact survival HUD shared by every in-run screen.
mod helpers;
pub mod policy;
#[cfg(test)]
mod tests;
pub mod weather;
use crate::game::{
    exec_orders::ExecOrder,
    state::{Region, Stats},
    weather::Weather,
};
use crate::i18n;
use yew::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct WeatherBadge {
    pub weather: Weather,
    pub mitigated: bool,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum HudPart {
    #[default]
    All,
    Resources,
    Conditions,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub stats: Stats,
    pub receipts: usize,
    pub day: u32,
    #[prop_or_default]
    pub pace: Option<crate::game::PaceId>,
    #[prop_or_default]
    pub diet: Option<crate::game::DietId>,
    pub region: Region,
    #[prop_or_default]
    pub part: HudPart,
    #[prop_or_default]
    pub moving: bool,
    #[prop_or(8)]
    pub clock_hour: u8,
    #[prop_or_default]
    pub clock_minute: u8,
    #[prop_or_default]
    pub exec_order: Option<ExecOrder>,
    #[prop_or_default]
    pub policy_readout: Option<policy::Readout>,
    #[prop_or_default]
    pub persona_id: Option<String>,
    #[prop_or_default]
    pub weather: Option<WeatherBadge>,
    #[prop_or_default]
    pub weather_readout: Option<weather::Readout>,
    #[prop_or_default]
    pub trip_resources: Html,
    #[prop_or_default]
    pub trip_destination: Html,
}

#[function_component(StatsBar)]
pub fn stats_bar(p: &Props) -> Html {
    crate::i18n::use_language();
    html! { <section class={classes!("survival-hud",(p.part==HudPart::Resources).then_some("resource-hud"),(p.part==HudPart::Conditions).then_some("conditions-hud"))} aria-label={i18n::t(if p.part==HudPart::Conditions {"journey.conditions"}else{"play.crew"})}>
        if p.part!=HudPart::Conditions {
        <dl class="critical-stats">
            {for [
                ("ux.supplies", "play.supplies_help",p.stats.supplies,p.stats.supplies<=2),
                ("ux.health", "play.health_help",p.stats.hp,p.stats.hp<=2),
                ("ux.sanity", "play.sanity_help",p.stats.sanity,p.stats.sanity<=2),
                ("play.credibility", "play.cred_help",p.stats.credibility,false),
                ("ux.receipt", "play.receipts_help",i32::try_from(p.receipts).unwrap_or(i32::MAX),false),
                ("play.morale", "play.morale_help",p.stats.morale,false),
                ("play.allies", "play.allies_help",p.stats.allies,false),
            ].into_iter().map(|(key,help,value,critical)| html!{
                <div data-stat={key} class={classes!("hud-stat", critical.then_some("critical"))}>
                    <dt>{i18n::t(key)}<crate::components::ui::context_help::ContextHelp title={i18n::t(key)} text={i18n::t(help)} /></dt>
                    <dd>{i18n::fmt_number(f64::from(value))}</dd>
                </div>
            })}
        </dl>
        }
        if p.part!=HudPart::Resources {
        <div class="hud-context-row"><div class="hud-primary-row"><div class="hud-journey-settings"><p class="hud-context">{super::journey_icon::render("clock")}<super::game_clock::GameClock day={p.day} hour={p.clock_hour} minute={p.clock_minute} moving={p.moving} /></p>
            if let Some(pace)=p.pace {<span class="hud-setting" aria-label={format!("{}: {}",i18n::t("play.pace"),i18n::t(&format!("play.{}",pace.as_str())))}>{super::journey_icon::render(pace.as_str())}<span class="hud-setting-name">{i18n::t(&format!("play.{}",pace.as_str()))}</span></span>}
            if let Some(diet)=p.diet {<span class="hud-setting" aria-label={format!("{}: {}",i18n::t("play.diet"),i18n::t(&format!("play.{}",diet.as_str())))}>{super::journey_icon::render(diet.as_str())}<span class="hud-setting-name">{i18n::t(&format!("play.{}",diet.as_str()))}</span></span>}
        </div>
        {p.trip_resources.clone()}</div><div class="hud-dynamic-row">{p.trip_destination.clone()}
        <div class="hud-conditions" role="group" aria-label={i18n::t("eo.hud.area")}>
            if let Some(w)=&p.weather {<weather::WeatherIndicator badge={w.clone()} readout={p.weather_readout.clone()} />}
            if let Some(order)=p.exec_order {<policy::PolicyIndicator {order} readout={p.policy_readout.clone()} />}
        </div></div></div>
        }
    </section> }
}

pub use helpers::weather_symbol;
