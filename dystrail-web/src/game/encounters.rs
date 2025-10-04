use rand::Rng;
use rand_chacha::ChaCha20Rng;

use super::data::{Encounter, EncounterData};
use super::state::Region;

/// Filter encounters by current mode and region, then choose weighted random.
pub fn pick_encounter(
    data: &EncounterData,
    is_deep: bool,
    region: Region,
    rng: &mut ChaCha20Rng,
) -> Option<Encounter> {
    let region_name = match region {
        Region::Heartland => "Heartland",
        Region::RustBelt => "RustBelt",
        Region::Beltway => "Beltway",
    };
    let mode_name = if is_deep { "deep_end" } else { "classic" };
    let pool: Vec<&Encounter> = data
        .encounters
        .iter()
        .filter(|e| {
            (e.regions.is_empty() || e.regions.iter().any(|r| r == region_name))
                && (e.modes.is_empty() || e.modes.iter().any(|m| m == mode_name))
        })
        .collect();
    if pool.is_empty() {
        return None;
    }
    let total_weight: u32 = pool.iter().map(|e| e.weight).sum();
    if total_weight == 0 {
        return None;
    }
    let mut roll = rng.random::<u32>() % total_weight;
    for e in pool {
        if roll < e.weight {
            return Some(e.clone());
        }
        roll -= e.weight;
    }
    None
}
