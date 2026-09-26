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
    // Keep the regional pair stable through a simulated day and across replay.
    // The seed and day are already part of the save and bare-code replay contract.
    pair[usize::from((seed ^ u64::from(day)).to_le_bytes()[0] & 1)]
}

#[must_use]
pub fn description(id: &str) -> String {
    format!(
        "{}: {}. {}",
        i18n::t("road_ad.label"),
        i18n::t(&format!("road_ad.{id}.headline")),
        i18n::t(&format!("road_ad.{id}.copy"))
    )
}

#[must_use]
pub fn companion(region: Region, seed: u64, day: u32) -> &'static str {
    selected(region, seed, day.wrapping_add(1))
}

fn illustration(id: &str) -> &'static str {
    match id {
        "clean_air" => "masks-v1",
        "public_land" => "distance-v1",
        "energy" => "battery-v1",
        "weather" => "ponchos-v1",
        "water" => "water-v1",
        "wellness" => "rations-v1",
        "tariffs" => "cash-v1",
        "farm" => "rations-v1",
        "jobs" => "coats-v1",
        "repair" => "alternator-v1",
        "access" => "distance-v1",
        "transparency" => "masks-v1",
        _ => "rations-v1",
    }
}

fn illustration_path(id: &str) -> String {
    let folder = if matches!(id, "tariffs" | "public_land" | "access") { "status" } else { "items" };
    crate::paths::asset_path(&format!("static/img/{folder}/{}.png", illustration(id)))
}

fn sign(id: &str, day: u32, slot: &str, duplicate: bool) -> Html {
    html! {<div class={classes!("road-billboard", if day.is_multiple_of(2) {"billboard-steel"} else {"billboard-timber"})}
        style={format!("--billboard-slot:{slot}%")}
        data-billboard={id.to_owned()} data-sign-slot={slot.to_owned()} data-repeat={duplicate.to_string()}>
        <div class="billboard-panel"><span class="billboard-art" data-art={id.to_owned()}>
            <img src={illustration_path(id)} alt="" decoding="async" />
        </span><span class="billboard-words"><span class="billboard-label">{i18n::t("road_ad.label")}</span>
            <strong class="billboard-headline-full">{i18n::t(&format!("road_ad.{id}.headline"))}</strong>
            <strong class="billboard-headline-compact">{i18n::t(&format!("road_ad.{id}.compact"))}</strong>
            <span class="billboard-copy">{i18n::t(&format!("road_ad.{id}.copy"))}</span>
        </span></div><span class="billboard-post post-left"/><span class="billboard-post post-right"/>
    </div>}
}

pub fn render_pair(region: Region, seed: u64, day: u32) -> Html {
    let first = selected(region, seed, day);
    let second = companion(region, seed, day);
    html! {<>
        {sign(first, day, "32.5", false)}
        {sign(second, day, "57.5", false)}
        {sign(first, day, "82.5", true)}
        {sign(second, day, "107.5", true)}
    </>}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signs_repeat_exactly_for_replay_and_offer_two_per_region() {
        let regions = [
            Region::PacificCoast,
            Region::MountainWest,
            Region::Southwest,
            Region::Heartland,
            Region::RustBelt,
            Region::Beltway,
        ];
        let mut all = std::collections::BTreeSet::new();
        for region in regions {
            let first = selected(region, 42, 1);
            assert_eq!(first, selected(region, 42, 1));
            assert_ne!(first, companion(region, 42, 1));
            all.insert(first);
            all.insert(selected(region, 42, 2));
        }
        assert_eq!(all.len(), 12);
    }
}
