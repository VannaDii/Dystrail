use rand::Rng;
use serde::Deserialize;
use std::sync::OnceLock;

use crate::journey::{EventDecisionTrace, RollValue, WeightFactor, WeightedCandidate};
use crate::state::Season;

const DEFAULT_RANDOM_EVENTS_DATA: &str =
    include_str!("../../dystrail-web/static/assets/data/otdeluxe/random_events.json");

const fn default_weight() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize)]
pub struct OtDeluxeRandomEventCatalog {
    #[serde(default)]
    pub chance_per_day: f32,
    #[serde(default)]
    pub events: Vec<OtDeluxeRandomEventDef>,
}

impl OtDeluxeRandomEventCatalog {
    #[must_use]
    pub fn load_from_static() -> Self {
        serde_json::from_str(DEFAULT_RANDOM_EVENTS_DATA).unwrap_or_default()
    }
}

impl Default for OtDeluxeRandomEventCatalog {
    fn default() -> Self {
        Self {
            chance_per_day: 0.0,
            events: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OtDeluxeRandomEventDef {
    pub id: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub variants: Vec<OtDeluxeRandomEventVariant>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OtDeluxeRandomEventVariant {
    pub id: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
}

#[derive(Debug, Clone)]
pub struct OtDeluxeRandomEventContext {
    pub season: Season,
    pub food_lbs: u16,
    pub oxen_total: u16,
    pub party_alive: u16,
    pub health_general: u16,
    pub spares_total: u16,
    pub weight_mult: f64,
    pub weight_cap: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct OtDeluxeRandomEventSelection {
    pub event_id: String,
    pub variant_id: Option<String>,
    pub chance_roll: f32,
    pub chance_threshold: f32,
}

#[derive(Debug, Clone)]
pub struct OtDeluxeRandomEventPick {
    pub selection: OtDeluxeRandomEventSelection,
    pub decision_trace: EventDecisionTrace,
    pub variant_trace: Option<EventDecisionTrace>,
}

#[must_use]
pub fn catalog() -> &'static OtDeluxeRandomEventCatalog {
    static CATALOG: OnceLock<OtDeluxeRandomEventCatalog> = OnceLock::new();
    CATALOG.get_or_init(OtDeluxeRandomEventCatalog::load_from_static)
}

pub fn pick_random_event_with_trace<R>(
    catalog: &OtDeluxeRandomEventCatalog,
    ctx: &OtDeluxeRandomEventContext,
    rng: &mut R,
) -> Option<OtDeluxeRandomEventPick>
where
    R: Rng + ?Sized,
{
    let base_chance = catalog.chance_per_day.clamp(0.0, 1.0);
    if base_chance <= 0.0 {
        return None;
    }

    let mut chance = base_chance;
    if matches!(ctx.season, Season::Winter) {
        chance *= 1.15;
    } else if matches!(ctx.season, Season::Summer) {
        chance *= 0.9;
    }
    if ctx.food_lbs < 150 {
        chance *= 1.1;
    }
    chance = chance.clamp(0.0, 1.0);

    let chance_roll = rng.r#gen::<f32>();
    if chance_roll >= chance {
        return None;
    }

    let mut weights = Vec::new();
    let mut total_weight = 0.0_f64;
    for event in &catalog.events {
        let (final_weight, factors) = event_weight_for_context(event, ctx);
        let base_weight = f64::from(event.weight);
        let candidate = WeightedCandidate {
            id: event.id.clone(),
            base_weight,
            multipliers: factors,
            final_weight,
        };
        weights.push((event, candidate));
        if final_weight > 0.0 {
            total_weight += final_weight;
        }
    }

    if total_weight <= 0.0 {
        return None;
    }

    let roll_f64 = rng.r#gen::<f64>() * total_weight;
    let mut remaining = roll_f64;
    let mut selected = catalog.events.first()?;
    for (event, candidate) in &weights {
        if candidate.final_weight <= 0.0 {
            continue;
        }
        if remaining < candidate.final_weight {
            selected = event;
            break;
        }
        remaining -= candidate.final_weight;
    }

    let candidates = weights
        .iter()
        .map(|(_, candidate)| candidate.clone())
        .collect();
    let decision_trace = EventDecisionTrace {
        pool_id: String::from("otdeluxe.random_events"),
        roll: RollValue::F64(roll_f64),
        candidates,
        chosen_id: selected.id.clone(),
    };

    let (variant_id, variant_trace) =
        pick_variant_with_trace(&selected.id, &selected.variants, rng);
    let selection = OtDeluxeRandomEventSelection {
        event_id: selected.id.clone(),
        variant_id,
        chance_roll,
        chance_threshold: chance,
    };

    Some(OtDeluxeRandomEventPick {
        selection,
        decision_trace,
        variant_trace,
    })
}

fn event_weight_for_context(
    event: &OtDeluxeRandomEventDef,
    ctx: &OtDeluxeRandomEventContext,
) -> (f64, Vec<WeightFactor>) {
    let base = f64::from(event.weight);
    if base <= 0.0 {
        return (
            0.0,
            vec![WeightFactor {
                label: String::from("base_zero"),
                value: 0.0,
            }],
        );
    }
    let mut factors = Vec::new();
    let mut weight = base;
    let id = event.id.as_str();

    match id {
        "weather_catastrophe" => {
            if matches!(ctx.season, Season::Winter) {
                apply_factor(&mut weight, &mut factors, "season_winter", 1.4);
            } else if matches!(ctx.season, Season::Summer) {
                apply_factor(&mut weight, &mut factors, "season_summer", 0.8);
            }
        }
        "resource_shortage" => {
            if ctx.food_lbs < 120 {
                apply_factor(&mut weight, &mut factors, "low_food", 1.3);
            }
        }
        "party_incident" => {
            if ctx.party_alive == 0 {
                apply_factor(&mut weight, &mut factors, "no_party", 0.0);
            } else if ctx.health_general >= 100 {
                apply_factor(&mut weight, &mut factors, "frail_party", 1.2);
            }
        }
        "oxen_incident" => {
            if ctx.oxen_total == 0 {
                apply_factor(&mut weight, &mut factors, "no_oxen", 0.0);
            } else if ctx.oxen_total < 3 {
                apply_factor(&mut weight, &mut factors, "low_oxen", 1.15);
            }
        }
        "resource_change" => {
            if ctx.food_lbs < 120 {
                apply_factor(&mut weight, &mut factors, "low_food", 1.1);
            }
        }
        "wagon_part_break" => {
            if ctx.spares_total == 0 {
                apply_factor(&mut weight, &mut factors, "no_spares", 0.9);
            } else {
                apply_factor(&mut weight, &mut factors, "has_spares", 1.05);
            }
        }
        "travel_hazard" => {
            if matches!(ctx.season, Season::Spring) {
                apply_factor(&mut weight, &mut factors, "season_spring", 1.1);
            }
        }
        _ => {}
    }

    if ctx.weight_mult.is_finite()
        && ctx.weight_mult >= 0.0
        && (ctx.weight_mult - 1.0).abs() > f64::EPSILON
    {
        apply_factor(
            &mut weight,
            &mut factors,
            "policy_event_weight_mult",
            ctx.weight_mult,
        );
    }
    if let Some(cap) = ctx.weight_cap
        && cap.is_finite()
        && cap >= 0.0
        && weight > cap
        && weight > 0.0
    {
        let factor = cap / weight;
        apply_factor(&mut weight, &mut factors, "policy_event_weight_cap", factor);
    }

    (weight, factors)
}

fn apply_factor(weight: &mut f64, factors: &mut Vec<WeightFactor>, label: &str, value: f64) {
    factors.push(WeightFactor {
        label: label.to_string(),
        value,
    });
    *weight *= value;
}

fn pick_variant_with_trace<R: Rng + ?Sized>(
    event_id: &str,
    variants: &[OtDeluxeRandomEventVariant],
    rng: &mut R,
) -> (Option<String>, Option<EventDecisionTrace>) {
    if variants.is_empty() {
        return (None, None);
    }

    let total_weight: u32 = variants.iter().map(|variant| variant.weight).sum();
    let (chosen_idx, roll) = if total_weight == 0 {
        let idx = rng.gen_range(0..variants.len());
        (idx, u32::try_from(idx).unwrap_or(0))
    } else {
        let roll = rng.gen_range(0..total_weight);
        let mut cursor = 0_u32;
        let mut selected = 0;
        for (idx, variant) in variants.iter().enumerate() {
            cursor = cursor.saturating_add(variant.weight);
            if roll < cursor {
                selected = idx;
                break;
            }
        }
        (selected, roll)
    };

    let candidates = variants
        .iter()
        .map(|variant| WeightedCandidate {
            id: variant.id.clone(),
            base_weight: f64::from(variant.weight),
            multipliers: Vec::new(),
            final_weight: f64::from(variant.weight),
        })
        .collect();
    let trace = EventDecisionTrace {
        pool_id: format!("otdeluxe.random_events.{event_id}.variant"),
        roll: RollValue::U32(roll),
        candidates,
        chosen_id: variants[chosen_idx].id.clone(),
    };

    (Some(variants[chosen_idx].id.clone()), Some(trace))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn find_event<'a>(
        catalog: &'a OtDeluxeRandomEventCatalog,
        id: &str,
    ) -> &'a OtDeluxeRandomEventDef {
        catalog
            .events
            .iter()
            .find(|event| event.id == id)
            .unwrap_or_else(|| panic!("missing event {id}"))
    }

    fn variant_ids(event: &OtDeluxeRandomEventDef) -> Vec<&str> {
        event
            .variants
            .iter()
            .map(|variant| variant.id.as_str())
            .collect()
    }

    #[test]
    fn random_event_catalog_covers_required_families() {
        let catalog = catalog();

        let weather = find_event(catalog, "weather_catastrophe");
        let weather_variants = variant_ids(weather);
        for id in [
            "blizzard",
            "hailstorm",
            "thunderstorm",
            "heavy_fog",
            "strong_winds",
        ] {
            assert!(
                weather_variants.contains(&id),
                "missing weather variant {id}"
            );
        }

        let shortage = find_event(catalog, "resource_shortage");
        let shortage_variants = variant_ids(shortage);
        for id in ["bad_water", "no_water", "no_grass"] {
            assert!(
                shortage_variants.contains(&id),
                "missing shortage variant {id}"
            );
        }

        let party = find_event(catalog, "party_incident");
        let party_variants = variant_ids(party);
        for id in ["lost_member", "snakebite"] {
            assert!(party_variants.contains(&id), "missing party variant {id}");
        }

        let oxen = find_event(catalog, "oxen_incident");
        let oxen_variants = variant_ids(oxen);
        for id in ["ox_wandered_off", "ox_sickness"] {
            assert!(oxen_variants.contains(&id), "missing oxen variant {id}");
        }

        let resources = find_event(catalog, "resource_change");
        let resource_variants = variant_ids(resources);
        for id in [
            "abandoned_wagon_empty",
            "abandoned_wagon_supplies",
            "thief",
            "wild_fruit",
            "mutual_aid_food",
            "gravesite",
            "fire",
        ] {
            assert!(
                resource_variants.contains(&id),
                "missing resource variant {id}"
            );
        }

        let wagon = find_event(catalog, "wagon_part_break");
        let wagon_variants = variant_ids(wagon);
        for id in ["repairable", "replaceable", "unrepairable"] {
            assert!(wagon_variants.contains(&id), "missing wagon variant {id}");
        }

        let travel = find_event(catalog, "travel_hazard");
        let travel_variants = variant_ids(travel);
        assert!(
            travel_variants.contains(&"rough_trail"),
            "missing travel hazard variant rough_trail"
        );
    }

    #[test]
    fn random_event_catalog_defaults_weights_when_missing() {
        let json = r#"{
            "chance_per_day": 1.0,
            "events": [
                {
                    "id": "weather_catastrophe",
                    "variants": [
                        { "id": "blizzard" }
                    ]
                }
            ]
        }"#;
        let catalog: OtDeluxeRandomEventCatalog =
            serde_json::from_str(json).expect("parse catalog");
        let event = catalog.events.first().expect("expected event");
        assert_eq!(event.weight, 1);
        let variant = event.variants.first().expect("expected variant");
        assert_eq!(variant.weight, 1);
    }

    #[test]
    fn random_event_catalog_default_is_empty() {
        let catalog = OtDeluxeRandomEventCatalog::default();
        assert!(catalog.chance_per_day.abs() < f32::EPSILON);
        assert!(catalog.events.is_empty());
    }

    #[test]
    fn random_event_weight_factors_apply_by_context() {
        let winter_ctx = OtDeluxeRandomEventContext {
            season: Season::Winter,
            food_lbs: 90,
            oxen_total: 0,
            party_alive: 0,
            health_general: 120,
            spares_total: 0,
            weight_mult: 1.0,
            weight_cap: None,
        };
        let spring_ctx = OtDeluxeRandomEventContext {
            season: Season::Spring,
            food_lbs: 90,
            oxen_total: 2,
            party_alive: 1,
            health_general: 120,
            spares_total: 2,
            weight_mult: 1.0,
            weight_cap: None,
        };
        let weather = OtDeluxeRandomEventDef {
            id: String::from("weather_catastrophe"),
            weight: 10,
            variants: Vec::new(),
        };
        let shortage = OtDeluxeRandomEventDef {
            id: String::from("resource_shortage"),
            weight: 10,
            variants: Vec::new(),
        };
        let party = OtDeluxeRandomEventDef {
            id: String::from("party_incident"),
            weight: 10,
            variants: Vec::new(),
        };
        let oxen = OtDeluxeRandomEventDef {
            id: String::from("oxen_incident"),
            weight: 10,
            variants: Vec::new(),
        };
        let wagon = OtDeluxeRandomEventDef {
            id: String::from("wagon_part_break"),
            weight: 10,
            variants: Vec::new(),
        };
        let hazard = OtDeluxeRandomEventDef {
            id: String::from("travel_hazard"),
            weight: 10,
            variants: Vec::new(),
        };

        let (weather_weight, weather_factors) = event_weight_for_context(&weather, &winter_ctx);
        assert!(weather_weight > f64::from(weather.weight));
        assert!(
            weather_factors
                .iter()
                .any(|factor| factor.label == "season_winter")
        );

        let (_, shortage_factors) = event_weight_for_context(&shortage, &winter_ctx);
        assert!(
            shortage_factors
                .iter()
                .any(|factor| factor.label == "low_food")
        );

        let (party_weight, party_factors) = event_weight_for_context(&party, &winter_ctx);
        assert!(party_weight.abs() < f64::EPSILON);
        assert!(
            party_factors
                .iter()
                .any(|factor| factor.label == "no_party")
        );

        let (oxen_weight, oxen_factors) = event_weight_for_context(&oxen, &winter_ctx);
        assert!(oxen_weight.abs() < f64::EPSILON);
        assert!(oxen_factors.iter().any(|factor| factor.label == "no_oxen"));

        let (_, wagon_factors) = event_weight_for_context(&wagon, &winter_ctx);
        assert!(
            wagon_factors
                .iter()
                .any(|factor| factor.label == "no_spares")
        );

        let (hazard_weight, hazard_factors) = event_weight_for_context(&hazard, &spring_ctx);
        assert!(hazard_weight > f64::from(hazard.weight));
        assert!(
            hazard_factors
                .iter()
                .any(|factor| factor.label == "season_spring")
        );
    }

    #[test]
    fn random_event_weight_overrides_apply() {
        let ctx = OtDeluxeRandomEventContext {
            season: Season::Fall,
            food_lbs: 200,
            oxen_total: 4,
            party_alive: 4,
            health_general: 80,
            spares_total: 1,
            weight_mult: 0.5,
            weight_cap: Some(3.0),
        };
        let event = OtDeluxeRandomEventDef {
            id: String::from("resource_change"),
            weight: 10,
            variants: Vec::new(),
        };
        let (weight, factors) = event_weight_for_context(&event, &ctx);

        assert!((weight - 3.0).abs() < 1e-6);
        assert!(
            factors
                .iter()
                .any(|factor| factor.label == "policy_event_weight_mult")
        );
        assert!(
            factors
                .iter()
                .any(|factor| factor.label == "policy_event_weight_cap")
        );
    }

    #[test]
    fn pick_variant_with_trace_handles_empty_and_zero_weights() {
        let mut rng = SmallRng::seed_from_u64(42);
        let (variant_id, trace) = pick_variant_with_trace("empty", &[], &mut rng);
        assert!(variant_id.is_none());
        assert!(trace.is_none());

        let variants = vec![
            OtDeluxeRandomEventVariant {
                id: String::from("a"),
                weight: 0,
            },
            OtDeluxeRandomEventVariant {
                id: String::from("b"),
                weight: 0,
            },
        ];
        let (variant_id, trace) = pick_variant_with_trace("zero", &variants, &mut rng);
        assert!(variant_id.is_some());
        assert!(trace.is_some());
    }

    #[test]
    fn pick_random_event_emits_decision_trace() {
        let catalog = OtDeluxeRandomEventCatalog {
            chance_per_day: 1.0,
            events: vec![
                OtDeluxeRandomEventDef {
                    id: String::from("resource_change"),
                    weight: 2,
                    variants: vec![OtDeluxeRandomEventVariant {
                        id: String::from("wild_fruit"),
                        weight: 1,
                    }],
                },
                OtDeluxeRandomEventDef {
                    id: String::from("travel_hazard"),
                    weight: 1,
                    variants: vec![OtDeluxeRandomEventVariant {
                        id: String::from("rough_trail"),
                        weight: 1,
                    }],
                },
            ],
        };
        let ctx = OtDeluxeRandomEventContext {
            season: Season::Fall,
            food_lbs: 200,
            oxen_total: 4,
            party_alive: 4,
            health_general: 80,
            spares_total: 1,
            weight_mult: 1.0,
            weight_cap: None,
        };
        let mut rng = SmallRng::seed_from_u64(7);
        let pick = pick_random_event_with_trace(&catalog, &ctx, &mut rng).expect("expected pick");

        assert!(matches!(pick.decision_trace.roll, RollValue::F64(_)));
        assert!(pick.decision_trace.candidates.len() >= 2);
        assert!(pick.selection.variant_id.is_some());
    }
}
