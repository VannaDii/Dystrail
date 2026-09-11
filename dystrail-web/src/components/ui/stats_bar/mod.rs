//! Compact survival HUD shared by every in-run screen.
mod helpers;
#[cfg(test)]
mod tests;
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

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct Props {
    pub stats: Stats,
    pub day: u32,
    pub region: Region,
    #[prop_or_default]
    pub moving: bool,
    #[prop_or(8)]
    pub clock_hour: u8,
    #[prop_or_default]
    pub clock_minute: u8,
    #[prop_or_default]
    pub exec_order: Option<ExecOrder>,
    #[prop_or_default]
    pub persona_id: Option<String>,
    #[prop_or_default]
    pub weather: Option<WeatherBadge>,
}

#[function_component(StatsBar)]
pub fn stats_bar(p: &Props) -> Html {
    html! { <section class="survival-hud" aria-label={i18n::t("play.crew")}>
        <dl class="critical-stats">
            {for [
                ("ux.supplies", "play.supplies_help",p.stats.supplies,p.stats.supplies<=2),
                ("ux.health", "play.health_help",p.stats.hp,p.stats.hp<=2),
                ("ux.sanity", "play.sanity_help",p.stats.sanity,p.stats.sanity<=2),
                ("ux.pants", "ux.pants_help",p.stats.pants,p.stats.pants>=80),
                ("play.credibility", "play.cred_help",p.stats.credibility,false),
                ("play.morale", "play.morale_help",p.stats.morale,false),
                ("play.allies", "play.allies_help",p.stats.allies,false),
            ].into_iter().map(|(key,help,value,critical)| html!{
                <div class={classes!("hud-stat", critical.then_some("critical"))}>
                    <dt>{i18n::t(key)}<crate::components::ui::context_help::ContextHelp title={i18n::t(key)} text={i18n::t(help)} /></dt>
                    <dd>{if key=="ux.pants" {format!("{value}%")} else {i18n::fmt_number(f64::from(value))}}</dd>
                </div>
            })}
        </dl>
        <div class="hud-context-row"><p class="hud-context"><span aria-hidden="true">{"◷ "}</span><super::game_clock::GameClock day={p.day} hour={p.clock_hour} minute={p.clock_minute} moving={p.moving} /></p>
        <div class="hud-conditions">
            if let Some(w)=&p.weather {<span class={helpers::weather_sprite_class(w.weather)}><span aria-hidden="true">{helpers::weather_symbol(w.weather)}</span>{i18n::t(w.weather.i18n_key())}</span>}
            if let Some(order)=p.exec_order {<span class={helpers::exec_sprite_class(order)}>{i18n::t(order.name_key())}</span>}
        </div></div>
    </section> }
}

pub use helpers::weather_symbol;
