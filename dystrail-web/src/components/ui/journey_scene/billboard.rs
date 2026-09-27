//! Localized roadside advertising. This presentation selection never touches simulation RNG.
use crate::{game::Region, i18n};
use yew::prelude::*;

const ADS: [&str; 12] = [
    "clean_air", "public_land", "energy", "weather", "water", "wellness",
    "tariffs", "farm", "jobs", "repair", "access", "transparency",
];

#[must_use]
pub fn selected(_region: Region, seed: u64, step: u32) -> &'static str {
    // Odd strides coprime with twelve visit the full pool before repeating.
    let stride = [1, 5, 7, 11][usize::from(seed.to_le_bytes()[1] & 3)];
    let start = usize::from(seed.to_le_bytes()[0]) % ADS.len();
    ADS[(start + stride * (step as usize % ADS.len())) % ADS.len()]
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

fn illustration(id: &str) -> (&'static str, &'static str) {
    match id {
        "clean_air" => ("items", "masks-v1"),
        "public_land" => ("status", "distance-v1"),
        "energy" => ("items", "battery-v1"),
        "weather" => ("items", "ponchos-v1"),
        "water" => ("items", "water-v1"),
        "wellness" => ("items", "rations-v1"),
        "tariffs" => ("status", "cash-v1"),
        "farm" => ("items", "fuel_pump-v1"),
        "jobs" => ("items", "coats-v1"),
        "repair" => ("items", "alternator-v1"),
        "access" => ("items", "press_pass-v1"),
        "transparency" => ("items", "legal_fund-v1"),
        _ => ("items", "rations-v1"),
    }
}

fn illustration_path(id: &str) -> String {
    let (folder, name) = illustration(id);
    crate::paths::asset_path(&format!("static/img/{folder}/{name}.png"))
}

fn sign(id: &str, slot: &str) -> Html {
    let frame = if id.as_bytes().last().is_some_and(|byte| byte & 1 == 0) {"billboard-steel"} else {"billboard-timber"};
    html! {<div class={classes!("road-billboard",frame)}
        style={format!("--billboard-slot:{slot}%")}
        data-billboard={id.to_owned()} data-sign-slot={slot.to_owned()}>
        <div class="billboard-panel"><span class="billboard-art" data-art={id.to_owned()}>
            <img src={illustration_path(id)} alt="" decoding="async" />
        </span><span class="billboard-words"><span class="billboard-label">{i18n::t("road_ad.label")}</span>
            <strong class="billboard-headline-full">{i18n::t(&format!("road_ad.{id}.headline"))}</strong>
            <strong class="billboard-headline-compact">{i18n::t(&format!("road_ad.{id}.compact"))}</strong>
            <span class="billboard-copy">{i18n::t(&format!("road_ad.{id}.copy"))}</span>
        </span></div><span class="billboard-post post-left"/><span class="billboard-post post-right"/>
    </div>}
}

pub fn render(region: Region, seed: u64, step: u32) -> Html {
    let first = selected(region, seed, step);
    sign(first, "35")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signs_repeat_exactly_for_replay_and_cycle_all_twelve_before_repeating() {
        let regions = [
            Region::PacificCoast,
            Region::MountainWest,
            Region::Southwest,
            Region::Heartland,
            Region::RustBelt,
            Region::Beltway,
        ];
        for region in regions {
            for seed in [0, 42, 81727] {
                let all = (0..12).map(|step| selected(region, seed, step)).collect::<std::collections::BTreeSet<_>>();
                assert_eq!(all.len(), 12);
                assert_eq!(selected(region, seed, 0), selected(region, seed, 12));
                assert_ne!(selected(region, seed, 0), selected(region, seed, 1));
            }
        }
        assert_eq!(ADS.iter().map(|id| illustration(id)).collect::<std::collections::BTreeSet<_>>().len(), ADS.len());
    }
}
