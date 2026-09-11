use super::pace::{diet_preview, pace_preview};
use crate::game::{GameState, PacingConfig};
use crate::i18n;
use web_sys::MouseEvent;
use yew::prelude::*;

pub struct PanelContext<'a> {
    pub travel_blocked: bool,
    pub breakdown_msg: Option<&'a str>,
    pub show_weather_details: bool,
    pub weather_details: Html,
    pub show_pace_diet: bool,
    pub pace_diet_panel: Html,
    pub weather_info: Html,
    pub logs: &'a [String],
    pub game_state: Option<&'a GameState>,
    pub pacing_config: &'a PacingConfig,
    pub on_show_pace_diet: &'a Callback<MouseEvent>,
    pub on_toggle_weather_details: &'a Callback<MouseEvent>,
    pub on_click: &'a Callback<MouseEvent>,
}

pub fn render_panel(ctx: PanelContext) -> Html {
    let overlay = ctx.show_pace_diet || ctx.show_weather_details;
    html! { <section class="travel-controls" aria-labelledby="screen-title">
        <div class="travel-heading"><h1 id="screen-title" tabindex="-1">{i18n::t(if ctx.game_state.is_some_and(|gs|gs.mode.is_deep()) {"ux.road_deep"} else {"ux.road"})}</h1>{ctx.weather_info}</div>
        if ctx.travel_blocked { <div id="breakdown-notice" class="alert" role="alert">
            <p>{ctx.breakdown_msg.unwrap_or_default()}</p><p>{i18n::t("vehicle.announce.blocked")}</p>
        </div> }
        if ctx.show_weather_details { {ctx.weather_details} }
        else if ctx.show_pace_diet { {ctx.pace_diet_panel} }
        else {
            if let Some(gs)=ctx.game_state {
                <p class="current-settings"><span title={pace_preview(ctx.pacing_config,gs.pace)}>{i18n::t(&format!("pacediet.menu.pace_{}",gs.pace.as_str()))}</span>{" · "}<span title={diet_preview(ctx.pacing_config,gs.diet)}>{i18n::t(&format!("pacediet.menu.diet_{}",gs.diet.as_str()))}</span></p>
                if gs.day <= 1 {<p class="onboarding-note">{i18n::t("ux.onboarding")}</p>}
            }
        }
        <div class="travel-actions">
            <button class="retro-btn-primary" onclick={ctx.on_click.clone()} disabled={overlay} aria-describedby={if ctx.travel_blocked {Some("breakdown-notice")} else {None}}>{i18n::t(if ctx.travel_blocked {"ux.resolve_vehicle"} else if ctx.game_state.is_some_and(|gs| gs.day_state.lifecycle.day_initialized) {"ux.continue_today"} else {"ux.next_day"})}</button>
            <button onclick={ctx.on_show_pace_diet.clone()}>{i18n::t("pacediet.title")}</button>
            <button onclick={ctx.on_toggle_weather_details.clone()}>{i18n::t("weather.details.header")}</button>
        </div>
        if let Some(gs)=ctx.game_state { {super::status::render_status(gs)} }
        <details class="journal"><summary>{i18n::t("ux.journal")}</summary><div class="log" role="log">
            {for ctx.logs.iter().rev().take(30).map(|line|html!{<p>{line}</p>})}
        </div></details>
    </section> }
}
