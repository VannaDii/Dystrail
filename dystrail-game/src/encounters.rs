//! Encounter selection logic
use crate::data::{Encounter, EncounterData};
use crate::state::Region;
use rand::Rng;

#[cfg(debug_assertions)]
fn debug_log_enabled() -> bool {
    matches!(std::env::var("DYSTRAIL_DEBUG_LOGS"), Ok(val) if val != "0")
}

#[cfg(not(debug_assertions))]
const fn debug_log_enabled() -> bool {
    false
}

pub fn pick_encounter<R: Rng>(
    region: Region,
    is_deep: bool,
    malnutrition_level: u32,
    starving: bool,
    data: &EncounterData,
    rng: &mut R,
) -> Option<Encounter> {
    let region_str = match region {
        Region::Heartland => "heartland",
        Region::RustBelt => "rustbelt",
        Region::Beltway => "beltway",
    };
    let mode_aliases: &[&str] = if is_deep {
        &["deep", "deep_end"]
    } else {
        &["classic"]
    };

    // Filter encounters by region and mode
    let candidates: Vec<&Encounter> = data
        .encounters
        .iter()
        .filter(|e| {
            let region_match = e.regions.is_empty()
                || e.regions.iter().any(|r| r.eq_ignore_ascii_case(region_str));
            let mode_match = e.modes.is_empty()
                || e.modes.iter().any(|m| {
                    mode_aliases
                        .iter()
                        .any(|alias| m.eq_ignore_ascii_case(alias))
                });
            region_match && mode_match
        })
        .collect();

    if debug_log_enabled() {
        println!(
            "Encounter selection | mode:{} region:{} candidates:{}",
            if is_deep { "Deep" } else { "Classic" },
            region_str,
            candidates.len()
        );
    }

    if candidates.is_empty() {
        return None;
    }

    // Calculate total weight with starvation adjustments
    let starvation_bonus = if starving || malnutrition_level > 0 {
        10 + (malnutrition_level * 5)
    } else {
        0
    };

    let weighted: Vec<(usize, u32)> = candidates
        .iter()
        .enumerate()
        .map(|(idx, enc)| {
            let mut weight = enc.weight;
            if starvation_bonus > 0 && is_forage(enc) {
                weight = weight.saturating_add(starvation_bonus);
            }
            (idx, weight.max(1))
        })
        .collect();

    let total_weight: u32 = weighted.iter().map(|(_, weight)| *weight).sum();
    if total_weight == 0 {
        return None;
    }

    // Random selection based on weight
    let roll = rng.random_range(0..total_weight);
    let mut current_weight = 0;

    for (idx, weight) in &weighted {
        current_weight += *weight;
        if roll < current_weight {
            return Some(candidates[*idx].clone());
        }
    }

    // Fallback to first candidate
    candidates.first().map(|e| (*e).clone())
}

fn is_forage(encounter: &Encounter) -> bool {
    let id_lower = encounter.id.to_lowercase();
    let name_lower = encounter.name.to_lowercase();
    id_lower.contains("forage")
        || name_lower.contains("forage")
        || name_lower.contains("scavenge")
        || name_lower.contains("gather")
}
