use rand::Rng;

use crate::disease::{FatalityModel, FatalityModifier};
use crate::mechanics::OtDeluxeOccupation;
use crate::mechanics::otdeluxe90s::{OtDeluxe90sPolicy, OtDeluxePace, OtDeluxeRations};
use crate::weather::Weather;

#[derive(Debug, Clone, Copy)]
pub struct OtDeluxeFatalityContext {
    pub health_general: u16,
    pub pace: OtDeluxePace,
    pub rations: OtDeluxeRations,
    pub weather: Weather,
    pub occupation: Option<OtDeluxeOccupation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OtDeluxeHealthLabel {
    Good,
    Fair,
    Poor,
    VeryPoor,
}

#[must_use]
pub fn otdeluxe_fatality_probability(
    model: &FatalityModel,
    context: OtDeluxeFatalityContext,
    policy: &OtDeluxe90sPolicy,
) -> f32 {
    let mut prob = model.base_prob_per_day.max(0.0);
    for modifier in &model.prob_modifiers {
        let mult = match modifier {
            FatalityModifier::Constant { mult } => *mult,
            FatalityModifier::HealthLabel {
                good,
                fair,
                poor,
                very_poor,
            } => match otdeluxe_health_label(context.health_general, policy) {
                OtDeluxeHealthLabel::Good => *good,
                OtDeluxeHealthLabel::Fair => *fair,
                OtDeluxeHealthLabel::Poor => *poor,
                OtDeluxeHealthLabel::VeryPoor => *very_poor,
            },
            FatalityModifier::Pace {
                steady,
                strenuous,
                grueling,
            } => match context.pace {
                OtDeluxePace::Steady => *steady,
                OtDeluxePace::Strenuous => *strenuous,
                OtDeluxePace::Grueling => *grueling,
            },
            FatalityModifier::Rations {
                filling,
                meager,
                bare_bones,
            } => match context.rations {
                OtDeluxeRations::Filling => *filling,
                OtDeluxeRations::Meager => *meager,
                OtDeluxeRations::BareBones => *bare_bones,
            },
            FatalityModifier::Weather { weather: key, mult } => {
                if *key == context.weather {
                    *mult
                } else {
                    1.0
                }
            }
        };
        prob *= sanitize_multiplier(mult);
    }
    if model.apply_doctor_mult && matches!(context.occupation, Some(OtDeluxeOccupation::Doctor)) {
        prob *= sanitize_multiplier(policy.occupation_advantages.doctor_fatality_mult);
    }
    if prob.is_finite() {
        prob.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[must_use]
pub fn otdeluxe_roll_disease_fatality<R>(
    model: &FatalityModel,
    rng: &mut R,
    context: OtDeluxeFatalityContext,
    policy: &OtDeluxe90sPolicy,
) -> bool
where
    R: Rng + ?Sized,
{
    let prob = otdeluxe_fatality_probability(model, context, policy);
    prob > 0.0 && rng.r#gen::<f32>() < prob
}

const fn otdeluxe_health_label(
    health_general: u16,
    policy: &OtDeluxe90sPolicy,
) -> OtDeluxeHealthLabel {
    if health_general <= policy.health.label_ranges.good_max {
        OtDeluxeHealthLabel::Good
    } else if health_general <= policy.health.label_ranges.fair_max {
        OtDeluxeHealthLabel::Fair
    } else if health_general <= policy.health.label_ranges.poor_max {
        OtDeluxeHealthLabel::Poor
    } else {
        OtDeluxeHealthLabel::VeryPoor
    }
}

const fn sanitize_multiplier(mult: f32) -> f32 {
    if mult.is_finite() { mult.max(0.0) } else { 1.0 }
}
