use super::Props;
use super::menu::render_menu_item;
use super::share::{resolved_epilogue_key, resolved_headline_key};
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
    let player = props
        .game_state
        .party
        .player_name(props.game_state.persona_id.as_deref());
    let epilogue_text = i18n::tr(
        &epilogue_key,
        Some(&std::collections::BTreeMap::from([
            ("name", player),
            ("crew", props.game_state.party.name.as_str()),
        ])),
    );

    html! {
        <section role="region" aria-labelledby="result-title" onkeydown={on_keydown} class="result-screen">
            <div class="result-art"><crate::components::ui::journey_scene::JourneyScene party={Some(props.game_state.party.clone())} road_asset={Some(crate::game::route::road_scene(&props.game_state).to_owned())} day={props.game_state.day} stage={crate::components::ui::journey_scene::SceneStage::Ending(props.game_state.boss.readiness.reached || props.game_state.boss.outcome.attempted)} deep={props.game_state.mode.is_deep()} weather={Some(props.game_state.weather_state.today)}>
                <figcaption class="result-art-caption"><p class="eyebrow">{i18n::t("play.outcome")}</p><h1 id="result-title" class="result-headline">{ &headline_text }</h1><p>{&epilogue_text}</p></figcaption>
            </crate::components::ui::journey_scene::JourneyScene></div>
            <div class="result-profile"><img class="crew-portrait" src={crate::paths::asset_path(&format!("static/img/journey/occupant-{}.png",props.game_state.persona_id.as_deref().unwrap_or("journalist")))} alt="" /><div><p class="eyebrow">{i18n::t("play.profile")}</p><strong>{player}</strong><span>{persona_name(summary)}</span><span>{&summary.mode}</span></div></div>

            {super::crew_story::render(&props.game_state)}
            <details class="ending-scorecard"><summary>{i18n::t("journey.scorecard")}</summary>
            <section class="result-info" aria-labelledby="result-info-heading">
                <h2 id="result-info-heading" class="sr-only">{ i18n::t("result.labels.stats") }</h2>
                { render_metadata(summary) }
            </section>

            <section class="stats-section" aria-labelledby="stats-heading">
                <h2 id="stats-heading">{ i18n::t("result.labels.stats") }</h2>
                { render_stats(summary, crate::game::route::physical_miles(&props.game_state)) }
            </section>


            </details>
            { render_menu(current_focus, on_menu_action) }

            <div aria-live="polite" aria-atomic="true" class="sr-only" id="announcements">
                { announcement }
            </div>
        </section>
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
            </div>

            <div class="score-display">
                <strong>{ i18n::t("result.labels.score") }{": "}</strong>
                <span class="score-value">{ crate::i18n::fmt_number(f64::from(summary.score)) }</span>
            </div>
        </>
    }
}

fn render_stats(summary: &ResultSummary, miles: f32) -> Html {
    html! {
        <dl class="stats-grid">
            <div><dt>{ i18n::t("result.labels.days") }</dt><dd>{ summary.days }</dd></div>
            <div><dt>{ i18n::t("result.labels.encounters") }</dt><dd>{ summary.encounters }</dd></div>
            <div><dt>{ i18n::t("result.labels.receipts") }</dt><dd>{ summary.receipts }</dd></div>
            <div><dt>{ i18n::t("result.labels.allies") }</dt><dd>{ summary.allies }</dd></div>
            <div><dt>{ i18n::t("result.labels.supplies") }</dt><dd>{ summary.supplies }</dd></div>
            <div><dt>{ i18n::t("result.labels.credibility") }</dt><dd>{ summary.credibility }</dd></div>
            <div><dt>{ i18n::t("result.labels.pants_pct") }</dt><dd>{ format!("{pants_pct}%", pants_pct = summary.pants_pct) }</dd></div>
            <div><dt>{ i18n::t("result.labels.breakdowns") }</dt><dd>{ summary.vehicle_breakdowns }</dd></div>
            <div><dt>{ i18n::t("result.labels.miles") }</dt><dd>{ crate::i18n::fmt_number(f64::from(miles).round()) }</dd></div>
            <div><dt>{ i18n::t("result.labels.score_threshold") }</dt><dd>{ crate::i18n::fmt_number(f64::from(summary.score_threshold)) }</dd></div>
            <div><dt>{ i18n::t("result.labels.passed_threshold") }</dt><dd>{ if summary.passed_threshold { i18n::t("result.badges.success") } else { i18n::t("result.badges.fail") } }</dd></div>
            <div><dt>{ i18n::t("result.labels.malnutrition") }</dt><dd>{ summary.malnutrition_days }</dd></div>
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

fn persona_name(summary: &ResultSummary) -> String {
    let key = format!("persona.{}.name", summary.persona_name);
    let translated = i18n::t(&key);
    if translated == key {
        let mut chars = summary.persona_name.chars();
        chars.next().map_or_else(String::new, |c| {
            c.to_uppercase().to_string() + chars.as_str()
        })
    } else {
        translated
    }
}
