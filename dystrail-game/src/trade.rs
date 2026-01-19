//! Trade offer generation and resolution.

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::journey::{EventDecisionTrace, RollValue, WeightedCandidate};
use crate::state::GameState;

const TRADE_MAX_OXEN: u32 = 2;
const TRADE_MAX_CLOTHES: u32 = 5;
const TRADE_MAX_BULLETS: u32 = 80;
const TRADE_MAX_FOOD_LBS: u32 = 100;
const TRADE_MAX_CASH_CENTS: u32 = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeGoodKind {
    Oxen,
    Clothes,
    Bullets,
    Wheel,
    Axle,
    Tongue,
    Food,
    Cash,
}

impl TradeGoodKind {
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Oxen => "oxen",
            Self::Clothes => "clothes",
            Self::Bullets => "bullets",
            Self::Wheel => "wheel",
            Self::Axle => "axle",
            Self::Tongue => "tongue",
            Self::Food => "food",
            Self::Cash => "cash",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeGood {
    pub kind: TradeGoodKind,
    pub amount: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeOffer {
    pub give: TradeGood,
    pub receive: TradeGood,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeResolution {
    Accepted,
    NoOffer,
    Unaffordable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeOutcome {
    pub offer: Option<TradeOffer>,
    pub resolution: TradeResolution,
}

impl TradeOutcome {
    #[must_use]
    pub const fn no_offer() -> Self {
        Self {
            offer: None,
            resolution: TradeResolution::NoOffer,
        }
    }
}

const TRADE_GOODS: [TradeGoodKind; 8] = [
    TradeGoodKind::Oxen,
    TradeGoodKind::Clothes,
    TradeGoodKind::Bullets,
    TradeGoodKind::Wheel,
    TradeGoodKind::Axle,
    TradeGoodKind::Tongue,
    TradeGoodKind::Food,
    TradeGoodKind::Cash,
];

#[must_use]
pub fn resolve_trade_with_rng(state: &mut GameState, rng: &mut impl Rng) -> TradeOutcome {
    let offer = generate_offer_with_rng(state, rng);
    apply_offer(state, offer)
}

#[must_use]
pub fn resolve_trade(state: &mut GameState) -> TradeOutcome {
    let offer = generate_offer_deterministic(state);
    apply_offer(state, offer)
}

fn generate_offer_with_rng(state: &mut GameState, rng: &mut impl Rng) -> Option<TradeOffer> {
    let give_candidates: Vec<TradeGoodKind> = TRADE_GOODS
        .iter()
        .copied()
        .filter(|kind| available_amount(state, *kind) > 0)
        .collect();
    if give_candidates.is_empty() {
        return None;
    }

    let give_roll = rng.gen_range(0..give_candidates.len());
    let give_kind = give_candidates[give_roll];
    record_trade_trace(
        state,
        "otdeluxe.trade.give",
        give_roll,
        &give_candidates,
        give_kind,
    );
    let receive_candidates: Vec<TradeGoodKind> = TRADE_GOODS
        .iter()
        .copied()
        .filter(|kind| *kind != give_kind)
        .collect();
    let receive_roll = rng.gen_range(0..receive_candidates.len());
    let receive_kind = receive_candidates[receive_roll];
    record_trade_trace(
        state,
        "otdeluxe.trade.receive",
        receive_roll,
        &receive_candidates,
        receive_kind,
    );

    let give_amount = generate_give_amount(rng, state, give_kind);
    let receive_amount = generate_receive_amount(rng, receive_kind);

    Some(TradeOffer {
        give: TradeGood {
            kind: give_kind,
            amount: give_amount,
        },
        receive: TradeGood {
            kind: receive_kind,
            amount: receive_amount,
        },
    })
}

fn record_trade_trace(
    state: &mut GameState,
    pool_id: &str,
    roll: usize,
    candidates: &[TradeGoodKind],
    chosen: TradeGoodKind,
) {
    let candidates = candidates
        .iter()
        .map(|kind| WeightedCandidate {
            id: kind.key().to_string(),
            base_weight: 1.0,
            multipliers: Vec::new(),
            final_weight: 1.0,
        })
        .collect();
    let trace = EventDecisionTrace {
        pool_id: pool_id.to_string(),
        roll: RollValue::U32(u32::try_from(roll).unwrap_or(0)),
        candidates,
        chosen_id: chosen.key().to_string(),
    };
    state.decision_traces_today.push(trace);
}

fn generate_offer_deterministic(state: &GameState) -> Option<TradeOffer> {
    let give_kind = TRADE_GOODS
        .iter()
        .copied()
        .find(|kind| available_amount(state, *kind) > 0)?;
    let receive_kind = TRADE_GOODS
        .iter()
        .copied()
        .find(|kind| *kind != give_kind)
        .unwrap_or(TradeGoodKind::Food);

    Some(TradeOffer {
        give: TradeGood {
            kind: give_kind,
            amount: minimum_amount(state, give_kind),
        },
        receive: TradeGood {
            kind: receive_kind,
            amount: minimum_amount(state, receive_kind),
        },
    })
}

fn generate_give_amount(rng: &mut impl Rng, state: &GameState, kind: TradeGoodKind) -> u32 {
    let available = available_amount(state, kind);
    let (min, max) = amount_bounds(kind);
    let desired = if min == max {
        min
    } else {
        rng.gen_range(min..=max)
    };
    desired.min(available.max(1))
}

fn generate_receive_amount(rng: &mut impl Rng, kind: TradeGoodKind) -> u32 {
    let (min, max) = amount_bounds(kind);
    if min == max {
        min
    } else {
        rng.gen_range(min..=max)
    }
}

fn minimum_amount(state: &GameState, kind: TradeGoodKind) -> u32 {
    let (min, _) = amount_bounds(kind);
    let available = available_amount(state, kind);
    min.max(1).min(available.max(1))
}

const fn amount_bounds(kind: TradeGoodKind) -> (u32, u32) {
    match kind {
        TradeGoodKind::Oxen => (1, TRADE_MAX_OXEN),
        TradeGoodKind::Clothes => (1, TRADE_MAX_CLOTHES),
        TradeGoodKind::Bullets => (20, TRADE_MAX_BULLETS),
        TradeGoodKind::Wheel | TradeGoodKind::Axle | TradeGoodKind::Tongue => (1, 1),
        TradeGoodKind::Food => (25, TRADE_MAX_FOOD_LBS),
        TradeGoodKind::Cash => (500, TRADE_MAX_CASH_CENTS),
    }
}

fn apply_offer(state: &mut GameState, offer: Option<TradeOffer>) -> TradeOutcome {
    let Some(offer) = offer else {
        return TradeOutcome::no_offer();
    };

    if !subtract_good(state, offer.give) {
        return TradeOutcome {
            offer: Some(offer),
            resolution: TradeResolution::Unaffordable,
        };
    }
    add_good(state, offer.receive);

    TradeOutcome {
        offer: Some(offer),
        resolution: TradeResolution::Accepted,
    }
}

fn available_amount(state: &GameState, kind: TradeGoodKind) -> u32 {
    match kind {
        TradeGoodKind::Oxen => u32::from(state.ot_deluxe.oxen.healthy),
        TradeGoodKind::Clothes => u32::from(state.ot_deluxe.inventory.clothes_sets),
        TradeGoodKind::Bullets => u32::from(state.ot_deluxe.inventory.bullets),
        TradeGoodKind::Wheel => u32::from(state.ot_deluxe.inventory.spares_wheels),
        TradeGoodKind::Axle => u32::from(state.ot_deluxe.inventory.spares_axles),
        TradeGoodKind::Tongue => u32::from(state.ot_deluxe.inventory.spares_tongues),
        TradeGoodKind::Food => u32::from(state.ot_deluxe.inventory.food_lbs),
        TradeGoodKind::Cash => state.ot_deluxe.inventory.cash_cents,
    }
}

fn subtract_good(state: &mut GameState, good: TradeGood) -> bool {
    let available = available_amount(state, good.kind);
    if available < good.amount || good.amount == 0 {
        return false;
    }
    match good.kind {
        TradeGoodKind::Oxen => {
            let amount = clamp_u16(good.amount);
            state.ot_deluxe.oxen.healthy = state.ot_deluxe.oxen.healthy.saturating_sub(amount);
        }
        TradeGoodKind::Clothes => {
            let amount = clamp_u16(good.amount);
            state.ot_deluxe.inventory.clothes_sets = state
                .ot_deluxe
                .inventory
                .clothes_sets
                .saturating_sub(amount);
        }
        TradeGoodKind::Bullets => {
            let amount = clamp_u16(good.amount);
            state.ot_deluxe.inventory.bullets =
                state.ot_deluxe.inventory.bullets.saturating_sub(amount);
        }
        TradeGoodKind::Wheel => {
            let amount = clamp_u8(good.amount);
            state.ot_deluxe.inventory.spares_wheels = state
                .ot_deluxe
                .inventory
                .spares_wheels
                .saturating_sub(amount);
        }
        TradeGoodKind::Axle => {
            let amount = clamp_u8(good.amount);
            state.ot_deluxe.inventory.spares_axles = state
                .ot_deluxe
                .inventory
                .spares_axles
                .saturating_sub(amount);
        }
        TradeGoodKind::Tongue => {
            let amount = clamp_u8(good.amount);
            state.ot_deluxe.inventory.spares_tongues = state
                .ot_deluxe
                .inventory
                .spares_tongues
                .saturating_sub(amount);
        }
        TradeGoodKind::Food => {
            let amount = clamp_u16(good.amount);
            state.ot_deluxe.inventory.food_lbs =
                state.ot_deluxe.inventory.food_lbs.saturating_sub(amount);
        }
        TradeGoodKind::Cash => {
            state.ot_deluxe.inventory.cash_cents = state
                .ot_deluxe
                .inventory
                .cash_cents
                .saturating_sub(good.amount);
        }
    }
    true
}

fn add_good(state: &mut GameState, good: TradeGood) {
    match good.kind {
        TradeGoodKind::Oxen => {
            let amount = clamp_u16(good.amount);
            state.ot_deluxe.oxen.healthy = state.ot_deluxe.oxen.healthy.saturating_add(amount);
        }
        TradeGoodKind::Clothes => {
            let amount = clamp_u16(good.amount);
            state.ot_deluxe.inventory.clothes_sets = state
                .ot_deluxe
                .inventory
                .clothes_sets
                .saturating_add(amount);
        }
        TradeGoodKind::Bullets => {
            let amount = clamp_u16(good.amount);
            state.ot_deluxe.inventory.bullets =
                state.ot_deluxe.inventory.bullets.saturating_add(amount);
        }
        TradeGoodKind::Wheel => {
            let amount = clamp_u8(good.amount);
            state.ot_deluxe.inventory.spares_wheels = state
                .ot_deluxe
                .inventory
                .spares_wheels
                .saturating_add(amount);
        }
        TradeGoodKind::Axle => {
            let amount = clamp_u8(good.amount);
            state.ot_deluxe.inventory.spares_axles = state
                .ot_deluxe
                .inventory
                .spares_axles
                .saturating_add(amount);
        }
        TradeGoodKind::Tongue => {
            let amount = clamp_u8(good.amount);
            state.ot_deluxe.inventory.spares_tongues = state
                .ot_deluxe
                .inventory
                .spares_tongues
                .saturating_add(amount);
        }
        TradeGoodKind::Food => {
            let amount = clamp_u16(good.amount);
            state.ot_deluxe.inventory.food_lbs =
                state.ot_deluxe.inventory.food_lbs.saturating_add(amount);
        }
        TradeGoodKind::Cash => {
            state.ot_deluxe.inventory.cash_cents = state
                .ot_deluxe
                .inventory
                .cash_cents
                .saturating_add(good.amount);
        }
    }
}

fn clamp_u16(value: u32) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

fn clamp_u8(value: u32) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otdeluxe_state::OtDeluxeInventory;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn stocked_state() -> GameState {
        let mut state = GameState::default();
        state.ot_deluxe.oxen.healthy = 6;
        state.ot_deluxe.inventory = OtDeluxeInventory {
            food_lbs: 100,
            bullets: 60,
            clothes_sets: 4,
            cash_cents: 5000,
            spares_wheels: 2,
            spares_axles: 2,
            spares_tongues: 2,
        };
        state
    }

    #[test]
    fn trade_offer_records_decision_traces() {
        let mut state = GameState::default();
        state.ot_deluxe.oxen.healthy = 2;
        state.ot_deluxe.inventory = OtDeluxeInventory {
            food_lbs: 50,
            bullets: 40,
            clothes_sets: 2,
            cash_cents: 500,
            spares_wheels: 1,
            spares_axles: 1,
            spares_tongues: 1,
        };

        let mut rng = SmallRng::seed_from_u64(7);
        let _ = resolve_trade_with_rng(&mut state, &mut rng);

        assert!(
            state
                .decision_traces_today
                .iter()
                .any(|trace| trace.pool_id == "otdeluxe.trade.give")
        );
        assert!(
            state
                .decision_traces_today
                .iter()
                .any(|trace| trace.pool_id == "otdeluxe.trade.receive")
        );
    }

    #[test]
    fn trade_returns_no_offer_when_no_goods_available() {
        let mut state = GameState::default();
        let mut rng = SmallRng::seed_from_u64(1);
        let outcome = resolve_trade_with_rng(&mut state, &mut rng);
        assert_eq!(outcome.resolution, TradeResolution::NoOffer);
        assert!(outcome.offer.is_none());

        let outcome = resolve_trade(&mut state);
        assert_eq!(outcome.resolution, TradeResolution::NoOffer);
        assert!(outcome.offer.is_none());
    }

    #[test]
    fn trade_amount_bounds_handle_singleton_goods() {
        let mut rng = SmallRng::seed_from_u64(11);
        let mut state = stocked_state();
        state.ot_deluxe.inventory.spares_wheels = 0;

        let give_amount = generate_give_amount(&mut rng, &state, TradeGoodKind::Wheel);
        let receive_amount = generate_receive_amount(&mut rng, TradeGoodKind::Axle);

        assert_eq!(amount_bounds(TradeGoodKind::Wheel), (1, 1));
        assert_eq!(give_amount, 1);
        assert_eq!(receive_amount, 1);
    }

    #[test]
    fn apply_offer_rejects_unaffordable_goods() {
        let mut state = GameState::default();
        state.ot_deluxe.inventory.food_lbs = 10;
        let offer = TradeOffer {
            give: TradeGood {
                kind: TradeGoodKind::Food,
                amount: 25,
            },
            receive: TradeGood {
                kind: TradeGoodKind::Cash,
                amount: 500,
            },
        };
        let outcome = apply_offer(&mut state, Some(offer));
        assert_eq!(outcome.resolution, TradeResolution::Unaffordable);
        assert_eq!(state.ot_deluxe.inventory.food_lbs, 10);
    }

    #[test]
    fn add_good_updates_each_inventory_slot() {
        let mut state = GameState::default();
        add_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Oxen,
                amount: 2,
            },
        );
        add_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Clothes,
                amount: 3,
            },
        );
        add_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Bullets,
                amount: 40,
            },
        );
        add_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Wheel,
                amount: 1,
            },
        );
        add_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Axle,
                amount: 1,
            },
        );
        add_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Tongue,
                amount: 1,
            },
        );
        add_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Food,
                amount: 25,
            },
        );
        add_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Cash,
                amount: 500,
            },
        );

        assert_eq!(state.ot_deluxe.oxen.healthy, 2);
        assert_eq!(state.ot_deluxe.inventory.clothes_sets, 3);
        assert_eq!(state.ot_deluxe.inventory.bullets, 40);
        assert_eq!(state.ot_deluxe.inventory.spares_wheels, 1);
        assert_eq!(state.ot_deluxe.inventory.spares_axles, 1);
        assert_eq!(state.ot_deluxe.inventory.spares_tongues, 1);
        assert_eq!(state.ot_deluxe.inventory.food_lbs, 25);
        assert_eq!(state.ot_deluxe.inventory.cash_cents, 500);
    }

    #[test]
    fn subtract_good_updates_each_inventory_slot() {
        let mut state = stocked_state();
        assert!(!subtract_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Food,
                amount: 0,
            }
        ));
        assert!(!subtract_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Food,
                amount: 1000,
            }
        ));

        assert!(subtract_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Oxen,
                amount: 2,
            }
        ));
        assert!(subtract_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Clothes,
                amount: 1,
            }
        ));
        assert!(subtract_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Bullets,
                amount: 20,
            }
        ));
        assert!(subtract_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Wheel,
                amount: 1,
            }
        ));
        assert!(subtract_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Axle,
                amount: 1,
            }
        ));
        assert!(subtract_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Tongue,
                amount: 1,
            }
        ));
        assert!(subtract_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Food,
                amount: 25,
            }
        ));
        assert!(subtract_good(
            &mut state,
            TradeGood {
                kind: TradeGoodKind::Cash,
                amount: 500,
            }
        ));

        assert_eq!(state.ot_deluxe.oxen.healthy, 4);
        assert_eq!(state.ot_deluxe.inventory.clothes_sets, 3);
        assert_eq!(state.ot_deluxe.inventory.bullets, 40);
        assert_eq!(state.ot_deluxe.inventory.spares_wheels, 1);
        assert_eq!(state.ot_deluxe.inventory.spares_axles, 1);
        assert_eq!(state.ot_deluxe.inventory.spares_tongues, 1);
        assert_eq!(state.ot_deluxe.inventory.food_lbs, 75);
        assert_eq!(state.ot_deluxe.inventory.cash_cents, 4500);
    }

    #[test]
    fn deterministic_offer_uses_minimum_amounts() {
        let mut state = GameState::default();
        state.ot_deluxe.oxen.healthy = 1;
        let offer = generate_offer_deterministic(&state).expect("offer expected");
        assert_eq!(offer.give.kind, TradeGoodKind::Oxen);
        assert_eq!(offer.give.amount, 1);
        assert_eq!(offer.receive.kind, TradeGoodKind::Clothes);
        assert_eq!(offer.receive.amount, 1);
    }
}
