//! Encounter selection logic
use crate::data::{Encounter, EncounterData};
use crate::state::Region;
use rand::Rng;

pub fn pick_encounter<R: Rng>(
    data: &EncounterData,
    is_deep: bool,
    region: Region,
    rng: &mut R,
) -> Option<Encounter> {
    let region_str = match region {
        Region::Heartland => "heartland",
        Region::RustBelt => "rustbelt",
        Region::Beltway => "beltway",
    };

    let mode_str = if is_deep { "deep" } else { "classic" };

    // Filter encounters by region and mode
    let candidates: Vec<&Encounter> = data
        .encounters
        .iter()
        .filter(|e| {
            let region_match = e.regions.is_empty() || e.regions.contains(&region_str.to_string());
            let mode_match = e.modes.is_empty() || e.modes.contains(&mode_str.to_string());
            region_match && mode_match
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Calculate total weight
    let total_weight: u32 = candidates.iter().map(|e| e.weight).sum();
    if total_weight == 0 {
        return None;
    }

    // Random selection based on weight
    let roll = rng.random_range(0..total_weight);
    let mut current_weight = 0;

    for encounter in &candidates {
        current_weight += encounter.weight;
        if roll < current_weight {
            return Some((*encounter).clone());
        }
    }

    // Fallback to first candidate
    candidates.first().map(|e| (*e).clone())
}
