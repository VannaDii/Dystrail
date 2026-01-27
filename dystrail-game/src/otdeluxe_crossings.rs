//! Oregon Trail Deluxe river crossing resolver and helpers.

use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::journey::{EventDecisionTrace, RollValue, WeightFactor, WeightedCandidate};
use crate::mechanics::otdeluxe90s::{OtDeluxeCrossingOutcomeWeights, OtDeluxeCrossingPolicy};
use crate::numbers::{clamp_f64_to_f32, round_f64_to_i32};
use crate::otdeluxe_state::{
    OtDeluxeCrossingMethod, OtDeluxeInventory, OtDeluxeRiver, OtDeluxeRiverBed, OtDeluxeRiverState,
};
use crate::state::Season;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxeCrossingOutcome {
    Safe,
    StuckInMud,
    SuppliesWet,
    Tipped,
    Sank,
    Drowned,
}

impl OtDeluxeCrossingOutcome {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::StuckInMud => "stuck_in_mud",
            Self::SuppliesWet => "supplies_wet",
            Self::Tipped => "tipped",
            Self::Sank => "sank",
            Self::Drowned => "drowned",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OtDeluxeCrossingOptions {
    bits: u8,
}

impl OtDeluxeCrossingOptions {
    const FORD: u8 = 1 << 0;
    const CAULK_FLOAT: u8 = 1 << 1;
    const FERRY: u8 = 1 << 2;
    const GUIDE: u8 = 1 << 3;

    #[must_use]
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    #[must_use]
    pub const fn with_ford(mut self) -> Self {
        self.bits |= Self::FORD;
        self
    }

    #[must_use]
    pub const fn with_caulk_float(mut self) -> Self {
        self.bits |= Self::CAULK_FLOAT;
        self
    }

    #[must_use]
    pub const fn with_ferry(mut self) -> Self {
        self.bits |= Self::FERRY;
        self
    }

    #[must_use]
    pub const fn with_guide(mut self) -> Self {
        self.bits |= Self::GUIDE;
        self
    }

    #[must_use]
    pub const fn ford(self) -> bool {
        self.bits & Self::FORD != 0
    }

    #[must_use]
    pub const fn caulk_float(self) -> bool {
        self.bits & Self::CAULK_FLOAT != 0
    }

    #[must_use]
    pub const fn ferry(self) -> bool {
        self.bits & Self::FERRY != 0
    }

    #[must_use]
    pub const fn guide(self) -> bool {
        self.bits & Self::GUIDE != 0
    }

    #[must_use]
    pub const fn is_allowed(self, method: OtDeluxeCrossingMethod) -> bool {
        match method {
            OtDeluxeCrossingMethod::Ford => self.ford(),
            OtDeluxeCrossingMethod::CaulkFloat => self.caulk_float(),
            OtDeluxeCrossingMethod::Ferry => self.ferry(),
            OtDeluxeCrossingMethod::Guide => self.guide(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeCrossingResolution {
    pub outcome: OtDeluxeCrossingOutcome,
    pub crossing_days: u8,
    pub wait_days: u8,
    pub drying_days: u8,
    pub loss_ratio: f32,
    pub drownings: u8,
}

impl OtDeluxeCrossingResolution {
    #[must_use]
    pub const fn total_extra_days(self) -> u8 {
        self.wait_days
            .saturating_add(self.crossing_days)
            .saturating_add(self.drying_days)
    }
}

#[derive(Debug, Clone, Copy)]
struct RiskContext {
    risk_mult: f32,
    depth_bonus_safe: f32,
    wet_bonus: f32,
    sank_bonus: f32,
    stuck_bonus: f32,
    tipped_bonus: f32,
    guide_risk_mult: f32,
}

#[derive(Debug, Clone, Copy)]
struct DepthBand {
    is_shallow: bool,
    is_wet_goods: bool,
    is_swamped: bool,
}

#[must_use]
pub fn crossing_options(
    policy: &OtDeluxeCrossingPolicy,
    river: OtDeluxeRiver,
    river_state: &OtDeluxeRiverState,
    inventory: &OtDeluxeInventory,
) -> OtDeluxeCrossingOptions {
    let profile = policy.river_profiles.profile_for(river);
    let ferry_available = profile.ferry_available
        && river_state.depth_ft >= policy.ferry_min_depth_ft
        && inventory.cash_cents >= policy.ferry_cost_cents;
    let caulk_available = river_state.depth_ft >= policy.float_min_depth_ft;
    let guide_available = matches!(river, OtDeluxeRiver::Snake)
        && inventory.clothes_sets >= policy.guide_cost_clothes_sets;
    let mut options = OtDeluxeCrossingOptions::empty().with_ford();
    if caulk_available {
        options = options.with_caulk_float();
    }
    if ferry_available {
        options = options.with_ferry();
    }
    if guide_available {
        options = options.with_guide();
    }
    options
}

#[must_use]
pub fn derive_river_state(
    policy: &OtDeluxeCrossingPolicy,
    river: OtDeluxeRiver,
    season: Season,
    rain_accum: f32,
) -> OtDeluxeRiverState {
    let profile = policy.river_profiles.profile_for(river);
    let rain_factor = if policy.rain_accum_max > 0.0 {
        (rain_accum / policy.rain_accum_max).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let depth = blend_profile_value(
        profile.min_depth_ft,
        profile.max_depth_ft,
        rain_factor * policy.rain_depth_mult,
        policy.seasonal_depth_mult.for_season(season),
    );
    let width = blend_profile_value(
        profile.min_width_ft,
        profile.max_width_ft,
        rain_factor * policy.rain_width_mult,
        policy.seasonal_depth_mult.for_season(season),
    );
    let swiftness = blend_profile_value(
        profile.min_swiftness,
        profile.max_swiftness,
        rain_factor * policy.rain_swiftness_mult,
        policy.seasonal_swiftness_mult.for_season(season),
    );
    OtDeluxeRiverState {
        width_ft: width,
        depth_ft: depth,
        swiftness,
        bed: profile.bed,
    }
}

#[must_use]
pub fn resolve_crossing_with_trace<R: RngCore>(
    policy: &OtDeluxeCrossingPolicy,
    _river: OtDeluxeRiver,
    river_state: &OtDeluxeRiverState,
    method: OtDeluxeCrossingMethod,
    rng: &mut R,
) -> (OtDeluxeCrossingResolution, Option<EventDecisionTrace>) {
    let (weights, factor_map) = adjusted_weights(policy, river_state, method);
    let (outcome, draw) = pick_outcome(&weights, rng);
    let mut loss_ratio = match outcome {
        OtDeluxeCrossingOutcome::Tipped => policy.tipped_loss_ratio,
        OtDeluxeCrossingOutcome::Sank | OtDeluxeCrossingOutcome::Drowned => policy.sank_loss_ratio,
        _ => 0.0,
    };
    if matches!(method, OtDeluxeCrossingMethod::Guide) {
        loss_ratio *= policy.guide_loss_mult;
    }
    loss_ratio = loss_ratio.clamp(0.0, 1.0);

    let wait_days = if matches!(method, OtDeluxeCrossingMethod::Ferry) {
        sample_wait_days(policy, rng)
    } else {
        0
    };
    let drying_days = if matches!(outcome, OtDeluxeCrossingOutcome::SuppliesWet) {
        policy.drying_cost_days
    } else {
        0
    };
    let drownings = if matches!(outcome, OtDeluxeCrossingOutcome::Drowned) {
        sample_drownings(policy, rng)
    } else {
        0
    };
    let crossing_days = policy.crossing_cost_days;

    let pool_id = format!("otdeluxe.crossing.{}", method_id(method));
    let trace = build_trace(&pool_id, weights, factor_map, outcome, draw);

    let resolution = OtDeluxeCrossingResolution {
        outcome,
        crossing_days,
        wait_days,
        drying_days,
        loss_ratio,
        drownings,
    };
    (resolution, Some(trace))
}

#[must_use]
pub fn resolve_crossing<R: RngCore>(
    policy: &OtDeluxeCrossingPolicy,
    river: OtDeluxeRiver,
    river_state: &OtDeluxeRiverState,
    method: OtDeluxeCrossingMethod,
    rng: &mut R,
) -> OtDeluxeCrossingResolution {
    resolve_crossing_with_trace(policy, river, river_state, method, rng).0
}

#[must_use]
pub const fn river_for_index(index: usize) -> Option<OtDeluxeRiver> {
    match index {
        0 => Some(OtDeluxeRiver::Kansas),
        1 => Some(OtDeluxeRiver::BigBlue),
        2 => Some(OtDeluxeRiver::Green),
        3 => Some(OtDeluxeRiver::Snake),
        _ => None,
    }
}

#[must_use]
pub const fn node_index_for_river(river: OtDeluxeRiver) -> u8 {
    match river {
        OtDeluxeRiver::Kansas => 1,
        OtDeluxeRiver::BigBlue => 2,
        OtDeluxeRiver::Green => 9,
        OtDeluxeRiver::Snake => 12,
    }
}

fn adjusted_weights(
    policy: &OtDeluxeCrossingPolicy,
    river_state: &OtDeluxeRiverState,
    method: OtDeluxeCrossingMethod,
) -> (OtDeluxeCrossingOutcomeWeights, FactorMap) {
    let mut weights = base_weights_for_method(policy, method);
    let depth_band = depth_band(policy, river_state.depth_ft);
    let (risk_mult, depth_bonus_safe, wet_bonus, sank_bonus) = depth_risk_factors(
        policy,
        river_state.depth_ft,
        river_state.swiftness,
        depth_band,
        method,
    );
    let (stuck_bonus, tipped_bonus) = bed_risk_factors(policy, river_state.bed);
    let guide_risk_mult = if matches!(method, OtDeluxeCrossingMethod::Guide) {
        policy.guide_risk_mult
    } else {
        1.0
    };

    let risk_ctx = RiskContext {
        risk_mult,
        depth_bonus_safe,
        wet_bonus,
        sank_bonus,
        stuck_bonus,
        tipped_bonus,
        guide_risk_mult,
    };
    apply_risk_to_weights(&mut weights, river_state, risk_ctx, method);
    let factor_map = FactorMap::new(river_state, risk_ctx, method);
    (weights, factor_map)
}

fn apply_risk_to_weights(
    weights: &mut OtDeluxeCrossingOutcomeWeights,
    river_state: &OtDeluxeRiverState,
    ctx: RiskContext,
    method: OtDeluxeCrossingMethod,
) {
    let mut non_safe_mult = ctx.risk_mult;
    if matches!(method, OtDeluxeCrossingMethod::Guide) {
        non_safe_mult *= ctx.guide_risk_mult;
    }

    weights.stuck *= non_safe_mult;
    weights.wet *= non_safe_mult;
    weights.tipped *= non_safe_mult;
    weights.sank *= non_safe_mult;
    weights.drowned *= non_safe_mult;

    if ctx.depth_bonus_safe > 1.0 {
        weights.safe *= ctx.depth_bonus_safe;
    }
    if ctx.wet_bonus > 1.0 {
        weights.wet *= ctx.wet_bonus;
    }
    if ctx.sank_bonus > 1.0 {
        weights.sank *= ctx.sank_bonus;
    }

    match river_state.bed {
        OtDeluxeRiverBed::Muddy => {
            if ctx.stuck_bonus > 1.0 {
                weights.stuck *= ctx.stuck_bonus;
            }
        }
        OtDeluxeRiverBed::Rocky => {
            if ctx.tipped_bonus > 1.0 {
                weights.tipped *= ctx.tipped_bonus;
            }
        }
        OtDeluxeRiverBed::Unknown => {}
    }
}

const fn base_weights_for_method(
    policy: &OtDeluxeCrossingPolicy,
    method: OtDeluxeCrossingMethod,
) -> OtDeluxeCrossingOutcomeWeights {
    match method {
        OtDeluxeCrossingMethod::Ford | OtDeluxeCrossingMethod::Guide => policy.outcome_weights.ford,
        OtDeluxeCrossingMethod::CaulkFloat => policy.outcome_weights.caulk_float,
        OtDeluxeCrossingMethod::Ferry => policy.outcome_weights.ferry,
    }
}

fn depth_band(policy: &OtDeluxeCrossingPolicy, depth: f32) -> DepthBand {
    let wet_min = policy.wet_goods_min_depth_ft;
    let swamp_min = policy.swamped_min_depth_ft;
    DepthBand {
        is_shallow: depth < wet_min,
        is_wet_goods: depth >= wet_min && depth <= swamp_min,
        is_swamped: depth > swamp_min,
    }
}

fn depth_risk_factors(
    policy: &OtDeluxeCrossingPolicy,
    _depth: f32,
    swiftness: f32,
    band: DepthBand,
    method: OtDeluxeCrossingMethod,
) -> (f32, f32, f32, f32) {
    let swiftness_mult = if matches!(method, OtDeluxeCrossingMethod::Ferry) {
        let risk = swiftness.clamp(0.0, 1.0);
        risk.mul_add(policy.ferry_accident_risk_max, 1.0)
    } else {
        swiftness.max(0.0).mul_add(policy.swiftness_risk_mult, 1.0)
    };

    let safe_bonus = if band.is_shallow {
        policy.shallow_safe_bonus
    } else {
        1.0
    };
    let wet_bonus = if band.is_wet_goods {
        policy.wet_goods_bonus
    } else {
        1.0
    };
    let sank_bonus = if band.is_swamped {
        policy.swamped_sank_bonus
    } else {
        1.0
    };
    (swiftness_mult, safe_bonus, wet_bonus, sank_bonus)
}

const fn bed_risk_factors(policy: &OtDeluxeCrossingPolicy, bed: OtDeluxeRiverBed) -> (f32, f32) {
    match bed {
        OtDeluxeRiverBed::Muddy => (policy.stuck_muddy_mult, 1.0),
        OtDeluxeRiverBed::Rocky => (1.0, policy.tipped_rocky_mult),
        OtDeluxeRiverBed::Unknown => (1.0, 1.0),
    }
}

fn pick_outcome<R: RngCore>(
    weights: &OtDeluxeCrossingOutcomeWeights,
    rng: &mut R,
) -> (OtDeluxeCrossingOutcome, f32) {
    let mut total = weights.safe;
    total += weights.stuck;
    total += weights.wet;
    total += weights.tipped;
    total += weights.sank;
    total += weights.drowned;
    if total <= f32::EPSILON {
        return (OtDeluxeCrossingOutcome::Safe, 0.0);
    }
    let draw = safe_sample_ratio(rng.next_u32());
    let mut accum = 0.0;
    let mut select = |weight: f32| {
        accum += weight / total;
        draw <= accum
    };

    if select(weights.safe) {
        (OtDeluxeCrossingOutcome::Safe, draw)
    } else if select(weights.stuck) {
        (OtDeluxeCrossingOutcome::StuckInMud, draw)
    } else if select(weights.wet) {
        (OtDeluxeCrossingOutcome::SuppliesWet, draw)
    } else if select(weights.tipped) {
        (OtDeluxeCrossingOutcome::Tipped, draw)
    } else if select(weights.sank) {
        (OtDeluxeCrossingOutcome::Sank, draw)
    } else {
        (OtDeluxeCrossingOutcome::Drowned, draw)
    }
}

fn sample_wait_days<R: RngCore>(policy: &OtDeluxeCrossingPolicy, rng: &mut R) -> u8 {
    let min = policy.ferry_wait_days_min;
    let max = policy.ferry_wait_days_max;
    if min >= max {
        return min;
    }
    let span = u32::from(max.saturating_sub(min)) + 1;
    let offset = rng.next_u32() % span;
    let offset_u8 = u8::try_from(offset).unwrap_or(u8::MAX);
    min.saturating_add(offset_u8)
}

fn sample_drownings<R: RngCore>(policy: &OtDeluxeCrossingPolicy, rng: &mut R) -> u8 {
    let min = policy.drownings_min;
    let max = policy.drownings_max;
    if min >= max {
        return min;
    }
    let span = u32::from(max.saturating_sub(min)) + 1;
    let offset = rng.next_u32() % span;
    let offset_u8 = u8::try_from(offset).unwrap_or(u8::MAX);
    min.saturating_add(offset_u8)
}

fn build_trace(
    pool_id: &str,
    weights: OtDeluxeCrossingOutcomeWeights,
    factor_map: FactorMap,
    outcome: OtDeluxeCrossingOutcome,
    draw: f32,
) -> EventDecisionTrace {
    let candidates = vec![
        WeightedCandidate {
            id: OtDeluxeCrossingOutcome::Safe.id().to_string(),
            base_weight: f64::from(weights.safe),
            multipliers: factor_map.multipliers_for(OtDeluxeCrossingOutcome::Safe),
            final_weight: f64::from(weights.safe),
        },
        WeightedCandidate {
            id: OtDeluxeCrossingOutcome::StuckInMud.id().to_string(),
            base_weight: f64::from(weights.stuck),
            multipliers: factor_map.multipliers_for(OtDeluxeCrossingOutcome::StuckInMud),
            final_weight: f64::from(weights.stuck),
        },
        WeightedCandidate {
            id: OtDeluxeCrossingOutcome::SuppliesWet.id().to_string(),
            base_weight: f64::from(weights.wet),
            multipliers: factor_map.multipliers_for(OtDeluxeCrossingOutcome::SuppliesWet),
            final_weight: f64::from(weights.wet),
        },
        WeightedCandidate {
            id: OtDeluxeCrossingOutcome::Tipped.id().to_string(),
            base_weight: f64::from(weights.tipped),
            multipliers: factor_map.multipliers_for(OtDeluxeCrossingOutcome::Tipped),
            final_weight: f64::from(weights.tipped),
        },
        WeightedCandidate {
            id: OtDeluxeCrossingOutcome::Sank.id().to_string(),
            base_weight: f64::from(weights.sank),
            multipliers: factor_map.multipliers_for(OtDeluxeCrossingOutcome::Sank),
            final_weight: f64::from(weights.sank),
        },
        WeightedCandidate {
            id: OtDeluxeCrossingOutcome::Drowned.id().to_string(),
            base_weight: f64::from(weights.drowned),
            multipliers: factor_map.multipliers_for(OtDeluxeCrossingOutcome::Drowned),
            final_weight: f64::from(weights.drowned),
        },
    ];
    EventDecisionTrace {
        pool_id: pool_id.to_string(),
        roll: RollValue::F32(draw),
        candidates,
        chosen_id: outcome.id().to_string(),
    }
}

const fn method_id(method: OtDeluxeCrossingMethod) -> &'static str {
    match method {
        OtDeluxeCrossingMethod::Ford => "ford",
        OtDeluxeCrossingMethod::CaulkFloat => "caulk_float",
        OtDeluxeCrossingMethod::Ferry => "ferry",
        OtDeluxeCrossingMethod::Guide => "guide",
    }
}

fn blend_profile_value(min: f32, max: f32, rain_factor: f32, season_factor: f32) -> f32 {
    let span = (max - min).max(0.0);
    let mut value = min + span * rain_factor.clamp(0.0, 1.0);
    value *= season_factor.max(0.0);
    value.clamp(min, max.max(min))
}

fn safe_sample_ratio(sample: u32) -> f32 {
    let denom = f64::from(u32::MAX) + 1.0;
    let ratio = (f64::from(sample) + 0.5) / denom;
    clamp_f64_to_f32(ratio.clamp(0.0, 1.0))
}

#[derive(Debug, Clone, Copy)]
struct FactorMap {
    risk: f32,
    depth_safe: f32,
    wet_bonus: f32,
    sank_bonus: f32,
    stuck_bonus: f32,
    tipped_bonus: f32,
    guide_risk: f32,
    bed: OtDeluxeRiverBed,
}

fn is_not_one(value: f32) -> bool {
    (value - 1.0).abs() > f32::EPSILON
}

impl FactorMap {
    const fn new(
        river_state: &OtDeluxeRiverState,
        ctx: RiskContext,
        method: OtDeluxeCrossingMethod,
    ) -> Self {
        Self {
            risk: ctx.risk_mult,
            depth_safe: ctx.depth_bonus_safe,
            wet_bonus: ctx.wet_bonus,
            sank_bonus: ctx.sank_bonus,
            stuck_bonus: ctx.stuck_bonus,
            tipped_bonus: ctx.tipped_bonus,
            guide_risk: match method {
                OtDeluxeCrossingMethod::Guide => ctx.guide_risk_mult,
                _ => 1.0,
            },
            bed: river_state.bed,
        }
    }

    fn multipliers_for(self, outcome: OtDeluxeCrossingOutcome) -> Vec<WeightFactor> {
        let mut factors = Vec::new();
        if !matches!(outcome, OtDeluxeCrossingOutcome::Safe) && is_not_one(self.risk) {
            factors.push(WeightFactor {
                label: String::from("swiftness"),
                value: f64::from(self.risk),
            });
        }
        if matches!(outcome, OtDeluxeCrossingOutcome::Safe) && is_not_one(self.depth_safe) {
            factors.push(WeightFactor {
                label: String::from("depth_shallow"),
                value: f64::from(self.depth_safe),
            });
        }
        if matches!(outcome, OtDeluxeCrossingOutcome::SuppliesWet) && is_not_one(self.wet_bonus) {
            factors.push(WeightFactor {
                label: String::from("depth_wet_goods"),
                value: f64::from(self.wet_bonus),
            });
        }
        if matches!(outcome, OtDeluxeCrossingOutcome::Sank) && is_not_one(self.sank_bonus) {
            factors.push(WeightFactor {
                label: String::from("depth_swamped"),
                value: f64::from(self.sank_bonus),
            });
        }
        if matches!(outcome, OtDeluxeCrossingOutcome::StuckInMud)
            && matches!(self.bed, OtDeluxeRiverBed::Muddy)
            && is_not_one(self.stuck_bonus)
        {
            factors.push(WeightFactor {
                label: String::from("bed_muddy"),
                value: f64::from(self.stuck_bonus),
            });
        }
        if matches!(outcome, OtDeluxeCrossingOutcome::Tipped)
            && matches!(self.bed, OtDeluxeRiverBed::Rocky)
            && is_not_one(self.tipped_bonus)
        {
            factors.push(WeightFactor {
                label: String::from("bed_rocky"),
                value: f64::from(self.tipped_bonus),
            });
        }
        if is_not_one(self.guide_risk) && !matches!(outcome, OtDeluxeCrossingOutcome::Safe) {
            factors.push(WeightFactor {
                label: String::from("guide_risk_mult"),
                value: f64::from(self.guide_risk),
            });
        }
        factors
    }
}

#[must_use]
pub fn apply_loss_ratio(value: u16, ratio: f32) -> u16 {
    if value == 0 || ratio <= 0.0 {
        return 0;
    }
    let scaled = f64::from(value) * f64::from(ratio.clamp(0.0, 1.0));
    let rounded = round_f64_to_i32(scaled);
    if rounded <= 0 {
        0
    } else {
        u16::try_from(rounded).unwrap_or(u16::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::OtDeluxe90sPolicy;
    use crate::mechanics::otdeluxe90s::OtDeluxeCrossingOutcomeWeights;
    use crate::otdeluxe_state::OtDeluxeInventory;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    struct FixedRng(u32);

    impl RngCore for FixedRng {
        fn next_u32(&mut self) -> u32 {
            self.0
        }

        fn next_u64(&mut self) -> u64 {
            u64::from(self.next_u32())
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            let bytes = self.next_u32().to_le_bytes();
            for (idx, out) in dest.iter_mut().enumerate() {
                *out = bytes[idx % bytes.len()];
            }
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    #[test]
    fn pick_outcome_uses_total_weight() {
        let weights = OtDeluxeCrossingOutcomeWeights {
            safe: 1.0,
            stuck: 0.0,
            wet: 0.0,
            tipped: 0.0,
            sank: 0.0,
            drowned: 0.0,
        };
        let mut rng = FixedRng(0);
        let (outcome, draw) = pick_outcome(&weights, &mut rng);
        assert_eq!(outcome, OtDeluxeCrossingOutcome::Safe);
        assert!(draw >= 0.0);
    }

    #[test]
    fn river_state_clamps_within_profile_bounds() {
        let policy = OtDeluxe90sPolicy::default();
        let river_state = derive_river_state(
            &policy.crossings,
            OtDeluxeRiver::Kansas,
            Season::Spring,
            999.0,
        );
        let profile = policy.crossings.river_profiles.kansas;
        assert!(river_state.depth_ft >= profile.min_depth_ft);
        assert!(river_state.depth_ft <= profile.max_depth_ft);
        assert!(river_state.width_ft >= profile.min_width_ft);
        assert!(river_state.width_ft <= profile.max_width_ft);
    }

    #[test]
    fn crossing_resolution_returns_wait_days_for_ferry() {
        let policy = OtDeluxe90sPolicy::default();
        let river_state = OtDeluxeRiverState {
            depth_ft: 3.0,
            width_ft: 200.0,
            swiftness: 0.4,
            bed: OtDeluxeRiverBed::Muddy,
        };
        let mut rng = FixedRng(1);
        let resolution = resolve_crossing(
            &policy.crossings,
            OtDeluxeRiver::Kansas,
            &river_state,
            OtDeluxeCrossingMethod::Ferry,
            &mut rng,
        );
        assert!(
            resolution.wait_days >= policy.crossings.ferry_wait_days_min
                && resolution.wait_days <= policy.crossings.ferry_wait_days_max
        );
    }

    #[test]
    fn crossing_options_reflect_inventory_and_river_profiles() {
        let policy = OtDeluxe90sPolicy::default();
        let depth = policy
            .crossings
            .float_min_depth_ft
            .max(policy.crossings.ferry_min_depth_ft)
            + 0.1;
        let river_state = OtDeluxeRiverState {
            depth_ft: depth,
            width_ft: 210.0,
            swiftness: 0.45,
            bed: OtDeluxeRiverBed::Muddy,
        };
        let inventory = OtDeluxeInventory {
            cash_cents: policy.crossings.ferry_cost_cents,
            clothes_sets: policy.crossings.guide_cost_clothes_sets,
            ..OtDeluxeInventory::default()
        };

        let kansas = crossing_options(
            &policy.crossings,
            OtDeluxeRiver::Kansas,
            &river_state,
            &inventory,
        );
        assert!(kansas.ford());
        assert!(kansas.caulk_float());
        assert!(kansas.ferry());
        assert!(!kansas.guide());

        let big_blue = crossing_options(
            &policy.crossings,
            OtDeluxeRiver::BigBlue,
            &river_state,
            &inventory,
        );
        assert!(!big_blue.ferry());

        let snake = crossing_options(
            &policy.crossings,
            OtDeluxeRiver::Snake,
            &river_state,
            &inventory,
        );
        assert!(snake.guide());

        let low_cash = OtDeluxeInventory {
            cash_cents: 0,
            ..inventory
        };
        let green_low_cash = crossing_options(
            &policy.crossings,
            OtDeluxeRiver::Green,
            &river_state,
            &low_cash,
        );
        assert!(!green_low_cash.ferry());
    }

    #[test]
    fn crossing_options_respect_depth_thresholds() {
        let policy = OtDeluxe90sPolicy::default();
        let inventory = OtDeluxeInventory {
            cash_cents: policy.crossings.ferry_cost_cents,
            clothes_sets: policy.crossings.guide_cost_clothes_sets,
            ..OtDeluxeInventory::default()
        };

        let shallow = OtDeluxeRiverState {
            depth_ft: 1.4,
            width_ft: 200.0,
            swiftness: 0.4,
            bed: OtDeluxeRiverBed::Muddy,
        };
        let shallow_opts = crossing_options(
            &policy.crossings,
            OtDeluxeRiver::Kansas,
            &shallow,
            &inventory,
        );
        assert!(shallow_opts.ford());
        assert!(!shallow_opts.caulk_float());
        assert!(!shallow_opts.ferry());

        let mid = OtDeluxeRiverState {
            depth_ft: 2.4,
            width_ft: 200.0,
            swiftness: 0.4,
            bed: OtDeluxeRiverBed::Muddy,
        };
        let mid_opts = crossing_options(&policy.crossings, OtDeluxeRiver::Kansas, &mid, &inventory);
        assert!(mid_opts.caulk_float());
        assert!(!mid_opts.ferry());

        let deep = OtDeluxeRiverState {
            depth_ft: 2.6,
            width_ft: 200.0,
            swiftness: 0.4,
            bed: OtDeluxeRiverBed::Muddy,
        };
        let deep_opts =
            crossing_options(&policy.crossings, OtDeluxeRiver::Kansas, &deep, &inventory);
        assert!(deep_opts.caulk_float());
        assert!(deep_opts.ferry());
    }

    #[test]
    fn river_index_and_node_mapping_matches_expected_table() {
        assert_eq!(river_for_index(0), Some(OtDeluxeRiver::Kansas));
        assert_eq!(river_for_index(1), Some(OtDeluxeRiver::BigBlue));
        assert_eq!(river_for_index(2), Some(OtDeluxeRiver::Green));
        assert_eq!(river_for_index(3), Some(OtDeluxeRiver::Snake));
        assert_eq!(river_for_index(4), None);

        assert_eq!(node_index_for_river(OtDeluxeRiver::Kansas), 1);
        assert_eq!(node_index_for_river(OtDeluxeRiver::BigBlue), 2);
        assert_eq!(node_index_for_river(OtDeluxeRiver::Green), 9);
        assert_eq!(node_index_for_river(OtDeluxeRiver::Snake), 12);
    }

    #[test]
    fn apply_loss_ratio_clamps_and_rounds() {
        assert_eq!(apply_loss_ratio(0, 0.5), 0);
        assert_eq!(apply_loss_ratio(100, 0.0), 0);
        assert_eq!(apply_loss_ratio(100, 0.5), 50);
        assert_eq!(apply_loss_ratio(100, 2.0), 100);
    }

    #[test]
    fn options_helpers_and_total_days_cover_paths() {
        let options = OtDeluxeCrossingOptions::empty().with_ford().with_ferry();
        assert!(options.ford());
        assert!(options.ferry());
        assert!(!options.caulk_float());
        assert!(options.is_allowed(OtDeluxeCrossingMethod::Ford));
        assert!(!options.is_allowed(OtDeluxeCrossingMethod::Guide));

        let resolution = OtDeluxeCrossingResolution {
            outcome: OtDeluxeCrossingOutcome::Safe,
            crossing_days: 1,
            wait_days: 2,
            drying_days: 3,
            loss_ratio: 0.0,
            drownings: 0,
        };
        assert_eq!(resolution.total_extra_days(), 6);
    }

    #[test]
    fn depth_and_bed_helpers_cover_bands() {
        let policy = OtDeluxe90sPolicy::default();
        let wet_min = policy.crossings.wet_goods_min_depth_ft;
        let swamp_min = policy.crossings.swamped_min_depth_ft;
        let shallow = depth_band(&policy.crossings, wet_min - 0.1);
        let wet = depth_band(&policy.crossings, (wet_min + swamp_min) * 0.5);
        let swamped = depth_band(&policy.crossings, swamp_min + 0.1);
        assert!(shallow.is_shallow);
        assert!(wet.is_wet_goods);
        assert!(swamped.is_swamped);

        let (risk, safe_bonus, wet_bonus, sank_bonus) = depth_risk_factors(
            &policy.crossings,
            0.0,
            0.8,
            wet,
            OtDeluxeCrossingMethod::Ford,
        );
        assert!(risk >= 1.0);
        assert!(safe_bonus >= 1.0);
        assert!(wet_bonus >= 1.0);
        assert!(sank_bonus >= 1.0);

        let (ferry_risk, _, _, _) = depth_risk_factors(
            &policy.crossings,
            0.0,
            0.6,
            shallow,
            OtDeluxeCrossingMethod::Ferry,
        );
        assert!(ferry_risk >= 1.0);

        let (stuck_bonus, tipped_bonus) =
            bed_risk_factors(&policy.crossings, OtDeluxeRiverBed::Muddy);
        assert!(stuck_bonus >= 1.0);
        assert!(tipped_bonus >= 1.0);
    }

    #[test]
    fn resolve_crossing_with_trace_covers_loss_branches() {
        let mut policy = OtDeluxe90sPolicy::default();
        policy.crossings.outcome_weights.ford = OtDeluxeCrossingOutcomeWeights {
            safe: 0.0,
            stuck: 0.0,
            wet: 0.0,
            tipped: 1.0,
            sank: 0.0,
            drowned: 0.0,
        };
        policy.crossings.tipped_loss_ratio = 0.3;
        policy.crossings.guide_loss_mult = 0.5;
        let river_state = OtDeluxeRiverState {
            depth_ft: 3.5,
            width_ft: 180.0,
            swiftness: 0.5,
            bed: OtDeluxeRiverBed::Rocky,
        };
        let mut rng = FixedRng(4);
        let (resolution, trace) = resolve_crossing_with_trace(
            &policy.crossings,
            OtDeluxeRiver::Snake,
            &river_state,
            OtDeluxeCrossingMethod::Guide,
            &mut rng,
        );
        assert_eq!(resolution.outcome, OtDeluxeCrossingOutcome::Tipped);
        let expected_loss = policy.crossings.tipped_loss_ratio * policy.crossings.guide_loss_mult;
        assert!((resolution.loss_ratio - expected_loss).abs() < f32::EPSILON);
        assert!(trace.is_some());

        policy.crossings.outcome_weights.ford = OtDeluxeCrossingOutcomeWeights {
            safe: 0.0,
            stuck: 0.0,
            wet: 0.0,
            tipped: 0.0,
            sank: 1.0,
            drowned: 0.0,
        };
        let (sank_res, _) = resolve_crossing_with_trace(
            &policy.crossings,
            OtDeluxeRiver::Kansas,
            &river_state,
            OtDeluxeCrossingMethod::Ford,
            &mut FixedRng(2),
        );
        assert_eq!(sank_res.outcome, OtDeluxeCrossingOutcome::Sank);
        assert!((sank_res.loss_ratio - policy.crossings.sank_loss_ratio).abs() < f32::EPSILON);
    }

    #[test]
    fn pick_outcome_and_range_samples_cover_fallbacks() {
        let zero_weights = OtDeluxeCrossingOutcomeWeights {
            safe: 0.0,
            stuck: 0.0,
            wet: 0.0,
            tipped: 0.0,
            sank: 0.0,
            drowned: 0.0,
        };
        let (outcome, draw) = pick_outcome(&zero_weights, &mut FixedRng(1));
        assert_eq!(outcome, OtDeluxeCrossingOutcome::Safe);
        assert!(draw >= 0.0);

        let mut policy = OtDeluxe90sPolicy::default();
        policy.crossings.ferry_wait_days_min = 2;
        policy.crossings.ferry_wait_days_max = 2;
        policy.crossings.drownings_min = 1;
        policy.crossings.drownings_max = 1;
        let mut rng = FixedRng(9);
        assert_eq!(sample_wait_days(&policy.crossings, &mut rng), 2);
        assert_eq!(sample_drownings(&policy.crossings, &mut rng), 1);
    }

    #[test]
    fn factor_map_collects_multipliers() {
        let river_state = OtDeluxeRiverState {
            depth_ft: 2.0,
            width_ft: 100.0,
            swiftness: 0.4,
            bed: OtDeluxeRiverBed::Muddy,
        };
        let ctx = RiskContext {
            risk_mult: 1.2,
            depth_bonus_safe: 1.1,
            wet_bonus: 1.3,
            sank_bonus: 1.4,
            stuck_bonus: 1.5,
            tipped_bonus: 1.6,
            guide_risk_mult: 1.7,
        };
        let factor_map = FactorMap::new(&river_state, ctx, OtDeluxeCrossingMethod::Guide);
        let safe = factor_map.multipliers_for(OtDeluxeCrossingOutcome::Safe);
        assert!(safe.iter().any(|factor| factor.label == "depth_shallow"));

        let stuck = factor_map.multipliers_for(OtDeluxeCrossingOutcome::StuckInMud);
        assert!(stuck.iter().any(|factor| factor.label == "bed_muddy"));

        let wet = factor_map.multipliers_for(OtDeluxeCrossingOutcome::SuppliesWet);
        assert!(wet.iter().any(|factor| factor.label == "depth_wet_goods"));
    }

    #[test]
    fn crossing_options_allow_caulk_float() {
        let options = OtDeluxeCrossingOptions::empty().with_caulk_float();
        assert!(options.is_allowed(OtDeluxeCrossingMethod::CaulkFloat));
    }

    #[test]
    fn derive_river_state_handles_zero_rain_accum_max() {
        let mut policy = OtDeluxe90sPolicy::default();
        policy.crossings.rain_accum_max = 0.0;
        let river_state = derive_river_state(
            &policy.crossings,
            OtDeluxeRiver::Kansas,
            Season::Spring,
            100.0,
        );
        assert!(river_state.depth_ft.is_finite());
    }

    #[test]
    fn resolve_crossing_with_trace_covers_drying_and_drownings() {
        let mut policy = OtDeluxe90sPolicy::default();
        policy.crossings.drying_cost_days = 2;
        policy.crossings.outcome_weights.ford = OtDeluxeCrossingOutcomeWeights {
            safe: 0.0,
            stuck: 0.0,
            wet: 1.0,
            tipped: 0.0,
            sank: 0.0,
            drowned: 0.0,
        };
        let river_state = OtDeluxeRiverState {
            depth_ft: 2.8,
            width_ft: 200.0,
            swiftness: 0.4,
            bed: OtDeluxeRiverBed::Muddy,
        };
        let mut rng = FixedRng(1);
        let (resolution, _) = resolve_crossing_with_trace(
            &policy.crossings,
            OtDeluxeRiver::Kansas,
            &river_state,
            OtDeluxeCrossingMethod::Ford,
            &mut rng,
        );
        assert_eq!(resolution.drying_days, 2);

        policy.crossings.drownings_min = 1;
        policy.crossings.drownings_max = 1;
        policy.crossings.outcome_weights.ford = OtDeluxeCrossingOutcomeWeights {
            safe: 0.0,
            stuck: 0.0,
            wet: 0.0,
            tipped: 0.0,
            sank: 0.0,
            drowned: 1.0,
        };
        let mut rng = FixedRng(2);
        let (resolution, _) = resolve_crossing_with_trace(
            &policy.crossings,
            OtDeluxeRiver::Kansas,
            &river_state,
            OtDeluxeCrossingMethod::Ford,
            &mut rng,
        );
        assert_eq!(resolution.drownings, 1);
    }

    #[test]
    fn base_weights_for_method_covers_caulk_float() {
        let policy = OtDeluxe90sPolicy::default();
        let weights =
            base_weights_for_method(&policy.crossings, OtDeluxeCrossingMethod::CaulkFloat);
        assert_eq!(weights, policy.crossings.outcome_weights.caulk_float);
    }

    #[test]
    fn pick_outcome_covers_wet_and_drowned_paths() {
        let mut rng = FixedRng(0);
        let wet_weights = OtDeluxeCrossingOutcomeWeights {
            safe: 0.0,
            stuck: 0.0,
            wet: 1.0,
            tipped: 0.0,
            sank: 0.0,
            drowned: 0.0,
        };
        let (outcome, _) = pick_outcome(&wet_weights, &mut rng);
        assert_eq!(outcome, OtDeluxeCrossingOutcome::SuppliesWet);

        let mut rng = FixedRng(0);
        let drown_weights = OtDeluxeCrossingOutcomeWeights {
            safe: 0.0,
            stuck: 0.0,
            wet: 0.0,
            tipped: 0.0,
            sank: 0.0,
            drowned: 1.0,
        };
        let (outcome, _) = pick_outcome(&drown_weights, &mut rng);
        assert_eq!(outcome, OtDeluxeCrossingOutcome::Drowned);
    }

    #[test]
    fn sample_drownings_spans_range() {
        let mut policy = OtDeluxe90sPolicy::default();
        policy.crossings.drownings_min = 1;
        policy.crossings.drownings_max = 3;
        let mut rng = FixedRng(2);
        let result = sample_drownings(&policy.crossings, &mut rng);
        assert!((1..=3).contains(&result));
    }

    #[test]
    fn ferry_wait_days_distribution_is_uniformish() {
        let policy = OtDeluxe90sPolicy::default();
        let mut rng = SmallRng::seed_from_u64(42);
        let mut counts = [0u32; 7];
        let draws = 7000;

        for _ in 0..draws {
            let sample = sample_wait_days(&policy.crossings, &mut rng);
            let idx = usize::from(sample);
            counts[idx] = counts[idx].saturating_add(1);
        }

        let expected = f64::from(draws) / 7.0;
        let chi_square: f64 = counts
            .iter()
            .map(|&count| {
                let diff = f64::from(count) - expected;
                diff * diff / expected
            })
            .sum();

        assert!(
            chi_square < 20.0,
            "ferry wait days chi-square {chi_square:.2} exceeds threshold"
        );
    }

    #[test]
    fn method_id_covers_caulk_float() {
        assert_eq!(method_id(OtDeluxeCrossingMethod::CaulkFloat), "caulk_float");
    }

    #[test]
    fn apply_loss_ratio_rounds_down_to_zero() {
        assert_eq!(apply_loss_ratio(1, 0.01), 0);
    }

    #[test]
    fn pick_outcome_handles_weighted_total() {
        let mut rng = FixedRng(1);
        let weights = OtDeluxeCrossingOutcomeWeights {
            safe: 1.0,
            stuck: 1.0,
            wet: 1.0,
            tipped: 1.0,
            sank: 1.0,
            drowned: 1.0,
        };
        let (outcome, draw) = pick_outcome(&weights, &mut rng);
        assert!(draw >= 0.0);
        assert!(matches!(
            outcome,
            OtDeluxeCrossingOutcome::Safe
                | OtDeluxeCrossingOutcome::StuckInMud
                | OtDeluxeCrossingOutcome::SuppliesWet
                | OtDeluxeCrossingOutcome::Tipped
                | OtDeluxeCrossingOutcome::Sank
                | OtDeluxeCrossingOutcome::Drowned
        ));
    }
}
