use super::pace::{diet_code, diet_preview, pace_code, pace_preview};
use crate::game::{GameState, PacingConfig};
use crate::i18n;
use web_sys::MouseEvent;
use yew::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelMode {
    Main,
    WeatherDetails,
}

pub struct IntentActions<'a> {
    pub on_trade: &'a Callback<MouseEvent>,
    pub on_hunt: &'a Callback<MouseEvent>,
}

pub struct PanelContext<'a> {
    pub travel_blocked: bool,
    pub breakdown_msg: Option<&'a str>,
    pub mode: PanelMode,
    pub weather_details: Html,
    pub weather_info: Html,
    pub logs: &'a [String],
    pub game_state: Option<&'a GameState>,
    pub pacing_config: &'a PacingConfig,
    pub intent_actions: Option<IntentActions<'a>>,
    pub on_open_inventory: &'a Callback<MouseEvent>,
    pub on_open_pace_diet: &'a Callback<MouseEvent>,
    pub on_open_map: &'a Callback<MouseEvent>,
    pub on_toggle_weather_details: &'a Callback<MouseEvent>,
    pub on_click: &'a Callback<MouseEvent>,
}

pub fn render_panel(ctx: &PanelContext<'_>) -> Html {
    html! {
        <section class="panel travel-shell">
            <header class="section-header">
                <h2>{ i18n::t("travel.title") }</h2>
                { ctx.weather_info.clone() }
            </header>

            { render_breakdown(ctx.travel_blocked, ctx.breakdown_msg) }

            { render_body(ctx) }

            { render_footer(ctx) }
        </section>
    }
}

fn render_breakdown(travel_blocked: bool, breakdown_msg: Option<&str>) -> Html {
    if !travel_blocked {
        return Html::default();
    }
    let Some(msg) = breakdown_msg else {
        return Html::default();
    };

    html! {
        <div class="alert breakdown-alert" role="alert" aria-live="assertive">
            <p>{ msg }</p>
            <p>{ i18n::t("vehicle.announce.blocked") }</p>
        </div>
    }
}

fn render_body(ctx: &PanelContext<'_>) -> Html {
    match ctx.mode {
        PanelMode::WeatherDetails => ctx.weather_details.clone(),
        PanelMode::Main => html! {
            <div class="travel-body">
                { render_block_notice(ctx.travel_blocked) }
                { render_current_settings(ctx.game_state, ctx.pacing_config) }
                { render_logs(ctx.logs) }
            </div>
        },
    }
}

fn render_block_notice(travel_blocked: bool) -> Html {
    if travel_blocked {
        html! {
            <p id="breakdown-notice" class="help-text">
                { i18n::t("vehicle.announce.blocked") }
            </p>
        }
    } else {
        Html::default()
    }
}

fn render_current_settings(game_state: Option<&GameState>, pacing_config: &PacingConfig) -> Html {
    let Some(gs) = game_state else {
        return Html::default();
    };

    html! {
        <div class="current-settings" role="status" aria-live="polite">
            <div class="pace-diet-row" role="list">
                <div class="condition-pill pace-pill" role="listitem" title={pace_preview(pacing_config, gs.pace)}>
                    <span class="sprite-badge sprite-pace" aria-hidden="true">{ pace_code(gs.pace) }</span>
                    <span class="condition-label">{ format!("{} {}", i18n::t("menu.pace"), gs.pace) }</span>
                </div>
                <div class="condition-pill diet-pill" role="listitem" title={diet_preview(pacing_config, gs.diet)}>
                    <span class="sprite-badge sprite-diet" aria-hidden="true">{ diet_code(gs.diet) }</span>
                    <span class="condition-label">{ format!("{} {}", i18n::t("menu.diet"), gs.diet) }</span>
                </div>
            </div>
        </div>
    }
}

fn render_logs(logs: &[String]) -> Html {
    if logs.is_empty() {
        return Html::default();
    }

    html! {
        <div class="log" role="log" aria-live="polite">
            { for logs.iter().map(|l| html!{ <p>{l}</p> }) }
        </div>
    }
}

fn render_footer(ctx: &PanelContext<'_>) -> Html {
    let intent_actions = ctx.intent_actions.as_ref().map_or_else(Html::default, |actions| {
        html! {
            <>
                <button onclick={actions.on_trade.clone()} aria-label={i18n::t("travel.trade")} class="retro-btn-secondary">
                    { i18n::t("travel.trade") }
                </button>
                <button onclick={actions.on_hunt.clone()} aria-label={i18n::t("travel.hunt")} class="retro-btn-secondary">
                    { i18n::t("travel.hunt") }
                </button>
            </>
        }
    });
    html! {
        <footer class="panel-footer">
            { intent_actions }
            <button onclick={ctx.on_open_inventory.clone()} aria-label={i18n::t("menu.inventory")} class="retro-btn-secondary">
                { i18n::t("menu.inventory") }
            </button>
            <button onclick={ctx.on_open_pace_diet.clone()} aria-label={i18n::t("pacediet.title")} class="retro-btn-secondary">
                { i18n::t("pacediet.title") }
            </button>
            <button onclick={ctx.on_open_map.clone()} aria-label={i18n::t("map.title")} class="retro-btn-secondary">
                { i18n::t("map.title") }
            </button>
            <button
                onclick={ctx.on_toggle_weather_details.clone()}
                aria-label={i18n::t("weather.details.header")}
                class="retro-btn-secondary"
            >
                { i18n::t("weather.details.header") }
            </button>
            <button
                onclick={ctx.on_click.clone()}
                aria-label={i18n::t("travel.next")}
                class="retro-btn-primary"
                disabled={ctx.travel_blocked}
                aria-describedby={if ctx.travel_blocked { "breakdown-notice" } else { "" }}
            >
                { i18n::t("travel.next") }
            </button>
        </footer>
    }
}
