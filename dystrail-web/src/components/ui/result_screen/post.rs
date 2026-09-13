//! Share content is derived from the actual ending and physical route progress.
use super::{Props, share};
use crate::{game::ResultSummary, i18n};

pub const PLAY_URL: &str = "https://dystrail.com/play/";

#[derive(Clone, PartialEq, Eq)]
pub struct Post {
    pub headline: String,
    pub player: String,
    pub persona: String,
    pub mode: String,
    pub location: String,
    pub progress: String,
    pub seed: String,
    pub text: String,
    pub image_alt: String,
    pub avatar: String,
    pub arrived: bool,
    pub rtl: bool,
    pub stats: Vec<(String, String)>,
}

#[must_use]
pub fn create(props: &Props, summary: &ResultSummary) -> Post {
    let gs = &props.game_state;
    let player = gs.party.player_name(gs.persona_id.as_deref()).to_owned();
    let headline = i18n::t(&share::resolved_headline_key(summary, props));
    let persona_id = gs.persona_id.as_deref().unwrap_or("journalist");
    let persona = i18n::t(&format!("persona.{persona_id}.name"));
    let location = super::super::route_map::location::location(gs);
    let miles = i18n::fmt_number(f64::from(crate::game::route::physical_miles(gs)).round());
    let percent = i18n::fmt_number(
        (f64::from(gs.miles_traveled_actual) / f64::from(gs.trail_distance.max(1.0)) * 100.0)
            .clamp(0.0, 100.0)
            .round(),
    );
    let progress = i18n::tr(
        "result.route_progress",
        Some(&std::collections::BTreeMap::from([(
            "percent",
            percent.as_str(),
        )])),
    );
    let stats = vec![
        (i18n::t("result.labels.miles"), miles),
        (
            i18n::t("result.labels.days"),
            i18n::fmt_number(f64::from(summary.days)),
        ),
        (
            i18n::t("result.labels.receipts"),
            i18n::fmt_number(f64::from(summary.receipts)),
        ),
        (
            i18n::t("result.labels.allies"),
            i18n::fmt_number(f64::from(summary.allies)),
        ),
        (
            i18n::t("result.labels.credibility"),
            i18n::fmt_number(f64::from(summary.credibility)),
        ),
        (
            i18n::t("result.labels.score"),
            i18n::fmt_number(f64::from(summary.score)),
        ),
    ];
    let stat_text = stats
        .iter()
        .map(|(label, value)| format!("{label}: {value}"))
        .collect::<Vec<_>>()
        .join(" · ");
    let introduction = i18n::tr(
        "result.compose.introduction",
        Some(&std::collections::BTreeMap::from([
            ("name", player.as_str()),
            ("persona", persona.as_str()),
        ])),
    );
    let run = share::interpolate_template(&i18n::t("result.compose.run"), summary, &headline);
    let text = format!(
        "{introduction}\n{headline}\n{location} · {progress}\n{stat_text}\n{run}\n{PLAY_URL}"
    );
    Post {
        image_alt: format!(
            "Dystopian Trail. {headline}. {player}, {persona}. {location}. {progress}. {stat_text}."
        ),
        headline,
        player,
        persona,
        location,
        progress,
        text,
        stats,
        seed: summary.seed.clone(),
        mode: summary.mode.clone(),
        avatar: format!("static/img/journey/occupant-{persona_id}.png"),
        arrived: gs.miles_traveled_actual >= gs.trail_distance || gs.boss.outcome.attempted,
        rtl: i18n::is_rtl(),
    }
}
