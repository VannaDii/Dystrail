use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::journey::{EventDecisionTrace, RollValue, WeightFactor, WeightedCandidate};
use crate::mechanics::otdeluxe90s::OtDeluxeAfflictionPolicy;
use crate::weather::Weather;

const DEFAULT_DISEASE_DATA: &str =
    include_str!("../../dystrail-web/static/assets/data/disease.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiseaseKind {
    Illness,
    Injury,
}

impl DiseaseKind {
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Illness => "illness",
            Self::Injury => "injury",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DiseaseCatalog {
    #[serde(default)]
    pub diseases: Vec<DiseaseDef>,
}

impl DiseaseCatalog {
    #[must_use]
    pub fn load_from_static() -> Self {
        serde_json::from_str(DEFAULT_DISEASE_DATA).unwrap_or_default()
    }

    #[must_use]
    pub fn default_catalog() -> &'static Self {
        static CATALOG: OnceLock<DiseaseCatalog> = OnceLock::new();
        CATALOG.get_or_init(Self::load_from_static)
    }

    /// # Errors
    ///
    /// Returns an error if the JSON cannot be parsed into a disease catalog.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[must_use]
    pub fn pick_by_kind<R>(&self, kind: DiseaseKind, rng: &mut R) -> Option<&DiseaseDef>
    where
        R: Rng + ?Sized,
    {
        let (pick, _) = self.pick_by_kind_with_trace(kind, rng);
        pick
    }

    #[must_use]
    pub fn pick_by_kind_with_trace<R>(
        &self,
        kind: DiseaseKind,
        rng: &mut R,
    ) -> (Option<&DiseaseDef>, Option<EventDecisionTrace>)
    where
        R: Rng + ?Sized,
    {
        let mut candidates = Vec::new();
        let mut total_weight = 0_u32;
        for (idx, disease) in self.diseases.iter().enumerate() {
            if disease.kind == kind {
                let weight = u32::from(disease.weight);
                total_weight = total_weight.saturating_add(weight);
                candidates.push((idx, weight));
            }
        }
        if candidates.is_empty() {
            return (None, None);
        }

        let (chosen_idx, roll) = if total_weight == 0 {
            let choice = rng.gen_range(0..candidates.len());
            let roll = u32::try_from(choice).unwrap_or(0);
            (candidates[choice].0, roll)
        } else {
            let mut roll = rng.gen_range(0..total_weight);
            let original_roll = roll;
            let first_idx = candidates.first().map_or(0, |(idx, _)| *idx);
            let mut selected = first_idx;
            for (idx, weight) in &candidates {
                if *weight == 0 {
                    continue;
                }
                if roll < *weight {
                    selected = *idx;
                    break;
                }
                roll = roll.saturating_sub(*weight);
            }
            (selected, original_roll)
        };

        let uniform_fallback = total_weight == 0;
        let weighted_candidates = candidates
            .iter()
            .filter_map(|(idx, weight)| {
                let disease = self.diseases.get(*idx)?;
                let base = if uniform_fallback {
                    1.0
                } else {
                    f64::from(*weight)
                };
                let multipliers = if uniform_fallback {
                    vec![WeightFactor {
                        label: String::from("uniform_fallback"),
                        value: 1.0,
                    }]
                } else {
                    Vec::new()
                };
                let final_weight = base;
                Some(WeightedCandidate {
                    id: disease.id.clone(),
                    base_weight: base,
                    multipliers,
                    final_weight,
                })
            })
            .collect();

        let trace = self
            .diseases
            .get(chosen_idx)
            .map(|disease| EventDecisionTrace {
                pool_id: format!("otdeluxe.affliction_disease.{}", kind.key()),
                roll: RollValue::U32(roll),
                candidates: weighted_candidates,
                chosen_id: disease.id.clone(),
            });

        let chosen = self.diseases.get(chosen_idx);
        (chosen, trace)
    }

    #[must_use]
    pub fn find_by_id(&self, id: &str) -> Option<&DiseaseDef> {
        self.diseases.iter().find(|disease| disease.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiseaseDef {
    pub id: String,
    pub kind: DiseaseKind,
    pub display_key: String,
    #[serde(default = "default_weight")]
    pub weight: u16,
    #[serde(default)]
    pub duration_days: Option<u8>,
    #[serde(default)]
    pub onset_effects: DiseaseEffects,
    #[serde(default)]
    pub daily_tick_effects: DiseaseEffects,
    #[serde(default)]
    pub fatality_model: Option<FatalityModel>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl DiseaseDef {
    #[must_use]
    pub fn duration_for(&self, policy: &OtDeluxeAfflictionPolicy) -> u8 {
        let duration = self.duration_days.unwrap_or(match self.kind {
            DiseaseKind::Illness => policy.illness_duration_days,
            DiseaseKind::Injury => policy.injury_duration_days,
        });
        duration.max(1)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiseaseEffects {
    #[serde(default)]
    pub health_general_delta: i32,
    #[serde(default)]
    pub food_lbs_delta: i32,
    #[serde(default = "default_one_f32")]
    pub travel_speed_mult: f32,
}

impl Default for DiseaseEffects {
    fn default() -> Self {
        Self {
            health_general_delta: 0,
            food_lbs_delta: 0,
            travel_speed_mult: default_one_f32(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FatalityModel {
    #[serde(default)]
    pub base_prob_per_day: f32,
    #[serde(default)]
    pub prob_modifiers: Vec<FatalityModifier>,
    #[serde(default)]
    pub apply_doctor_mult: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FatalityModifier {
    Constant {
        mult: f32,
    },
    HealthLabel {
        good: f32,
        fair: f32,
        poor: f32,
        very_poor: f32,
    },
    Pace {
        steady: f32,
        strenuous: f32,
        grueling: f32,
    },
    Rations {
        filling: f32,
        meager: f32,
        bare_bones: f32,
    },
    Weather {
        weather: Weather,
        mult: f32,
    },
}

const fn default_weight() -> u16 {
    1
}

const fn default_one_f32() -> f32 {
    1.0
}
