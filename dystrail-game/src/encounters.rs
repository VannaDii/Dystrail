//! Encounter selection logic
#[cfg(debug_assertions)]
use crate::constants::DEBUG_ENV_VAR;
use crate::constants::{ENCOUNTER_REPEAT_WINDOW_DAYS, ROTATION_LOOKBACK_DAYS};
use crate::data::{Encounter, EncounterData};
use crate::journey::event::{EventDecisionTrace, RollValue, WeightFactor, WeightedCandidate};
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

#[derive(Debug, Clone, PartialEq)]
pub struct EncounterPick {
    pub encounter: Option<Encounter>,
    pub rotation_satisfied: bool,
    pub decision_trace: Option<EventDecisionTrace>,
}

const fn pick_none(rotation_satisfied: bool) -> EncounterPick {
    EncounterPick {
        encounter: None,
        rotation_satisfied,
        decision_trace: None,
    }
}

fn find_candidate<'a>(candidates: &[&'a Encounter], encounter_id: &str) -> Option<&'a Encounter> {
    candidates
        .iter()
        .copied()
        .find(|encounter| encounter.id == encounter_id)
}

fn rotation_ready(current_day: u32, last_seen: &HashMap<&str, u32>, encounter_id: &str) -> bool {
    last_seen
        .get(encounter_id)
        .map(|day| current_day.saturating_sub(*day))
        .is_none_or(|age| age >= ENCOUNTER_REPEAT_WINDOW_DAYS)
}

fn try_pick_ready_from_rotation_queue(
    request: &EncounterRequest<'_>,
    rotation_queue: &mut VecDeque<String>,
    candidates: &[&Encounter],
    last_seen: &HashMap<&str, u32>,
) -> Option<Encounter> {
    if rotation_queue.is_empty() {
        return None;
    }
    let queue_len = rotation_queue.len();
    for _ in 0..queue_len {
        let Some(next_id) = rotation_queue.pop_front() else {
            break;
        };
        if let Some(encounter) = find_candidate(candidates, &next_id)
            && rotation_ready(request.current_day, last_seen, encounter.id.as_str())
        {
            if debug_log_enabled() {
                println!(
                    "Encounter rotation | day {} queued {}",
                    request.current_day, encounter.id
                );
            }
            return Some(encounter.clone());
        }
        rotation_queue.push_back(next_id);
    }
    None
}

fn try_pick_forced_from_rotation_queue(
    request: &EncounterRequest<'_>,
    rotation_queue: &mut VecDeque<String>,
    candidates: &[&Encounter],
    last_seen: &HashMap<&str, u32>,
) -> Option<Encounter> {
    if !request.force_rotation {
        return None;
    }
    if rotation_queue.is_empty() {
        *rotation_queue = build_rotation_backlog(request, candidates, last_seen);
    }
    while let Some(next_id) = rotation_queue.pop_front() {
        if let Some(encounter) = find_candidate(candidates, &next_id) {
            if debug_log_enabled() {
                println!(
                    "Encounter rotation | day {} forced {}",
                    request.current_day, encounter.id
                );
            }
            return Some(encounter.clone());
        }
    }
    None
}

pub fn pick_encounter<R: Rng>(
    request: &EncounterRequest<'_>,
    rotation_queue: &mut VecDeque<String>,
    rng: &mut R,
) -> EncounterPick {
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
        return pick_none(false);
    }

    let last_seen = build_last_seen_map(request.recent);
    if rotation_queue.is_empty() {
        *rotation_queue = build_rotation_backlog(request, &candidates, &last_seen);
    }

    if let Some(encounter) =
        try_pick_ready_from_rotation_queue(request, rotation_queue, &candidates, &last_seen)
    {
        return EncounterPick {
            encounter: Some(encounter),
            rotation_satisfied: true,
            decision_trace: None,
        };
    }

    if let Some(encounter) =
        try_pick_forced_from_rotation_queue(request, rotation_queue, &candidates, &last_seen)
    {
        return EncounterPick {
            encounter: Some(encounter),
            rotation_satisfied: true,
            decision_trace: None,
        };
    }

    let (primary, fallback) = categorize_candidates(request, &candidates, &last_seen);
    let (selection, rotation_satisfied) =
        determine_selection(primary, fallback, request.force_rotation, candidates.len());

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
    let Some((chosen_idx, roll)) = choose_weighted(&weighted, rng) else {
        return pick_none(rotation_satisfied);
    };

    let chosen_ref = candidates.get(chosen_idx).copied();
    let chosen = chosen_ref.map(|encounter| (*encounter).clone());

    let decision_trace =
        chosen_ref.map(|encounter| build_decision_trace(&candidates, &weighted, roll, encounter));

    EncounterPick {
        encounter: chosen,
        rotation_satisfied,
        decision_trace,
    }
}

fn build_decision_trace(
    candidates: &[&Encounter],
    weights: &[(usize, u32)],
    roll: u32,
    chosen: &Encounter,
) -> EventDecisionTrace {
    const POOL_ID: &str = "dystrail.encounter";

    let weighted_candidates = weights
        .iter()
        .filter_map(|(idx, final_weight)| {
            let encounter = *candidates.get(*idx)?;
            let base_weight = f64::from(encounter.weight.max(1));
            let final_weight_f = f64::from(*final_weight);
            let multiplier = if base_weight > 0.0 {
                final_weight_f / base_weight
            } else {
                1.0
            };
            Some(WeightedCandidate {
                id: encounter.id.clone(),
                base_weight,
                multipliers: vec![WeightFactor {
                    label: String::from("effective"),
                    value: multiplier,
                }],
                final_weight: final_weight_f,
            })
        })
        .collect();

    EventDecisionTrace {
        pool_id: String::from(POOL_ID),
        roll: RollValue::U32(roll),
        candidates: weighted_candidates,
        chosen_id: chosen.id.clone(),
    }
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

fn choose_weighted<R: Rng>(weights: &[(usize, u32)], rng: &mut R) -> Option<(usize, u32)> {
    let total_weight: u32 = weights.iter().map(|(_, weight)| *weight).sum();
    if total_weight == 0 {
        return None;
    }

    let roll = rng.gen_range(0..total_weight);
    let mut current = 0;
    for (idx, weight) in weights {
        current += *weight;
        if roll < current {
            return Some((*idx, roll));
        }
    }

    weights.first().map(|(idx, _)| (*idx, roll))
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
    use crate::constants::FLOAT_EPSILON;
    use crate::state::{RecentEncounter, Region};
    use rand::Rng;
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
    fn weighted_selection_emits_decision_trace() {
        let data = sample_encounters();
        let current_day = 20;
        let recent = vec![
            RecentEncounter::new(String::from("alpha"), current_day - 1, Region::Heartland),
            RecentEncounter::new(String::from("beta"), current_day - 1, Region::Heartland),
            RecentEncounter::new(String::from("gamma"), current_day - 1, Region::Heartland),
        ];
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &recent,
            current_day,
            policy: None,
            force_rotation: false,
        };

        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(&recent);
        let (primary, fallback) = categorize_candidates(&request, &candidates, &last_seen);
        let (selection, rotation_satisfied) =
            determine_selection(primary, fallback, request.force_rotation, candidates.len());
        assert!(!rotation_satisfied, "no forced rotation in this scenario");
        let weighted = build_weights(selection, &candidates, &request, &last_seen, None, 0);
        let total_weight: u32 = weighted.iter().map(|(_, weight)| *weight).sum();
        assert!(total_weight > 0, "expected non-empty weighted pool");

        let mut expected_rng = ChaCha20Rng::from_seed([1u8; 32]);
        let expected_roll = expected_rng.gen_range(0..total_weight);
        let mut expected_choice = weighted
            .first()
            .map(|(idx, _)| *idx)
            .expect("weighted pool has entries");
        let mut current = 0_u32;
        for (idx, weight) in &weighted {
            current = current.saturating_add(*weight);
            if expected_roll < current {
                expected_choice = *idx;
                break;
            }
        }

        let mut queue = VecDeque::new();
        let mut rng = ChaCha20Rng::from_seed([1u8; 32]);
        let pick = pick_encounter(&request, &mut queue, &mut rng);
        assert_eq!(pick.rotation_satisfied, rotation_satisfied);

        let encounter = pick.encounter.expect("encounter selected");
        assert_eq!(encounter.id, candidates[expected_choice].id);

        let trace = pick.decision_trace.expect("decision trace recorded");
        assert_eq!(trace.pool_id, "dystrail.encounter");
        assert_eq!(trace.roll, RollValue::U32(expected_roll));
        assert_eq!(trace.chosen_id, encounter.id);
        assert_eq!(trace.candidates.len(), weighted.len());

        for (candidate_trace, (idx, weight)) in trace.candidates.iter().zip(weighted.iter()) {
            let encounter = candidates[*idx];
            let base_weight = f64::from(encounter.weight.max(1));
            let final_weight = f64::from(*weight);
            assert_eq!(candidate_trace.id, encounter.id);
            assert!(
                (candidate_trace.base_weight - base_weight).abs() < FLOAT_EPSILON,
                "expected base weight {base_weight} but got {}",
                candidate_trace.base_weight
            );
            assert!(
                (candidate_trace.final_weight - final_weight).abs() < FLOAT_EPSILON,
                "expected final weight {final_weight} but got {}",
                candidate_trace.final_weight
            );
            assert_eq!(candidate_trace.multipliers.len(), 1);
            assert_eq!(candidate_trace.multipliers[0].label, "effective");
            let expected_multiplier = final_weight / base_weight;
            let actual_multiplier = candidate_trace.multipliers[0].value;
            assert!(
                (actual_multiplier - expected_multiplier).abs() < FLOAT_EPSILON,
                "expected multiplier {expected_multiplier} but got {actual_multiplier}"
            );
        }
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
        let pick = choose_weighted(&weights, &mut rng).map(|(idx, _)| idx);
        assert_eq!(pick, Some(1));
    }
}
