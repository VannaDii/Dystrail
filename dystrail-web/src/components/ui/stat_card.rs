//! Shared numeric readout for outcomes, action receipts, weather and the journal.
use crate::{game::Stats, game::journal::ResourceChange, i18n};
use yew::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct StatCard {
    pub key: String,
    pub value: String,
    pub subtitle: String,
    pub tone: &'static str,
}

impl StatCard {
    pub fn render(&self) -> Html {
        self.render_content(None)
    }

    pub fn render_illustrated(&self, asset: &str) -> Html {
        self.render_content(Some(asset))
    }

    fn render_content(&self, asset: Option<&str>) -> Html {
        html! {<li class={classes!("change", "stat-card", self.tone)} data-stat={self.key.clone()}>
            if let Some(asset) = asset {
                <img class="stat-card-art" src={crate::paths::asset_path(asset)} alt="" width="64" height="64" decoding="sync" />
            }
            <span>{crate::app::receipt::label(&self.key)}</span>
            <strong><bdi>{&self.value}</bdi></strong>
            if !self.subtitle.is_empty() {<small><bdi>{&self.subtitle}</bdi></small>}
        </li>}
    }
}

#[must_use]
pub fn resources(before: &Stats, after: &Stats) -> Vec<ResourceChange> {
    [
        ("ux.supplies", before.supplies, after.supplies),
        ("ux.health", before.hp, after.hp),
        ("ux.sanity", before.sanity, after.sanity),
        ("play.credibility", before.credibility, after.credibility),
        ("play.morale", before.morale, after.morale),
        ("play.allies", before.allies, after.allies),
    ]
    .into_iter()
    .filter(|(_, before, after)| before != after)
    .map(|(key, before, after)| ResourceChange {
        key: key.into(),
        before: i64::from(before),
        after: i64::from(after),
    })
    .collect()
}

#[must_use]
pub fn change(resource: &ResourceChange) -> StatCard {
    StatCard {
        key: resource.key.clone(),
        value: crate::app::receipt::delta(resource),
        subtitle: if resource.key.starts_with("store.items.")
            || matches!(resource.key.as_str(), "ux.supplies" | "play.cash")
        {
            i18n::tr(
                "ux.remaining_count",
                Some(&std::collections::BTreeMap::from([(
                    "count",
                    crate::app::receipt::value(&resource.key, resource.after).as_str(),
                )])),
            )
        } else {
            format!(
                "{} → {}",
                crate::app::receipt::value(&resource.key, resource.before),
                crate::app::receipt::value(&resource.key, resource.after)
            )
        },
        tone: if resource.key == "play.elapsed" {
            "neutral"
        } else if resource.after < resource.before {
            "harmful"
        } else {
            "helpful"
        },
    }
}

fn change_cards(before: &Stats, after: &Stats, extra: &[ResourceChange]) -> Vec<StatCard> {
    let duration = extra
        .iter()
        .find(|resource| resource.key == "play.driving_time")
        .and_then(|resource| resource.after.checked_sub(resource.before))
        .and_then(|minutes| u32::try_from(minutes).ok())
        .filter(|minutes| *minutes > 0)
        .map(crate::app::receipt::driving_duration);
    resources(before, after)
        .iter()
        .chain(extra)
        .filter(|resource| resource.key != "play.driving_time")
        .map(|resource| {
            let mut card = change(resource);
            if resource.key == "play.miles"
                && let Some(duration) = &duration
            {
                card.subtitle.clone_from(duration);
            }
            card
        })
        .collect()
}

pub fn render_changes(before: &Stats, after: &Stats, extra: &[ResourceChange]) -> Html {
    let cards = change_cards(before, after, extra);
    if cards.is_empty() {
        return Html::default();
    }
    html! {<ul class="resource-changes" aria-label={i18n::t("ux.outcome")}>
        {for cards.iter().map(StatCard::render)}
    </ul>}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_rollover_keeps_driving_time_in_the_mileage_card() {
        i18n::set_lang("en");
        let stats = Stats::default();
        let extra = [
            ResourceChange {
                key: "play.elapsed".into(),
                before: 1,
                after: 2,
            },
            ResourceChange {
                key: "play.miles".into(),
                before: 1000,
                after: 1600,
            },
            ResourceChange {
                key: "play.driving_time".into(),
                before: 420,
                after: 480,
            },
        ];
        let cards = change_cards(&stats, &stats, &extra);
        assert_eq!(cards.len(), 2);
        let miles = cards.iter().find(|card| card.key == "play.miles").unwrap();
        assert_eq!(miles.value, "+60.0");
        assert_eq!(miles.subtitle, "1 h driving");
        let old_entry = change_cards(&stats, &stats, &extra[..2]);
        assert_eq!(old_entry[1].subtitle, "100.0 → 160.0");
    }
}
