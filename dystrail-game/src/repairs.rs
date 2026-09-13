//! Explicit roadside decisions. No resources leave the van before a valid choice.
use crate::{GameState, vehicle::Part};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepairChoice {
    Onboard,
    Purchase,
    Barter,
    Radio,
}
impl RepairChoice {
    pub const ALL: [Self; 4] = [Self::Onboard, Self::Purchase, Self::Barter, Self::Radio];
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Onboard => "trail.repair_onboard",
            Self::Purchase => "trail.repair_purchase",
            Self::Barter => "trail.repair_barter",
            Self::Radio => "trail.repair_radio",
        }
    }
    #[must_use]
    pub const fn minutes(self) -> u16 {
        match self {
            Self::Onboard => 60,
            Self::Purchase => 90,
            Self::Barter => 120,
            Self::Radio => 240,
        }
    }
}
impl GameState {
    #[must_use]
    pub const fn spare_count(&self, part: Part) -> i32 {
        match part {
            Part::Tire => self.inventory.spares.tire,
            Part::Battery => self.inventory.spares.battery,
            Part::Alternator => self.inventory.spares.alt,
            Part::FuelPump => self.inventory.spares.pump,
        }
    }
    /// Same shelf prices as outfitting, plus a visible $10 delivery fee outside town.
    #[must_use]
    pub fn replacement_cost(&self, part: Part) -> i64 {
        let price = match part {
            Part::Tire => 1500,
            Part::Battery => 2200,
            Part::Alternator => 2100,
            Part::FuelPump => 1700,
        };
        crate::store::calculate_effective_price(price, f64::from(self.mods.store_discount_pct))
            + if self.continuity.route_services.stop.is_some() {
                0
            } else {
                1000
            }
    }
    #[must_use]
    pub fn can_repair(&self, choice: RepairChoice) -> bool {
        let Some(b) = &self.breakdown else {
            return false;
        };
        if self.ending.is_some() || self.continuity.abandoned {
            return false;
        }
        match choice {
            RepairChoice::Onboard => self.spare_count(b.part) > 0,
            RepairChoice::Purchase => self.budget_cents >= self.replacement_cost(b.part),
            RepairChoice::Barter => self.stats.supplies >= 4,
            RepairChoice::Radio => true,
        }
    }
    /// Install the named replacement or arrange a local work exchange; never roll a free fix.
    pub fn choose_repair(&mut self, choice: RepairChoice) -> bool {
        if !self.can_repair(choice) {
            return false;
        }
        let Some(b) = self.breakdown.clone() else {
            return false;
        };
        match choice {
            RepairChoice::Onboard => {
                self.consume_spare_for_part(b.part);
            }
            RepairChoice::Purchase => {
                let cost = self.replacement_cost(b.part);
                self.budget_cents -= cost;
                self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
                self.repairs_spent_cents += cost;
            }
            RepairChoice::Barter => self.stats.supplies -= 4,
            RepairChoice::Radio => {
                self.stats.sanity = (self.stats.sanity - 2).max(0);
                self.stats.morale = (self.stats.morale - 1).max(0);
            }
        }
        let repair_amount = if choice == RepairChoice::Radio {
            3.0
        } else {
            8.0
        };
        self.vehicle.repair(repair_amount);
        self.vehicle.set_wear(self.vehicle.wear - repair_amount);
        self.vehicle.set_breakdown_cooldown(2);
        self.add_day_reason_tag("repair");
        self.breakdown = None;
        self.last_breakdown_part = None;
        self.day_state.travel.travel_blocked = false;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vehicle::Breakdown;

    #[test]
    fn a_paid_or_fitted_repair_reduces_the_wear_that_causes_repeat_breakdowns() {
        for choice in RepairChoice::ALL {
            let mut gs = GameState::default();
            gs.vehicle.health = 80.0;
            gs.vehicle.wear = 20.0;
            gs.budget_cents = 10_000;
            gs.stats.supplies = 10;
            gs.inventory.spares.battery = 1;
            gs.breakdown = Some(Breakdown {
                part: Part::Battery,
                day_started: 1,
            });
            assert!(gs.choose_repair(choice));
            let amount = if choice == RepairChoice::Radio {
                3.0
            } else {
                8.0
            };
            assert!((gs.vehicle.health - (80.0 + amount)).abs() < f32::EPSILON);
            assert!((gs.vehicle.wear - (20.0 - amount)).abs() < f32::EPSILON);
            let repaired = serde_json::to_value(&gs).unwrap();
            assert!(!gs.choose_repair(choice));
            assert_eq!(serde_json::to_value(&gs).unwrap(), repaired);
        }
    }

    #[test]
    fn pending_breakdowns_block_all_automatic_recovery_before_a_day_starts() {
        for mode in [crate::GameMode::Classic, crate::GameMode::Deep] {
            for policy in [
                crate::state::PolicyKind::Balanced,
                crate::state::PolicyKind::Aggressive,
            ] {
                for miles in [100.0, 1_920.0] {
                    for part in [Part::Tire, Part::Battery, Part::Alternator, Part::FuelPump] {
                        let mut gs = GameState {
                            mode,
                            policy: Some(policy),
                            miles_traveled_actual: miles,
                            budget_cents: 10_000,
                            breakdown: Some(Breakdown {
                                part,
                                day_started: 1,
                            }),
                            ..GameState::default()
                        };
                        gs.continuity.interactive_repairs = true;
                        gs.vehicle.health = 0.0;
                        gs.inventory.spares = crate::state::Spares {
                            tire: 2,
                            battery: 2,
                            alt: 2,
                            pump: 2,
                        };
                        gs.day_state.travel.travel_blocked = true;
                        let before = serde_json::to_value(&gs).unwrap();
                        for _ in 0..3 {
                            let (ended, _, started) = gs.travel_next_leg(
                                &crate::endgame::EndgameTravelCfg::default_config(),
                            );
                            assert!(!ended && !started);
                            assert_eq!(serde_json::to_value(&gs).unwrap(), before);
                        }
                        assert!(gs.choose_repair(RepairChoice::Onboard));
                        assert_eq!(gs.spare_count(part), 1);
                        assert!(gs.breakdown.is_none());
                        assert_eq!(gs.budget_cents, 10_000);
                    }
                }
            }
        }
    }

    #[test]
    fn destroyed_vehicle_does_not_silently_spend_parts_or_cash() {
        for mode in [crate::GameMode::Classic, crate::GameMode::Deep] {
            let mut gs = GameState {
                mode,
                policy: Some(crate::state::PolicyKind::Balanced),
                budget_cents: 10_000,
                ..GameState::default()
            };
            gs.continuity.interactive_repairs = true;
            gs.vehicle.health = 0.0;
            gs.inventory.spares.tire = 2;
            let (ended, _, _) =
                gs.travel_next_leg(&crate::endgame::EndgameTravelCfg::default_config());
            assert!(ended);
            assert!(gs.ending.is_some());
            assert_eq!(gs.budget_cents, 10_000);
            assert_eq!(gs.inventory.spares.tire, 2);
            assert!(gs.vehicle.health.abs() < f32::EPSILON);
            assert_eq!(gs.repairs_spent_cents, 0);
        }
    }

    #[test]
    fn interactive_travel_cannot_silently_install_an_available_part() {
        let mut gs = GameState {
            breakdown: Some(Breakdown {
                part: Part::FuelPump,
                day_started: 1,
            }),
            ..GameState::default()
        };
        gs.inventory.spares.pump = 1;
        gs.continuity.interactive_repairs = true;
        let cash = gs.budget_cents;
        let _ = gs.travel_next_leg(&crate::endgame::EndgameTravelCfg::default());
        assert!(gs.breakdown.is_some());
        assert_eq!(gs.inventory.spares.pump, 1);
        assert_eq!(gs.budget_cents, cash);
        assert!(gs.choose_repair(RepairChoice::Onboard));
        assert_eq!(gs.inventory.spares.pump, 0);
        assert!(gs.breakdown.is_none());
    }
    #[test]
    fn purchase_uses_the_store_price_and_only_charges_delivery_outside_town() {
        let mut gs = GameState::default();
        for (part, price) in [
            (Part::Tire, 1500),
            (Part::Battery, 2200),
            (Part::Alternator, 2100),
            (Part::FuelPump, 1700),
        ] {
            gs.continuity.route_services.stop = None;
            assert_eq!(gs.replacement_cost(part), price + 1000);
            gs.continuity.route_services.stop = Some(160);
            assert_eq!(gs.replacement_cost(part), price);
        }
    }
    #[test]
    fn repairs_are_costed_atomic_and_allow_a_cashless_exit() {
        let mut gs = GameState {
            breakdown: Some(Breakdown {
                part: Part::Battery,
                day_started: 1,
            }),
            ..GameState::default()
        };
        gs.stats.supplies = 0;
        gs.budget_cents = 0;
        assert!(!gs.choose_repair(RepairChoice::Onboard));
        assert!(!gs.choose_repair(RepairChoice::Purchase));
        assert!(!gs.choose_repair(RepairChoice::Barter));
        assert!(gs.breakdown.is_some());
        let sanity = gs.stats.sanity;
        assert!(gs.choose_repair(RepairChoice::Radio));
        assert_eq!(gs.stats.sanity, sanity - 2);
        assert!(gs.breakdown.is_none());
        assert_eq!(gs.days_with_repair, 1);
        assert!(!gs.choose_repair(RepairChoice::Radio));
        assert_eq!(gs.days_with_repair, 1);
        gs.breakdown = Some(Breakdown {
            part: Part::FuelPump,
            day_started: 1,
        });
        gs.budget_cents = 2700;
        assert!(gs.choose_repair(RepairChoice::Purchase));
        assert_eq!(gs.budget_cents, 0);
        assert_eq!(gs.days_with_repair, 1, "two repairs on one day count once");
    }
}
