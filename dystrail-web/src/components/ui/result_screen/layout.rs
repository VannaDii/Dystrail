use super::Props;
use super::menu::render_menu_item;
use super::share::{resolved_epilogue_key, resolved_headline_key};
use crate::game::MechanicalPolicyId;
use crate::game::ResultSummary;
use crate::i18n;
use yew::prelude::*;

pub fn render_body(
    props: &Props,
    summary: &ResultSummary,
    current_focus: u8,
    announcement: String,
    on_keydown: Callback<KeyboardEvent>,
    on_menu_action: &Callback<u8>,
) -> Html {
    let headline_key = resolved_headline_key(summary, props);
    let epilogue_key = resolved_epilogue_key(summary, props);
    let headline_text = i18n::t(&headline_key);
    let epilogue_text = i18n::t(&epilogue_key);
    let show_thresholds = props.game_state.mechanical_policy == MechanicalPolicyId::DystrailLegacy;

    html! {
        <main role="main" aria-labelledby="result-title" onkeydown={on_keydown} tabindex="0" class="result-screen">
            <h1 id="result-title" class="result-headline">{ &headline_text }</h1>

            <section class="result-info" aria-labelledby="result-info-heading">
                <h2 id="result-info-heading" class="sr-only">{ i18n::t("result.labels.stats") }</h2>
                { render_metadata(summary) }
            </section>

            <section class="stats-section" aria-labelledby="stats-heading">
                <h2 id="stats-heading">{ i18n::t("result.labels.stats") }</h2>
                { render_stats(summary, show_thresholds) }
            </section>

            <section class="epilogue-section">
                <p class="epilogue">{ &epilogue_text }</p>
            </section>

            { render_menu(current_focus, on_menu_action) }

            <div aria-live="polite" aria-atomic="true" class="sr-only" id="announcements">
                { announcement }
            </div>
        </main>
    }
}

fn render_metadata(summary: &ResultSummary) -> Html {
    html! {
        <>
            <div class="result-metadata">
                <span class="metadata-item">
                    <strong>{ i18n::t("result.labels.seed") }{": "}</strong>
                    { &summary.seed }
                </span>
                <span class="metadata-item">
                    <strong>{ i18n::t("result.labels.persona") }{": "}</strong>
                    { &summary.persona_name }{ " (" }{ &summary.mult_str }{ ")" }
                </span>
                <span class="metadata-item">
                    <strong>{ i18n::t("result.labels.mode") }{": "}</strong>
                    { &summary.mode }
                    { if summary.dp_badge {
                        html! { <span class="badge">{ i18n::t("result.badges.mode_deep") }</span> }
                    } else {
                        html! {}
                    }}
                </span>
            </div>

            <div class="score-display">
                <strong>{ i18n::t("result.labels.score") }{": "}</strong>
                <span class="score-value">{ crate::i18n::fmt_number(f64::from(summary.score)) }</span>
            </div>
        </>
    }
}

fn render_stats(summary: &ResultSummary, show_thresholds: bool) -> Html {
    html! {
        <dl class="stats-grid">
            <dt>{ i18n::t("result.labels.days") }</dt>
            <dd>{ summary.days }</dd>
            <dt>{ i18n::t("result.labels.encounters") }</dt>
            <dd>{ summary.encounters }</dd>
            <dt>{ i18n::t("result.labels.receipts") }</dt>
            <dd>{ summary.receipts }</dd>
            <dt>{ i18n::t("result.labels.allies") }</dt>
            <dd>{ summary.allies }</dd>
            <dt>{ i18n::t("result.labels.supplies") }</dt>
            <dd>{ summary.supplies }</dd>
            <dt>{ i18n::t("result.labels.credibility") }</dt>
            <dd>{ summary.credibility }</dd>
            <dt>{ i18n::t("result.labels.pants_pct") }</dt>
            <dd>{ format!("{pants_pct}%", pants_pct = summary.pants_pct) }</dd>
            <dt>{ i18n::t("result.labels.breakdowns") }</dt>
            <dd>{ summary.vehicle_breakdowns }</dd>
            <dt>{ i18n::t("result.labels.miles") }</dt>
            <dd>{ crate::i18n::fmt_number(f64::from(summary.miles_traveled).round()) }</dd>
            { if show_thresholds {
                html! {
                    <>
                        <dt>{ i18n::t("result.labels.score_threshold") }</dt>
                        <dd>{ crate::i18n::fmt_number(f64::from(summary.score_threshold)) }</dd>
                        <dt>{ i18n::t("result.labels.passed_threshold") }</dt>
                        <dd>{ if summary.passed_threshold { i18n::t("result.badges.success") } else { i18n::t("result.badges.fail") } }</dd>
                    </>
                }
            } else {
                html! {}
            }}
            <dt>{ i18n::t("result.labels.malnutrition") }</dt>
            <dd>{ summary.malnutrition_days }</dd>
        </dl>
    }
}

fn render_menu(current_focus: u8, on_menu_action: &Callback<u8>) -> Html {
    html! {
        <nav class="result-menu" role="menu" aria-label={ i18n::t("result.title") }>
            <ul role="none">
                { render_menu_item(current_focus, 1, &i18n::t("result.menu.copy_share"), on_menu_action) }
                { render_menu_item(current_focus, 2, &i18n::t("result.menu.copy_seed"), on_menu_action) }
                { render_menu_item(current_focus, 3, &i18n::t("result.menu.replay_seed"), on_menu_action) }
                { render_menu_item(current_focus, 4, &i18n::t("result.menu.new_run"), on_menu_action) }
                { render_menu_item(current_focus, 5, &i18n::t("result.menu.export"), on_menu_action) }
                { render_menu_item(current_focus, 0, &i18n::t("result.menu.title"), on_menu_action) }
            </ul>
        </nav>
    }
}
