use super::Props;
use super::menu::render_menu_item;
use super::share::{resolved_epilogue_key, resolved_headline_key};
use crate::components::ui::character_portrait::{self, Expression};
use crate::components::ui::stat_card::StatCard;
use crate::game::ResultSummary;
use crate::i18n;
use yew::prelude::*;

pub fn render_body(
    props: &Props,
    summary: &ResultSummary,
    current_focus: u8,
    announcement: String,
    on_clear: Callback<()>,
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
    let gs = &props.game_state;
    let arrived = gs.miles_traveled_actual >= gs.trail_distance || gs.boss.outcome.attempted;
    let progress = crate::i18n::fmt_number(
        (f64::from(gs.miles_traveled_actual) / f64::from(gs.trail_distance.max(1.0)) * 100.0)
            .clamp(0.0, 100.0)
            .round(),
    );
    let progress_text = i18n::tr(
        "result.route_progress",
        Some(&std::collections::BTreeMap::from([(
            "percent",
            progress.as_str(),
        )])),
    );

    html! {
        <section role="region" aria-labelledby="result-title" onkeydown={on_keydown} class="result-screen">
            <div class="result-art"><crate::components::ui::journey_scene::JourneyScene day={gs.day} hour={u8::try_from(gs.continuity.clock_minutes / 60).unwrap_or(8)} stage={crate::components::ui::journey_scene::SceneStage::Ending(arrived)} deep={gs.mode.is_deep()} weather={Some(gs.weather_state.today)}>
                <figcaption class="result-art-caption">
                    <div class="result-outcome"><p class="eyebrow">{i18n::t("play.outcome")}</p><h1 id="result-title" class="result-headline">{ &headline_text }</h1><p class="result-epilogue">{&epilogue_text}</p><p class="result-location">{super::super::route_map::location::location(gs)}<span>{progress_text}</span></p></div>
                    <div class="result-profile">{character_portrait::framed(gs.persona_id.as_deref().unwrap_or("journalist"),player,Expression::ending(gs))}<div><p class="eyebrow">{i18n::t("play.profile")}</p><span>{persona_name(summary)}</span><span>{&summary.mode}</span></div></div>
                </figcaption>
            </crate::components::ui::journey_scene::JourneyScene></div>

            {super::crew_story::render(&props.game_state)}
            if let Some(report)=&gs.boss.hearing {{crate::pages::boss::summary::scorecard(report)}}
            <section class="ending-scorecard" aria-labelledby="scorecard-title"><h2 id="scorecard-title">{i18n::t("journey.scorecard")}</h2>
            <section class="result-info" aria-labelledby="result-info-heading">
                <h2 id="result-info-heading" class="sr-only">{ i18n::t("result.labels.stats") }</h2>
                { render_metadata(summary) }
            </section>

            <section class="stats-section" aria-labelledby="stats-heading">
                <h2 id="stats-heading">{ i18n::t("result.labels.stats") }</h2>
                { render_stats(summary, crate::game::route::physical_miles(&props.game_state)) }
            </section>


            </section>
            { render_menu(current_focus, on_menu_action) }

            <crate::components::status_notice::StatusNotice message={announcement} {on_clear} />
        </section>
    }
}

fn render_metadata(summary: &ResultSummary) -> Html {
    html! {
        <div class="result-metadata">
            <span class="metadata-item">
                <strong>{ i18n::t("result.labels.seed") }{": "}</strong>
                <bdi>{ &summary.seed }</bdi>
            </span>
        </div>
    }
}

fn render_stats(summary: &ResultSummary, miles: f32) -> Html {
    let numeric = [
        ("result.labels.score", f64::from(summary.score)),
        ("result.labels.days", f64::from(summary.days)),
        ("result.labels.encounters", f64::from(summary.encounters)),
        ("result.labels.receipts", f64::from(summary.receipts)),
        ("result.labels.allies", f64::from(summary.allies)),
        ("result.labels.supplies", f64::from(summary.supplies)),
        ("result.labels.credibility", f64::from(summary.credibility)),
        (
            "result.labels.breakdowns",
            f64::from(summary.vehicle_breakdowns),
        ),
        ("result.labels.miles", f64::from(miles).round()),
        (
            "result.labels.score_threshold",
            f64::from(summary.score_threshold),
        ),
        (
            "result.labels.malnutrition",
            f64::from(summary.malnutrition_days),
        ),
    ];
    html! {
        <ul class="stats-grid result-stats">
            {for numeric.into_iter().map(|(key, value)| StatCard {
                key: key.into(),
                value: i18n::fmt_number(value),
                subtitle: String::new(),
                tone: "neutral",
            }.render())}
            {StatCard {
                key: "result.labels.passed_threshold".into(),
                value: i18n::t(if summary.passed_threshold { "result.badges.success" } else { "result.badges.fail" }),
                subtitle: String::new(),
                tone: if summary.passed_threshold { "helpful" } else { "harmful" },
            }.render()}
        </ul>
    }
}

fn render_menu(current_focus: u8, on_menu_action: &Callback<u8>) -> Html {
    html! {
        <nav class="result-menu" role="menu" aria-label={ i18n::t("result.title") }>
            <ul role="none">
                { render_menu_item(current_focus, 1, &i18n::t("result.compose.title"), on_menu_action) }
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
