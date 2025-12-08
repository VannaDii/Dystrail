//! Encounter selection logic
#[cfg(debug_assertions)]
use crate::constants::DEBUG_ENV_VAR;
use crate::constants::{ENCOUNTER_REPEAT_WINDOW_DAYS, ROTATION_LOOKBACK_DAYS};
use crate::data::{Encounter, EncounterData};
use crate::state::{PolicyKind, RecentEncounter, Region};
use rand::Rng;
use std::collections::{HashMap, VecDeque};

#[cfg(debug_assertions)]
fn debug_log_enabled() -> bool {
    matches!(std::env::var(DEBUG_ENV_VAR), Ok(val) if val != "0")
}

#[cfg(not(debug_assertions))]
const fn debug_log_enabled() -> bool {
    false
}

pub struct EncounterRequest<'a> {
    pub region: Region,
    pub is_deep: bool,
    pub malnutrition_level: u32,
    pub starving: bool,
    pub data: &'a EncounterData,
    pub recent: &'a [RecentEncounter],
    pub current_day: u32,
    pub policy: Option<PolicyKind>,
    pub force_rotation: bool,
}

pub fn pick_encounter<R: Rng>(
    request: &EncounterRequest<'_>,
    rotation_queue: &mut VecDeque<String>,
    rng: &mut R,
) -> (Option<Encounter>, bool) {
    let candidates = filter_candidates(request);

    if debug_log_enabled() {
        println!(
            "Encounter selection | mode:{} region:{} candidates:{}",
            if request.is_deep { "Deep" } else { "Classic" },
            request.region.asset_key(),
            candidates.len()
        );
    }

    if candidates.is_empty() {
        return (None, false);
    }

    let last_seen = build_last_seen_map(request.recent);
    if rotation_queue.is_empty() {
        *rotation_queue = build_rotation_backlog(request, &candidates, &last_seen);
    }

    if !rotation_queue.is_empty() {
        let queue_len = rotation_queue.len();
        for _ in 0..queue_len {
            let Some(next_id) = rotation_queue.pop_front() else {
                break;
            };
            if let Some((_, encounter)) = candidates
                .iter()
                .enumerate()
                .find(|(_, enc)| enc.id == next_id)
            {
                let ready_for_rotation = last_seen
                    .get(encounter.id.as_str())
                    .map(|day| request.current_day.saturating_sub(*day))
                    .is_none_or(|age| age >= ENCOUNTER_REPEAT_WINDOW_DAYS);
                if ready_for_rotation {
                    if debug_log_enabled() {
                        println!(
                            "Encounter rotation | day {} queued {}",
                            request.current_day, encounter.id
                        );
                    }
                    return (Some((*encounter).clone()), true);
                }
            }
            rotation_queue.push_back(next_id);
        }
    }

    if request.force_rotation {
        if rotation_queue.is_empty() {
            *rotation_queue = build_rotation_backlog(request, &candidates, &last_seen);
        }
        while let Some(next_id) = rotation_queue.pop_front() {
            if let Some((_, encounter)) = candidates
                .iter()
                .enumerate()
                .find(|(_, enc)| enc.id == next_id)
            {
                if debug_log_enabled() {
                    println!(
                        "Encounter rotation | day {} forced {}",
                        request.current_day, encounter.id
                    );
                }
                return (Some((*encounter).clone()), true);
            }
        }
    }
    let (primary, fallback) = categorize_candidates(request, &candidates, &last_seen);
    let (selection, rotation_satisfied) =
        determine_selection(primary, fallback, request.force_rotation, candidates.len());

    if selection.is_empty() {
        return (None, rotation_satisfied);
    }

    let region_counts =
        if request.is_deep && matches!(request.policy, Some(PolicyKind::Conservative)) {
            Some(build_recent_region_counts(request.recent))
        } else {
            None
        };
    let region_min = region_counts.as_ref().map_or(0, global_min_region_count);

    let weighted = build_weights(
        selection,
        &candidates,
        request,
        &last_seen,
        region_counts.as_ref(),
        region_min,
    );
    let Some(chosen_idx) = choose_weighted(&weighted, rng) else {
        return (None, rotation_satisfied);
    };

    (
        candidates
            .get(chosen_idx)
            .map(|encounter| (*encounter).clone()),
        rotation_satisfied,
    )
}

fn is_forage(encounter: &Encounter) -> bool {
    let id_lower = encounter.id.to_lowercase();
    let name_lower = encounter.name.to_lowercase();
    id_lower.contains("forage")
        || name_lower.contains("forage")
        || name_lower.contains("scavenge")
        || name_lower.contains("gather")
}

fn filter_candidates<'a>(request: &EncounterRequest<'a>) -> Vec<&'a Encounter> {
    let region_str = match request.region {
        Region::Heartland => "heartland",
        Region::RustBelt => "rustbelt",
        Region::Beltway => "beltway",
    };
    let mode_aliases: &[&str] = if request.is_deep {
        &["deep", "deep_end"]
    } else {
        &["classic"]
    };

    request
        .data
        .encounters
        .iter()
        .filter(|encounter| {
            let region_match = encounter.regions.is_empty()
                || encounter
                    .regions
                    .iter()
                    .any(|region| region.eq_ignore_ascii_case(region_str));
            let mode_match = encounter.modes.is_empty()
                || encounter.modes.iter().any(|mode| {
                    mode_aliases
                        .iter()
                        .any(|alias| mode.eq_ignore_ascii_case(alias))
                });
            region_match && mode_match
        })
        .collect()
}

fn build_last_seen_map(recent: &[RecentEncounter]) -> HashMap<&str, u32> {
    let mut last_seen: HashMap<&str, u32> = HashMap::new();
    for entry in recent {
        let value = last_seen.entry(entry.id.as_str()).or_insert(entry.day);
        *value = (*value).max(entry.day);
    }
    last_seen
}

fn categorize_candidates<'a>(
    request: &EncounterRequest<'a>,
    candidates: &[&'a Encounter],
    last_seen: &HashMap<&str, u32>,
) -> (Vec<usize>, Vec<usize>) {
    let mut primary = Vec::new();
    let mut fallback = Vec::new();
    for (idx, encounter) in candidates.iter().enumerate() {
        let chainable = encounter.chainable;
        let last_day = last_seen.get(encounter.id.as_str()).copied();
        let age = last_day.map(|day| request.current_day.saturating_sub(day));
        let passes_no_repeat =
            chainable || age.is_none_or(|days| days >= ENCOUNTER_REPEAT_WINDOW_DAYS);
        let meets_rotation = chainable || age.is_none_or(|days| days >= ROTATION_LOOKBACK_DAYS);

        if request.force_rotation {
            if meets_rotation {
                primary.push(idx);
            } else {
                fallback.push(idx);
            }
        } else if passes_no_repeat {
            primary.push(idx);
        } else {
            fallback.push(idx);
        }
    }
    (primary, fallback)
}

fn determine_selection(
    primary: Vec<usize>,
    fallback: Vec<usize>,
    force_rotation: bool,
    total_candidates: usize,
) -> (Vec<usize>, bool) {
    let rotation_satisfied = force_rotation && !primary.is_empty();

    let mut selection = if primary.is_empty() {
        fallback
    } else {
        primary
    };

    if selection.is_empty() {
        selection = (0..total_candidates).collect();
    }

    (selection, rotation_satisfied)
}

fn build_weights(
    selection: Vec<usize>,
    candidates: &[&Encounter],
    request: &EncounterRequest<'_>,
    last_seen: &HashMap<&str, u32>,
    region_counts: Option<&HashMap<Region, u32>>,
    region_min: u32,
) -> Vec<(usize, u32)> {
    let mut weighted = Vec::with_capacity(selection.len());
    let starvation_bonus = if request.starving || request.malnutrition_level > 0 {
        10 + (request.malnutrition_level * 5)
    } else {
        0
    };

    for idx in selection {
        let encounter = candidates[idx];
        let mut weight = encounter.weight.max(1);
        if starvation_bonus > 0 && is_forage(encounter) {
            weight = weight.saturating_add(starvation_bonus);
        }
        if encounter.chainable {
            weight = weight.saturating_sub(2);
        }
        if let Some(last_day) = last_seen.get(encounter.id.as_str()) {
            let age = request.current_day.saturating_sub(*last_day).max(1);
            weight = weight.saturating_add(age.saturating_mul(2));
        } else {
            weight = weight
                .saturating_add(ROTATION_LOOKBACK_DAYS.max(1))
                .saturating_add(12);
        }
        if let Some(counts) = region_counts
            && request.is_deep
            && matches!(request.policy, Some(PolicyKind::Conservative))
        {
            let regions = encounter_regions(encounter, request.region);
            if !regions.is_empty() {
                let encounter_min = regions
                    .iter()
                    .map(|region| counts.get(region).copied().unwrap_or(0))
                    .min()
                    .unwrap_or(0);
                if encounter_min <= region_min {
                    let boosted = weight.saturating_mul(110).saturating_add(99) / 100;
                    weight = boosted.max(weight.saturating_add(1));
                }
            }
        }
        weighted.push((idx, weight.max(1)));
    }

    weighted
}

fn choose_weighted<R: Rng>(weights: &[(usize, u32)], rng: &mut R) -> Option<usize> {
    let total_weight: u32 = weights.iter().map(|(_, weight)| *weight).sum();
    if total_weight == 0 {
        return None;
    }

    let roll = rng.gen_range(0..total_weight);
    let mut current = 0;
    for (idx, weight) in weights {
        current += *weight;
        if roll < current {
            return Some(*idx);
        }
    }

    weights.first().map(|(idx, _)| *idx)
}

fn build_rotation_backlog<'a>(
    request: &EncounterRequest<'a>,
    candidates: &[&'a Encounter],
    last_seen: &HashMap<&str, u32>,
) -> VecDeque<String> {
    let mut ready: Vec<(String, bool, u32)> = Vec::new();
    let mut pending: Vec<(String, bool, u32)> = Vec::new();
    for encounter in candidates {
        if encounter.chainable {
            continue;
        }
        let last_day = last_seen.get(encounter.id.as_str()).copied();
        let age = last_day.map_or(u32::MAX, |day| request.current_day.saturating_sub(day));
        let rotation_ready = last_day.is_none() || age >= ROTATION_LOOKBACK_DAYS;
        let target = if rotation_ready {
            &mut ready
        } else {
            &mut pending
        };
        target.push((encounter.id.clone(), encounter.chainable, age));
    }

    if !pending.is_empty() {
        pending.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
        ready.extend(pending);
    }

    if ready.is_empty() {
        return VecDeque::new();
    }

    ready.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| b.2.cmp(&a.2))
            .then_with(|| a.0.cmp(&b.0))
    });

    ready.into_iter().map(|(id, _, _)| id).collect()
}

fn build_recent_region_counts(recent: &[RecentEncounter]) -> HashMap<Region, u32> {
    let mut counts: HashMap<Region, u32> = HashMap::new();
    for entry in recent {
        if let Some(region) = entry.region {
            *counts.entry(region).or_default() += 1;
        }
    }
    counts
}

fn global_min_region_count(counts: &HashMap<Region, u32>) -> u32 {
    [Region::Heartland, Region::RustBelt, Region::Beltway]
        .iter()
        .map(|region| counts.get(region).copied().unwrap_or(0))
        .min()
        .unwrap_or(0)
}

const fn parse_region(label: &str) -> Option<Region> {
    if label.eq_ignore_ascii_case("heartland") {
        Some(Region::Heartland)
    } else if label.eq_ignore_ascii_case("rustbelt") {
        Some(Region::RustBelt)
    } else if label.eq_ignore_ascii_case("beltway") {
        Some(Region::Beltway)
    } else {
        None
    }
}

fn encounter_regions(encounter: &Encounter, fallback: Region) -> Vec<Region> {
    let mut regions: Vec<Region> = encounter
        .regions
        .iter()
        .filter_map(|label| parse_region(label))
        .collect();
    if regions.is_empty() {
        regions.push(fallback);
    }
    regions
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{RecentEncounter, Region};
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use std::collections::VecDeque;

    fn make_enc(id: &str, regions: &[&str]) -> Encounter {
        Encounter {
            id: id.to_string(),
            name: format!("Encounter {id}"),
            desc: String::new(),
            weight: 3,
            regions: regions.iter().map(|r| (*r).to_string()).collect(),
            modes: vec!["classic".to_string()],
            choices: Vec::new(),
            hard_stop: false,
            major_repair: false,
            chainable: false,
        }
    }

    fn sample_encounters() -> EncounterData {
        EncounterData::from_encounters(vec![
            make_enc("alpha", &["Heartland"]),
            make_enc("beta", &["Heartland", "RustBelt"]),
            make_enc("gamma", &[]),
        ])
    }

    fn mk_request(data: &EncounterData) -> EncounterRequest<'_> {
        EncounterRequest {
            region: Region::Heartland,
            is_deep: true,
            malnutrition_level: 2,
            starving: false,
            data,
            recent: &[],
            current_day: 12,
            policy: Some(PolicyKind::Conservative),
            force_rotation: false,
        }
    }

    #[test]
    fn rotation_backlog_skips_recent_encounters() {
        let data = EncounterData::from_encounters(vec![
            make_enc("enc_a", &["Heartland"]),
            make_enc("enc_b", &["Heartland"]),
            make_enc("enc_c", &["Heartland"]),
            make_enc("enc_d", &["Heartland"]),
            make_enc("enc_e", &["Heartland"]),
        ]);
        let recent = vec![
            RecentEncounter::new("enc_a".to_string(), 17, Region::Heartland),
            RecentEncounter::new("enc_b".to_string(), 15, Region::Heartland),
            RecentEncounter::new("enc_e".to_string(), 19, Region::Heartland),
        ];
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &recent,
            current_day: 20,
            policy: None,
            force_rotation: true,
        };

        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(request.recent);
        let backlog = build_rotation_backlog(&request, &candidates, &last_seen);

        let mut ids: Vec<_> = backlog.into_iter().collect();
        ids.sort();
        assert_eq!(ids, vec!["enc_a", "enc_b", "enc_c", "enc_d", "enc_e"]);
    }

    #[test]
    fn pick_encounter_respects_rotation_queue() {
        let data = sample_encounters();
        let mut queue = VecDeque::new();
        let mut request = mk_request(&data);
        request.force_rotation = true;
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        let _ = pick_encounter(&request, &mut queue, &mut rng);
    }

    #[test]
    fn determine_selection_falls_back() {
        let (primary, satisfied) = determine_selection(vec![], vec![1, 2, 3], false, 4);
        assert_eq!(primary, vec![1, 2, 3]);
        assert!(!satisfied);
        let (primary_force, satisfied) = determine_selection(vec![], vec![], true, 2);
        assert_eq!(primary_force, vec![0, 1]);
        assert!(!satisfied);
    }

    #[test]
    fn weighted_choice_prefers_higher_weight() {
        let mut rng = ChaCha20Rng::from_seed([1u8; 32]);
        let weights = vec![(0, 1), (1, 50)];
        let pick = choose_weighted(&weights, &mut rng);
        assert_eq!(pick, Some(1));
    }
}
