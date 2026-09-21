//! Localized roadside advertising. This presentation selection never touches simulation RNG.
use crate::{game::Region, i18n};
use yew::prelude::*;

#[must_use]
pub fn selected(region: Region, seed: u64, day: u32) -> &'static str {
    let pair = match region {
        Region::PacificCoast => ["clean_air", "public_land"],
        Region::MountainWest => ["energy", "weather"],
        Region::Southwest => ["water", "wellness"],
        Region::Heartland => ["tariffs", "farm"],
        Region::RustBelt => ["jobs", "repair"],
        Region::Beltway => ["access", "transparency"],
    };
    // Retain one sign throughout a simulated day, including paused/reloaded views.
    // The seed and day are already part of the save and bare-code replay contract.
    pair[usize::from((seed ^ u64::from(day)).to_le_bytes()[0] & 1)]
}

#[must_use]
pub fn description(id: &str) -> String {
    format!("{}: {}. {}", i18n::t("road_ad.label"),
        i18n::t(&format!("road_ad.{id}.headline")),
        i18n::t(&format!("road_ad.{id}.copy")))
}

pub fn render(id: &str, day: u32) -> Html {
    html! {<div class={classes!("road-billboard", if day.is_multiple_of(2) {"billboard-steel"} else {"billboard-timber"})}
        data-billboard={id.to_owned()} data-art-version="2">
        <div class="billboard-panel"><span class="billboard-label">{i18n::t("road_ad.label")}</span>
            <strong>{i18n::t(&format!("road_ad.{id}.headline"))}</strong>
            <span class="billboard-copy">{i18n::t(&format!("road_ad.{id}.copy"))}</span>
        </div><span class="billboard-post post-left"/><span class="billboard-post post-right"/>
    </div>}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signs_repeat_exactly_for_replay_and_offer_two_per_region() {
        let regions=[Region::PacificCoast,Region::MountainWest,Region::Southwest,
            Region::Heartland,Region::RustBelt,Region::Beltway];
        let mut all=std::collections::BTreeSet::new();
        for region in regions {
            let first=selected(region,42,1);
            assert_eq!(first,selected(region,42,1));
            assert_ne!(first,selected(region,42,2));
            all.insert(first);all.insert(selected(region,42,2));
        }
        assert_eq!(all.len(),12);
    }
}
