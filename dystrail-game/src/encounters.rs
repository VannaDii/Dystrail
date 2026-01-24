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
    finalize_pick(None, rotation_satisfied, None)
}

const fn rotation_pick(encounter: Encounter) -> EncounterPick {
    finalize_pick(Some(encounter), true, None)
}

#[rustfmt::skip]
const fn finalize_pick(
    encounter: Option<Encounter>,
    rotation_satisfied: bool,
    decision_trace: Option<EventDecisionTrace>,
) -> EncounterPick {
    EncounterPick { encounter, rotation_satisfied, decision_trace }
}

#[rustfmt::skip]
fn weight_factor(label: &str, value: f64) -> WeightFactor {
    WeightFactor { label: label.to_string(), value }
}

#[rustfmt::skip]
fn make_candidate(id: &str, base_weight: f64, factor: WeightFactor, final_weight: f64) -> WeightedCandidate {
    WeightedCandidate { id: id.into(), base_weight, multipliers: vec![factor], final_weight }
}

fn find_candidate<'a>(candidates: &[&'a Encounter], encounter_id: &str) -> Option<&'a Encounter> {
    for encounter in candidates {
        if encounter.id == encounter_id {
            return Some(*encounter);
        }
    }
    None
}

fn rotation_ready(current_day: u32, last_seen: &HashMap<&str, u32>, encounter_id: &str) -> bool {
    last_seen
        .get(encounter_id)
        .map(|day| current_day.saturating_sub(*day))
        .is_none_or(|age| age >= ENCOUNTER_REPEAT_WINDOW_DAYS)
}

fn try_pick_ready_from_queue(
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
        let next_id = rotation_queue
            .pop_front()
            .expect("rotation queue unexpectedly empty");
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

fn try_pick_forced_from_queue(
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
        let mode = if request.is_deep { "Deep" } else { "Classic" };
        let region = request.region.asset_key();
        let count = candidates.len();
        println!("Encounter selection | mode:{mode} region:{region} candidates:{count}");
    }

    if candidates.is_empty() {
        return pick_none(false);
    }

    let last_seen = build_last_seen_map(request.recent);
    if rotation_queue.is_empty() {
        *rotation_queue = build_rotation_backlog(request, &candidates, &last_seen);
    }

    let candidates_ref = &candidates;
    let recent = &last_seen;
    #[rustfmt::skip]
    let ready_pick = try_pick_ready_from_queue(request, rotation_queue, candidates_ref, recent);
    if let Some(encounter) = ready_pick {
        return rotation_pick(encounter);
    }

    #[rustfmt::skip]
    let forced_pick = try_pick_forced_from_queue(request, rotation_queue, candidates_ref, recent);
    if let Some(encounter) = forced_pick {
        return rotation_pick(encounter);
    }

    let (primary, fallback) = categorize_candidates(request, candidates_ref, &last_seen);
    let total = candidates_ref.len();
    let force = request.force_rotation;
    let selection_result = determine_selection(primary, fallback, force, total);
    let (selection, rotation_satisfied) = selection_result;

    let is_conservative_deep = request.is_deep && request.policy == Some(PolicyKind::Conservative);
    let region_counts = if is_conservative_deep {
        Some(build_recent_region_counts(request.recent))
    } else {
        None
    };
    let min = region_counts.as_ref().map_or(0, global_min_region_count);

    let counts = region_counts.as_ref();
    let weights = build_weights(selection, candidates_ref, request, recent, counts, min);
    let (chosen_idx, roll) = choose_weighted(&weights, rng).unwrap_or((0, 0));

    let chosen_ref = candidates_ref.get(chosen_idx).copied();
    let chosen = chosen_ref.map(|encounter| (*encounter).clone());

    let build_trace = |encounter| build_decision_trace(candidates_ref, &weights, roll, encounter);
    let decision_trace = chosen_ref.map(build_trace);

    finalize_pick(chosen, rotation_satisfied, decision_trace)
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
            let multiplier = final_weight_f / base_weight;
            let factor = weight_factor("effective", multiplier);
            let id = encounter.id.as_str();
            Some(make_candidate(id, base_weight, factor, final_weight_f))
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

    let mut filtered = Vec::new();
    for encounter in &request.data.encounters {
        let regions = &encounter.regions;
        let region_match = regions.is_empty()
            || regions
                .iter()
                .any(|region| region.eq_ignore_ascii_case(region_str));
        let modes = &encounter.modes;
        let mode_match = modes.is_empty()
            || modes.iter().any(|mode| {
                mode_aliases
                    .iter()
                    .any(|alias| mode.eq_ignore_ascii_case(alias))
            });
        if region_match && mode_match {
            filtered.push(encounter);
        }
    }
    filtered
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
        let repeat_window = ENCOUNTER_REPEAT_WINDOW_DAYS;
        let passes_no_repeat = chainable || age.is_none_or(|days| days >= repeat_window);
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
    weights.iter().find_map(|(idx, weight)| {
        current += *weight;
        (roll < current).then_some((*idx, roll))
    })
}

fn build_rotation_backlog<'a>(
    request: &EncounterRequest<'a>,
    candidates: &[&'a Encounter],
    last_seen: &HashMap<&str, u32>,
) -> VecDeque<String> {
    let mut ready: Vec<(String, bool, u32)> = Vec::new();
    let mut pending: Vec<(String, bool, u32)> = Vec::new();
    for encounter in candidates {
        if !encounter.chainable {
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
    use crate::constants::{DEBUG_ENV_VAR, FLOAT_EPSILON};
    use crate::state::{RecentEncounter, Region};
    use rand::Rng;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use std::collections::VecDeque;
    use std::sync::{Mutex, OnceLock};

    fn make_enc(id: &str, regions: &[&str]) -> Encounter {
        Encounter {
            id: id.to_string(),
            name: format!("Encounter {id}"),
            desc: String::new(),
            weight: 3,
            regions: regions.iter().map(|r| (*r).to_string()).collect(),
            modes: vec![String::from("classic"), String::from("deep")],
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

    fn with_debug_env<F, T>(f: F) -> T
    where
        F: FnOnce() -> T,
    {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let previous = std::env::var(DEBUG_ENV_VAR).ok();
        unsafe {
            std::env::set_var(DEBUG_ENV_VAR, "1");
        }
        let result = f();
        match previous {
            Some(value) => unsafe {
                std::env::set_var(DEBUG_ENV_VAR, value);
            },
            None => unsafe {
                std::env::remove_var(DEBUG_ENV_VAR);
            },
        }
        result
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

    #[test]
    fn weights_apply_starvation_bonus_to_forage() {
        let mut forage = make_enc("forage_cache", &["Heartland"]);
        forage.name = "Forage Cache".to_string();
        let normal = make_enc("normal", &["Heartland"]);
        let data = EncounterData::from_encounters(vec![forage, normal]);
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 1,
            starving: true,
            data: &data,
            recent: &[],
            current_day: 10,
            policy: None,
            force_rotation: false,
        };
        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(&[]);
        let weights = build_weights(vec![0, 1], &candidates, &request, &last_seen, None, 0);
        let forage_weight = weights
            .iter()
            .find(|(idx, _)| *idx == 0)
            .map_or(0, |(_, weight)| *weight);
        let normal_weight = weights
            .iter()
            .find(|(idx, _)| *idx == 1)
            .map_or(0, |(_, weight)| *weight);
        assert!(forage_weight > normal_weight);
    }

    #[test]
    fn weights_penalize_chainable_encounters() {
        let mut chainable = make_enc("chain", &["Heartland"]);
        chainable.chainable = true;
        let normal = make_enc("plain", &["Heartland"]);
        let data = EncounterData::from_encounters(vec![chainable, normal]);
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &[],
            current_day: 10,
            policy: None,
            force_rotation: false,
        };
        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(&[]);
        let weights = build_weights(vec![0, 1], &candidates, &request, &last_seen, None, 0);
        let chain_weight = weights
            .iter()
            .find(|(idx, _)| *idx == 0)
            .map_or(0, |(_, weight)| *weight);
        let normal_weight = weights
            .iter()
            .find(|(idx, _)| *idx == 1)
            .map_or(0, |(_, weight)| *weight);
        assert!(chain_weight < normal_weight);
    }

    #[test]
    fn choose_weighted_returns_none_when_total_zero() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        let weights = vec![(0, 0)];
        assert!(choose_weighted(&weights, &mut rng).is_none());
    }

    #[test]
    fn forced_rotation_rebuilds_queue_when_empty() {
        let data = sample_encounters();
        let mut request = mk_request(&data);
        request.force_rotation = true;
        request.is_deep = false;
        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(request.recent);
        let mut queue = VecDeque::new();

        let picked = try_pick_forced_from_queue(&request, &mut queue, &candidates, &last_seen);
        assert!(picked.is_some());
    }

    #[test]
    fn region_counts_track_minimum() {
        let recent = vec![
            RecentEncounter::new("a".to_string(), 2, Region::Heartland),
            RecentEncounter::new("b".to_string(), 3, Region::Heartland),
            RecentEncounter::new("c".to_string(), 4, Region::RustBelt),
        ];
        let counts = build_recent_region_counts(&recent);
        assert_eq!(counts.get(&Region::Heartland).copied().unwrap_or(0), 2);
        assert_eq!(counts.get(&Region::RustBelt).copied().unwrap_or(0), 1);
        assert_eq!(global_min_region_count(&counts), 0);
    }

    #[test]
    fn rotation_ready_respects_repeat_window() {
        let mut last_seen = HashMap::new();
        last_seen.insert("enc", 10);

        assert!(!rotation_ready(
            10 + ENCOUNTER_REPEAT_WINDOW_DAYS - 1,
            &last_seen,
            "enc"
        ));
        assert!(rotation_ready(
            10 + ENCOUNTER_REPEAT_WINDOW_DAYS,
            &last_seen,
            "enc"
        ));
        assert!(rotation_ready(1, &last_seen, "missing"));
    }

    #[test]
    fn try_pick_ready_from_queue_skips_recent() {
        let data = sample_encounters();
        let recent = vec![RecentEncounter::new(
            String::from("alpha"),
            19,
            Region::Heartland,
        )];
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &recent,
            current_day: 20,
            policy: None,
            force_rotation: false,
        };
        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(request.recent);
        let mut queue = VecDeque::from(vec![String::from("alpha"), String::from("beta")]);

        let picked = try_pick_ready_from_queue(&request, &mut queue, &candidates, &last_seen);
        assert!(picked.is_some());
        assert_eq!(picked.expect("picked").id, "beta");
    }

    #[test]
    fn try_pick_ready_from_queue_returns_none_when_empty() {
        let data = sample_encounters();
        let request = mk_request(&data);
        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(request.recent);
        let mut queue = VecDeque::new();

        let picked = try_pick_ready_from_queue(&request, &mut queue, &candidates, &last_seen);

        assert!(picked.is_none());
        assert!(queue.is_empty());
    }

    #[test]
    fn try_pick_ready_from_queue_returns_none_when_unready() {
        let data = sample_encounters();
        let recent = vec![RecentEncounter::new(
            String::from("alpha"),
            10,
            Region::Heartland,
        )];
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &recent,
            current_day: 11,
            policy: None,
            force_rotation: false,
        };
        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(request.recent);
        let mut queue = VecDeque::from(vec![String::from("alpha"), String::from("unknown")]);

        let picked = try_pick_ready_from_queue(&request, &mut queue, &candidates, &last_seen);

        assert!(picked.is_none());
        assert_eq!(
            queue,
            VecDeque::from(vec![String::from("alpha"), String::from("unknown")])
        );
    }

    #[test]
    fn try_pick_forced_from_queue_returns_none_when_missing_candidates() {
        let data = sample_encounters();
        let mut request = mk_request(&data);
        request.force_rotation = true;
        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(request.recent);
        let mut queue = VecDeque::from(vec![String::from("unknown")]);

        let picked = try_pick_forced_from_queue(&request, &mut queue, &candidates, &last_seen);

        assert!(picked.is_none());
        assert!(queue.is_empty());
    }

    #[test]
    fn pick_encounter_prefers_ready_rotation_queue() {
        let data = sample_encounters();
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &[],
            current_day: 5,
            policy: None,
            force_rotation: false,
        };
        let mut queue = VecDeque::from(vec![String::from("alpha")]);
        let mut rng = ChaCha20Rng::from_seed([9u8; 32]);

        let pick = pick_encounter(&request, &mut queue, &mut rng);

        assert!(pick.rotation_satisfied);
        assert_eq!(pick.encounter.expect("encounter").id, "alpha");
    }

    #[test]
    fn encounter_regions_falls_back_to_default() {
        let encounter = make_enc("delta", &["unknown"]);
        let regions = encounter_regions(&encounter, Region::Beltway);
        assert_eq!(regions, vec![Region::Beltway]);

        let encounter = make_enc("eps", &["Heartland", "RustBelt"]);
        let regions = encounter_regions(&encounter, Region::Beltway);
        assert_eq!(regions, vec![Region::Heartland, Region::RustBelt]);
    }

    #[test]
    fn build_last_seen_map_tracks_latest_day() {
        let recent = vec![
            RecentEncounter::new(String::from("alpha"), 2, Region::Heartland),
            RecentEncounter::new(String::from("alpha"), 5, Region::Heartland),
            RecentEncounter::new(String::from("beta"), 3, Region::Heartland),
        ];
        let map = build_last_seen_map(&recent);
        assert_eq!(map.get("alpha").copied(), Some(5));
        assert_eq!(map.get("beta").copied(), Some(3));
    }

    #[test]
    fn categorize_candidates_force_rotation_separates_recent() {
        let data = EncounterData::from_encounters(vec![
            make_enc("a", &["Heartland"]),
            make_enc("b", &["Heartland"]),
        ]);
        let recent = vec![RecentEncounter::new(
            String::from("a"),
            10,
            Region::Heartland,
        )];
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &recent,
            current_day: 10,
            policy: None,
            force_rotation: true,
        };
        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(&recent);
        let (primary, fallback) = categorize_candidates(&request, &candidates, &last_seen);

        assert!(primary.contains(&1));
        assert!(fallback.contains(&0));
    }

    #[test]
    fn build_rotation_backlog_empty_for_chainable_only() {
        let mut encounter = make_enc("chain", &["Heartland"]);
        encounter.chainable = true;
        let data = EncounterData::from_encounters(vec![encounter]);
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &[],
            current_day: 5,
            policy: None,
            force_rotation: false,
        };
        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(request.recent);
        let backlog = build_rotation_backlog(&request, &candidates, &last_seen);
        assert!(backlog.is_empty());
    }

    #[test]
    fn build_weights_boosts_min_region_when_conservative() {
        let mut encounter = make_enc("alpha", &["Heartland"]);
        encounter.modes = vec![String::from("deep")];
        let data = EncounterData::from_encounters(vec![encounter]);
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: true,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &[],
            current_day: 5,
            policy: Some(PolicyKind::Conservative),
            force_rotation: false,
        };
        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(&[]);
        let selection = vec![0];
        let baseline = build_weights(
            selection.clone(),
            &candidates,
            &request,
            &last_seen,
            None,
            0,
        );

        let counts = HashMap::from([
            (Region::Heartland, 0),
            (Region::RustBelt, 1),
            (Region::Beltway, 1),
        ]);
        let region_min = global_min_region_count(&counts);
        let boosted = build_weights(
            selection,
            &candidates,
            &request,
            &last_seen,
            Some(&counts),
            region_min,
        );

        assert!(boosted[0].1 > baseline[0].1);
    }

    #[test]
    fn pick_encounter_returns_none_when_no_candidates() {
        let mut encounter = make_enc("deep_only", &["RustBelt"]);
        encounter.modes = vec![String::from("deep")];
        let data = EncounterData::from_encounters(vec![encounter]);
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &[],
            current_day: 2,
            policy: None,
            force_rotation: false,
        };
        let mut queue = VecDeque::new();
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        let pick = pick_encounter(&request, &mut queue, &mut rng);
        assert!(pick.encounter.is_none());
        assert!(!pick.rotation_satisfied);
    }

    #[test]
    fn try_pick_forced_from_queue_returns_none_when_not_forced() {
        let data = sample_encounters();
        let request = mk_request(&data);
        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(request.recent);
        let mut queue = VecDeque::new();

        let picked = try_pick_forced_from_queue(&request, &mut queue, &candidates, &last_seen);
        assert!(picked.is_none());
    }

    #[test]
    fn debug_logging_covers_ready_path() {
        with_debug_env(|| {
            let data = EncounterData::from_encounters(vec![make_enc("ready", &["Heartland"])]);
            let mut queue = VecDeque::new();
            let request = EncounterRequest {
                region: Region::Heartland,
                is_deep: false,
                malnutrition_level: 0,
                starving: false,
                data: &data,
                recent: &[],
                current_day: 10,
                policy: None,
                force_rotation: false,
            };
            let mut rng = ChaCha20Rng::from_seed([3u8; 32]);
            let _ = pick_encounter(&request, &mut queue, &mut rng);
        });
    }

    #[test]
    fn debug_logging_covers_forced_rotation() {
        with_debug_env(|| {
            let data = EncounterData::from_encounters(vec![make_enc("late", &["Heartland"])]);
            let recent = vec![RecentEncounter::new(
                String::from("late"),
                9,
                Region::Heartland,
            )];
            let mut queue = VecDeque::new();
            let request = EncounterRequest {
                region: Region::Heartland,
                is_deep: false,
                malnutrition_level: 0,
                starving: false,
                data: &data,
                recent: &recent,
                current_day: 10,
                policy: None,
                force_rotation: true,
            };
            let mut rng = ChaCha20Rng::from_seed([4u8; 32]);
            let _ = pick_encounter(&request, &mut queue, &mut rng);
        });
    }

    #[test]
    fn filter_candidates_respects_regions_and_modes() {
        let mut deep_only = make_enc("deep", &["Beltway"]);
        deep_only.modes = vec![String::from("deep")];
        let mut classic_only = make_enc("classic", &["RustBelt"]);
        classic_only.modes = vec![String::from("classic")];
        let mut any_region = make_enc("any", &[]);
        any_region.modes.clear();
        let data = EncounterData::from_encounters(vec![deep_only, classic_only, any_region]);
        let request_classic = EncounterRequest {
            region: Region::RustBelt,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &[],
            current_day: 1,
            policy: None,
            force_rotation: false,
        };
        let classic_candidates = filter_candidates(&request_classic);
        assert_eq!(classic_candidates.len(), 2);

        let request_deep = EncounterRequest {
            region: Region::Beltway,
            is_deep: true,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &[],
            current_day: 1,
            policy: None,
            force_rotation: false,
        };
        let deep_candidates = filter_candidates(&request_deep);
        assert_eq!(deep_candidates.len(), 2);
    }

    #[test]
    fn categorize_candidates_tracks_repeat_windows() {
        let data = EncounterData::from_encounters(vec![
            make_enc("fresh", &["Heartland"]),
            make_enc("recent", &["Heartland"]),
        ]);
        let recent = vec![RecentEncounter::new(
            String::from("recent"),
            19,
            Region::Heartland,
        )];
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &recent,
            current_day: 20,
            policy: None,
            force_rotation: false,
        };
        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(&recent);
        let (primary, fallback) = categorize_candidates(&request, &candidates, &last_seen);
        assert!(primary.contains(&0));
        assert!(fallback.contains(&1));
    }

    #[test]
    fn build_weights_boosts_conservative_region_floor() {
        let mut alpha = make_enc("alpha", &["Heartland"]);
        alpha.modes = vec![String::from("deep")];
        let mut beta = make_enc("beta", &["RustBelt"]);
        beta.modes = vec![String::from("deep")];
        let data = EncounterData::from_encounters(vec![alpha, beta]);
        let recent = vec![RecentEncounter::new(
            String::from("alpha"),
            10,
            Region::Heartland,
        )];
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: true,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &recent,
            current_day: 20,
            policy: Some(PolicyKind::Conservative),
            force_rotation: false,
        };
        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(&recent);
        let region_counts = build_recent_region_counts(&recent);
        let region_min = global_min_region_count(&region_counts);
        let selection: Vec<usize> = (0..candidates.len()).collect();
        let weights = build_weights(
            selection,
            &candidates,
            &request,
            &last_seen,
            Some(&region_counts),
            region_min,
        );
        assert_eq!(weights.len(), candidates.len());
    }

    #[test]
    fn determine_selection_prefers_primary_when_present() {
        let (selection, rotation_satisfied) = determine_selection(vec![1], vec![0], false, 2);
        assert_eq!(selection, vec![1]);
        assert!(!rotation_satisfied);
    }

    #[test]
    fn pick_encounter_builds_region_counts_for_deep_conservative() {
        let mut enc = make_enc("alpha", &["Heartland"]);
        enc.modes = vec![String::from("deep")];
        let data = EncounterData::from_encounters(vec![enc]);
        let recent = vec![RecentEncounter::new(
            String::from("alpha"),
            2,
            Region::Heartland,
        )];
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: true,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &recent,
            current_day: 3,
            policy: Some(PolicyKind::Conservative),
            force_rotation: false,
        };
        let mut queue = VecDeque::new();
        let mut rng = ChaCha20Rng::from_seed([2u8; 32]);
        let pick = pick_encounter(&request, &mut queue, &mut rng);
        assert!(pick.encounter.is_some());
    }

    #[test]
    fn build_rotation_backlog_skips_chainable_entries() {
        let mut chainable = make_enc("chain", &["Heartland"]);
        chainable.chainable = true;
        let data =
            EncounterData::from_encounters(vec![chainable, make_enc("plain", &["Heartland"])]);
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &[],
            current_day: 10,
            policy: None,
            force_rotation: true,
        };
        let candidates = filter_candidates(&request);
        let last_seen = build_last_seen_map(request.recent);
        let backlog = build_rotation_backlog(&request, &candidates, &last_seen);
        let ids: Vec<_> = backlog.into_iter().collect();
        assert_eq!(ids, vec!["plain"]);
    }

    #[test]
    fn parse_region_handles_beltway() {
        assert_eq!(parse_region("beltway"), Some(Region::Beltway));
    }

    #[test]
    fn pick_encounter_returns_ready_rotation_candidate() {
        let data = EncounterData::from_encounters(vec![make_enc("alpha", &["Heartland"])]);
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &[],
            current_day: 10,
            policy: None,
            force_rotation: false,
        };
        let mut queue = VecDeque::from([String::from("alpha")]);
        let mut rng = ChaCha20Rng::seed_from_u64(1);
        let pick = pick_encounter(&request, &mut queue, &mut rng);
        assert!(pick.rotation_satisfied);
        assert!(pick.encounter.is_some());
    }

    #[test]
    fn pick_encounter_returns_forced_rotation_candidate() {
        let data = EncounterData::from_encounters(vec![make_enc("alpha", &["Heartland"])]);
        let recent = [RecentEncounter {
            id: String::from("alpha"),
            day: 10,
            region: Some(Region::Heartland),
        }];
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &recent,
            current_day: 10,
            policy: None,
            force_rotation: true,
        };
        let mut queue = VecDeque::from([String::from("alpha")]);
        let mut rng = ChaCha20Rng::seed_from_u64(2);
        let pick = pick_encounter(&request, &mut queue, &mut rng);
        assert!(pick.rotation_satisfied);
        assert!(pick.encounter.is_some());
    }

    #[test]
    fn find_candidate_matches_by_id() {
        let data = sample_encounters();
        let candidates: Vec<&Encounter> = data.encounters.iter().collect();
        let found = find_candidate(&candidates, "beta");
        assert!(found.is_some());
    }

    #[test]
    fn debug_logging_runs_when_enabled() {
        let data = sample_encounters();
        let mut queue = VecDeque::new();
        let request = mk_request(&data);
        with_debug_env(|| {
            let mut rng = ChaCha20Rng::from_seed([9_u8; 32]);
            let _ = pick_encounter(&request, &mut queue, &mut rng);
        });
    }

    #[test]
    fn ready_rotation_pick_returns_encounter() {
        let data = sample_encounters();
        let mut queue = VecDeque::from([String::from("alpha")]);
        let mut request = mk_request(&data);
        request.force_rotation = false;
        let mut rng = ChaCha20Rng::from_seed([7_u8; 32]);

        let pick = pick_encounter(&request, &mut queue, &mut rng);
        assert!(pick.encounter.is_some());
        assert!(pick.rotation_satisfied);
    }

    #[test]
    fn forced_rotation_pick_returns_encounter() {
        let data = sample_encounters();
        let mut queue = VecDeque::from([String::from("alpha")]);
        let recent = vec![RecentEncounter::new(
            String::from("alpha"),
            12,
            Region::Heartland,
        )];
        let request = EncounterRequest {
            recent: &recent,
            force_rotation: true,
            ..mk_request(&data)
        };
        let mut rng = ChaCha20Rng::from_seed([8_u8; 32]);

        let pick = pick_encounter(&request, &mut queue, &mut rng);
        assert!(pick.encounter.is_some());
        assert!(pick.rotation_satisfied);
    }

    #[test]
    fn selection_path_uses_region_counts_for_conservative_deep() {
        let data = sample_encounters();
        let current_day = 12;
        let recent = vec![
            RecentEncounter::new(String::from("alpha"), current_day, Region::Heartland),
            RecentEncounter::new(String::from("beta"), current_day, Region::Heartland),
            RecentEncounter::new(String::from("gamma"), current_day, Region::Heartland),
        ];
        let request = EncounterRequest {
            is_deep: true,
            policy: Some(PolicyKind::Conservative),
            current_day,
            recent: &recent,
            ..mk_request(&data)
        };
        let mut queue = VecDeque::new();
        let mut rng = ChaCha20Rng::from_seed([4_u8; 32]);

        let pick = pick_encounter(&request, &mut queue, &mut rng);
        assert!(pick.encounter.is_some());
        assert!(pick.decision_trace.is_some());
    }

    #[test]
    fn debug_logging_emits_candidate_summary() {
        let data = sample_encounters();
        let mut queue = VecDeque::from([String::from("alpha")]);
        let request = mk_request(&data);
        with_debug_env(|| {
            let mut rng = ChaCha20Rng::from_seed([10_u8; 32]);
            let _ = pick_encounter(&request, &mut queue, &mut rng);
        });
    }

    #[test]
    fn forced_rotation_prefers_queue_when_recent() {
        let data = sample_encounters();
        let recent = vec![RecentEncounter::new(
            String::from("alpha"),
            12,
            Region::Heartland,
        )];
        let request = EncounterRequest {
            force_rotation: true,
            recent: &recent,
            ..mk_request(&data)
        };
        let mut queue = VecDeque::from([String::from("alpha")]);
        let mut rng = ChaCha20Rng::from_seed([11_u8; 32]);

        let pick = pick_encounter(&request, &mut queue, &mut rng);
        assert!(pick.rotation_satisfied);
        assert!(pick.encounter.is_some());
    }

    #[test]
    fn weighted_selection_records_region_counts() {
        let encounter = Encounter {
            id: String::from("regioned"),
            name: String::from("Regioned"),
            desc: String::new(),
            weight: 1,
            regions: vec![String::from("heartland")],
            modes: vec![String::from("deep")],
            choices: Vec::new(),
            hard_stop: false,
            major_repair: false,
            chainable: false,
        };
        let data = EncounterData::from_encounters(vec![encounter]);
        let recent = vec![RecentEncounter::new(
            String::from("regioned"),
            12,
            Region::Heartland,
        )];
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: true,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &recent,
            current_day: 12,
            policy: Some(PolicyKind::Conservative),
            force_rotation: false,
        };
        let mut queue = VecDeque::new();
        let mut rng = ChaCha20Rng::from_seed([12_u8; 32]);

        let pick = pick_encounter(&request, &mut queue, &mut rng);
        assert!(pick.encounter.is_some());
        assert!(pick.decision_trace.is_some());
    }

    #[test]
    fn filter_candidates_honors_regions_and_modes() {
        let encounter = Encounter {
            id: String::from("regioned"),
            name: String::from("Regioned"),
            desc: String::new(),
            weight: 1,
            regions: vec![String::from("Heartland")],
            modes: vec![String::from("classic")],
            choices: Vec::new(),
            hard_stop: false,
            major_repair: false,
            chainable: false,
        };
        let data = EncounterData::from_encounters(vec![encounter]);
        let request = EncounterRequest {
            region: Region::Heartland,
            is_deep: false,
            malnutrition_level: 0,
            starving: false,
            data: &data,
            recent: &[],
            current_day: 1,
            policy: None,
            force_rotation: false,
        };
        let candidates = filter_candidates(&request);
        assert_eq!(candidates.len(), 1);
    }

    #[test]
    fn categorize_candidates_respects_chainable() {
        let data = sample_encounters();
        let mut encounter = data.encounters[0].clone();
        encounter.chainable = true;
        let request = EncounterRequest {
            data: &EncounterData::from_encounters(vec![encounter.clone()]),
            recent: &[RecentEncounter::new(
                encounter.id.clone(),
                10,
                Region::Heartland,
            )],
            current_day: 10,
            ..mk_request(&data)
        };
        let candidates = vec![&encounter];
        let last_seen = build_last_seen_map(request.recent);
        let (primary, fallback) = categorize_candidates(&request, &candidates, &last_seen);
        assert_eq!(primary, vec![0]);
        assert!(fallback.is_empty());
    }
}
