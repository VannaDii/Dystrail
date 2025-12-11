use super::pace::{diet_code, diet_preview, pace_code, pace_preview};
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
    html! {
        <section class="panel travel-shell">
            <header class="section-header">
                <h2>{ i18n::t("travel.title") }</h2>
                { ctx.weather_info }
            </header>

            { render_breakdown(ctx.travel_blocked, ctx.breakdown_msg) }

            { render_body(
                ctx.travel_blocked,
                ctx.show_weather_details,
                ctx.weather_details,
                ctx.show_pace_diet,
                ctx.pace_diet_panel,
                ctx.game_state,
                ctx.pacing_config,
                ctx.logs,
            ) }

            { render_footer(
                ctx.travel_blocked,
                ctx.on_show_pace_diet,
                ctx.on_toggle_weather_details,
                ctx.on_click,
            ) }
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

#[allow(clippy::too_many_arguments)]
fn render_body(
    travel_blocked: bool,
    show_weather_details: bool,
    weather_details: Html,
    show_pace_diet: bool,
    pace_diet_panel: Html,
    game_state: Option<&GameState>,
    pacing_config: &PacingConfig,
    logs: &[String],
) -> Html {
    if show_weather_details {
        return weather_details;
    }
    if show_pace_diet {
        return pace_diet_panel;
    }

    html! {
        <div class="travel-body">
            { render_block_notice(travel_blocked) }
            { render_current_settings(game_state, pacing_config) }
            { render_logs(logs) }
        </div>
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

fn render_footer(
    travel_blocked: bool,
    on_show_pace_diet: &Callback<MouseEvent>,
    on_toggle_weather_details: &Callback<MouseEvent>,
    on_click: &Callback<MouseEvent>,
) -> Html {
    html! {
        <footer class="panel-footer">
            <button onclick={on_show_pace_diet.clone()} aria-label={i18n::t("pacediet.title")} class="retro-btn-secondary">
                { i18n::t("pacediet.title") }
            </button>
            <button
                onclick={on_toggle_weather_details.clone()}
                aria-label={i18n::t("weather.details.header")}
                class="retro-btn-secondary"
            >
                { i18n::t("weather.details.header") }
            </button>
            <button
                onclick={on_click.clone()}
                aria-label={i18n::t("travel.next")}
                class="retro-btn-primary"
                disabled={travel_blocked}
                aria-describedby={if travel_blocked { "breakdown-notice" } else { "" }}
            >
                { i18n::t("travel.next") }
            </button>
        </footer>
    }
}
