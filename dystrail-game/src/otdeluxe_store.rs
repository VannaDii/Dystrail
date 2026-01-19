//! Store pricing and purchase logic for Oregon Trail Deluxe parity.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::mechanics::otdeluxe90s::OtDeluxeStorePolicy;
use crate::otdeluxe_state::{OtDeluxeInventory, OtDeluxeOxenState};
use crate::otdeluxe_trail::price_multiplier_pct_for_node;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxeStoreItem {
    Oxen,
    ClothesSet,
    AmmoBox,
    FoodLb,
    Wheel,
    Axle,
    Tongue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeStoreLineItem {
    pub item: OtDeluxeStoreItem,
    pub quantity: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtDeluxeStoreReceipt {
    pub total_cost_cents: u32,
    pub lines: Vec<OtDeluxeStoreLineItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OtDeluxeStoreError {
    ExceedsCap {
        item: OtDeluxeStoreItem,
        requested: u16,
        remaining: u16,
    },
    InsufficientCash {
        required_cents: u32,
        available_cents: u32,
    },
}

#[must_use]
pub const fn base_price_cents(policy: &OtDeluxeStorePolicy, item: OtDeluxeStoreItem) -> u32 {
    match item {
        OtDeluxeStoreItem::Oxen => policy.base_prices_cents.ox,
        OtDeluxeStoreItem::ClothesSet => policy.base_prices_cents.clothes_set,
        OtDeluxeStoreItem::AmmoBox => policy.base_prices_cents.ammo_box,
        OtDeluxeStoreItem::FoodLb => policy.base_prices_cents.food_lb,
        OtDeluxeStoreItem::Wheel => policy.base_prices_cents.wheel,
        OtDeluxeStoreItem::Axle => policy.base_prices_cents.axle,
        OtDeluxeStoreItem::Tongue => policy.base_prices_cents.tongue,
    }
}

#[must_use]
pub const fn max_inventory(policy: &OtDeluxeStorePolicy, item: OtDeluxeStoreItem) -> u16 {
    match item {
        OtDeluxeStoreItem::Oxen => policy.max_buy.oxen,
        OtDeluxeStoreItem::ClothesSet => policy.max_buy.clothes_sets,
        OtDeluxeStoreItem::AmmoBox => policy.max_buy.ammo_boxes,
        OtDeluxeStoreItem::FoodLb => policy.max_buy.food_lbs,
        OtDeluxeStoreItem::Wheel => policy.max_buy.wheels,
        OtDeluxeStoreItem::Axle => policy.max_buy.axles,
        OtDeluxeStoreItem::Tongue => policy.max_buy.tongues,
    }
}

#[must_use]
pub fn price_cents_at_node(
    policy: &OtDeluxeStorePolicy,
    item: OtDeluxeStoreItem,
    node_index: u8,
) -> u32 {
    let base = base_price_cents(policy, item);
    let multiplier = price_multiplier_pct_for_node(policy, node_index);
    let scaled = u64::from(base).saturating_mul(u64::from(multiplier)) / 100_u64;
    u32::try_from(scaled).unwrap_or(u32::MAX)
}

fn current_inventory(
    policy: &OtDeluxeStorePolicy,
    inventory: &OtDeluxeInventory,
    oxen: OtDeluxeOxenState,
    item: OtDeluxeStoreItem,
) -> u16 {
    match item {
        OtDeluxeStoreItem::Oxen => oxen.total(),
        OtDeluxeStoreItem::ClothesSet => inventory.clothes_sets,
        OtDeluxeStoreItem::AmmoBox => {
            let per_box = policy.bullets_per_box.max(1);
            let bullets = u32::from(inventory.bullets);
            let per_box_u32 = u32::from(per_box);
            let boxes = bullets.saturating_add(per_box_u32.saturating_sub(1)) / per_box_u32;
            u16::try_from(boxes).unwrap_or(u16::MAX)
        }
        OtDeluxeStoreItem::FoodLb => inventory.food_lbs,
        OtDeluxeStoreItem::Wheel => inventory.spares_wheels.into(),
        OtDeluxeStoreItem::Axle => inventory.spares_axles.into(),
        OtDeluxeStoreItem::Tongue => inventory.spares_tongues.into(),
    }
}

fn remaining_capacity(
    policy: &OtDeluxeStorePolicy,
    inventory: &OtDeluxeInventory,
    oxen: OtDeluxeOxenState,
    item: OtDeluxeStoreItem,
) -> u16 {
    let current = current_inventory(policy, inventory, oxen, item);
    max_inventory(policy, item).saturating_sub(current)
}

/// Quote an `OTDeluxe` store purchase without mutating state.
///
/// # Errors
///
/// Returns an error when any line exceeds the remaining per-item capacity.
pub fn quote_purchase(
    policy: &OtDeluxeStorePolicy,
    node_index: u8,
    inventory: &OtDeluxeInventory,
    oxen: OtDeluxeOxenState,
    lines: &[OtDeluxeStoreLineItem],
) -> Result<OtDeluxeStoreReceipt, OtDeluxeStoreError> {
    let mut aggregated: BTreeMap<OtDeluxeStoreItem, u16> = BTreeMap::new();
    for line in lines {
        if line.quantity == 0 {
            continue;
        }
        let entry = aggregated.entry(line.item).or_insert(0);
        *entry = entry.saturating_add(line.quantity);
    }

    let mut total_cost: u64 = 0;
    let mut receipt_lines = Vec::with_capacity(aggregated.len());

    for (item, quantity) in aggregated {
        let remaining = remaining_capacity(policy, inventory, oxen, item);
        if quantity > remaining {
            return Err(OtDeluxeStoreError::ExceedsCap {
                item,
                requested: quantity,
                remaining,
            });
        }
        let price = price_cents_at_node(policy, item, node_index);
        total_cost =
            total_cost.saturating_add(u64::from(price).saturating_mul(u64::from(quantity)));
        receipt_lines.push(OtDeluxeStoreLineItem { item, quantity });
    }

    let total_cost_cents = u32::try_from(total_cost).unwrap_or(u32::MAX);
    Ok(OtDeluxeStoreReceipt {
        total_cost_cents,
        lines: receipt_lines,
    })
}

/// Apply a validated `OTDeluxe` store purchase to inventory.
///
/// # Errors
///
/// Returns an error when any line exceeds capacity or cash is insufficient.
pub fn apply_purchase(
    policy: &OtDeluxeStorePolicy,
    node_index: u8,
    inventory: &mut OtDeluxeInventory,
    oxen: &mut OtDeluxeOxenState,
    lines: &[OtDeluxeStoreLineItem],
) -> Result<OtDeluxeStoreReceipt, OtDeluxeStoreError> {
    let receipt = quote_purchase(policy, node_index, inventory, *oxen, lines)?;
    let available = inventory.cash_cents;
    if receipt.total_cost_cents > available {
        return Err(OtDeluxeStoreError::InsufficientCash {
            required_cents: receipt.total_cost_cents,
            available_cents: available,
        });
    }
    inventory.cash_cents = available.saturating_sub(receipt.total_cost_cents);

    let bullets_per_box = policy.bullets_per_box.max(1);
    let max_bullets =
        u32::from(policy.max_buy.ammo_boxes).saturating_mul(u32::from(bullets_per_box));

    for line in &receipt.lines {
        match line.item {
            OtDeluxeStoreItem::Oxen => {
                let cap = max_inventory(policy, line.item);
                oxen.healthy = oxen.healthy.saturating_add(line.quantity).min(cap);
            }
            OtDeluxeStoreItem::ClothesSet => {
                let cap = max_inventory(policy, line.item);
                inventory.clothes_sets = inventory
                    .clothes_sets
                    .saturating_add(line.quantity)
                    .min(cap);
            }
            OtDeluxeStoreItem::AmmoBox => {
                let added_bullets =
                    u32::from(line.quantity).saturating_mul(u32::from(bullets_per_box));
                let current_bullets = u32::from(inventory.bullets);
                let updated = current_bullets
                    .saturating_add(added_bullets)
                    .min(max_bullets);
                inventory.bullets = u16::try_from(updated).unwrap_or(u16::MAX);
            }
            OtDeluxeStoreItem::FoodLb => {
                let cap = max_inventory(policy, line.item);
                inventory.food_lbs = inventory.food_lbs.saturating_add(line.quantity).min(cap);
            }
            OtDeluxeStoreItem::Wheel => {
                let cap = max_inventory(policy, line.item);
                let added = u8::try_from(line.quantity.min(u16::from(u8::MAX))).unwrap_or(u8::MAX);
                inventory.spares_wheels = inventory
                    .spares_wheels
                    .saturating_add(added)
                    .min(u8::try_from(cap.min(u16::from(u8::MAX))).unwrap_or(u8::MAX));
            }
            OtDeluxeStoreItem::Axle => {
                let cap = max_inventory(policy, line.item);
                let added = u8::try_from(line.quantity.min(u16::from(u8::MAX))).unwrap_or(u8::MAX);
                inventory.spares_axles = inventory
                    .spares_axles
                    .saturating_add(added)
                    .min(u8::try_from(cap.min(u16::from(u8::MAX))).unwrap_or(u8::MAX));
            }
            OtDeluxeStoreItem::Tongue => {
                let cap = max_inventory(policy, line.item);
                let added = u8::try_from(line.quantity.min(u16::from(u8::MAX))).unwrap_or(u8::MAX);
                inventory.spares_tongues = inventory
                    .spares_tongues
                    .saturating_add(added)
                    .min(u8::try_from(cap.min(u16::from(u8::MAX))).unwrap_or(u8::MAX));
            }
        }
    }

    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::otdeluxe90s::OtDeluxe90sPolicy;

    #[test]
    fn price_respects_node_multiplier() {
        let policy = OtDeluxe90sPolicy::default();
        let store = &policy.store;
        let base = base_price_cents(store, OtDeluxeStoreItem::Oxen);
        assert_eq!(base, 2000);
        let price_start = price_cents_at_node(store, OtDeluxeStoreItem::Oxen, 0);
        let price_late = price_cents_at_node(store, OtDeluxeStoreItem::Oxen, 15);
        assert_eq!(price_start, 2000);
        assert_eq!(price_late, 5000);
    }

    #[test]
    fn purchase_enforces_caps_and_cash() {
        let policy = OtDeluxe90sPolicy::default();
        let store = &policy.store;
        let mut inventory = OtDeluxeInventory {
            cash_cents: 10_000,
            ..OtDeluxeInventory::default()
        };
        let mut oxen = OtDeluxeOxenState::default();

        let lines = [OtDeluxeStoreLineItem {
            item: OtDeluxeStoreItem::Oxen,
            quantity: 5,
        }];
        let receipt =
            apply_purchase(store, 0, &mut inventory, &mut oxen, &lines).expect("purchase succeeds");
        assert_eq!(receipt.total_cost_cents, 10_000);
        assert_eq!(oxen.healthy, 5);
        assert_eq!(inventory.cash_cents, 0);

        let too_many = [OtDeluxeStoreLineItem {
            item: OtDeluxeStoreItem::Oxen,
            quantity: 30,
        }];
        let err = quote_purchase(store, 0, &inventory, oxen, &too_many).expect_err("cap enforced");
        match err {
            OtDeluxeStoreError::ExceedsCap {
                item, remaining, ..
            } => {
                assert_eq!(item, OtDeluxeStoreItem::Oxen);
                assert_eq!(remaining, 15);
            }
            OtDeluxeStoreError::InsufficientCash { .. } => {
                panic!("unexpected error")
            }
        }
    }

    #[test]
    fn ammo_boxes_convert_to_bullets() {
        let policy = OtDeluxe90sPolicy::default();
        let store = &policy.store;
        let mut inventory = OtDeluxeInventory {
            cash_cents: 1000,
            bullets: 0,
            ..OtDeluxeInventory::default()
        };
        let mut oxen = OtDeluxeOxenState::default();

        let lines = [OtDeluxeStoreLineItem {
            item: OtDeluxeStoreItem::AmmoBox,
            quantity: 2,
        }];
        let receipt =
            apply_purchase(store, 0, &mut inventory, &mut oxen, &lines).expect("purchase succeeds");
        assert_eq!(receipt.total_cost_cents, 400);
        assert_eq!(inventory.bullets, 40);
    }

    #[test]
    fn ammo_box_cap_accounts_for_partial_boxes() {
        let policy = OtDeluxe90sPolicy::default();
        let store = &policy.store;
        let inventory = OtDeluxeInventory {
            bullets: 15,
            ..OtDeluxeInventory::default()
        };
        let oxen = OtDeluxeOxenState::default();

        let lines = [OtDeluxeStoreLineItem {
            item: OtDeluxeStoreItem::AmmoBox,
            quantity: 50,
        }];
        let err = quote_purchase(store, 0, &inventory, oxen, &lines).expect_err("cap enforced");
        match err {
            OtDeluxeStoreError::ExceedsCap {
                item, remaining, ..
            } => {
                assert_eq!(item, OtDeluxeStoreItem::AmmoBox);
                assert_eq!(remaining, 49);
            }
            OtDeluxeStoreError::InsufficientCash { .. } => {
                panic!("unexpected error")
            }
        }
    }

    #[test]
    fn quote_purchase_aggregates_and_ignores_zero_lines() {
        let policy = OtDeluxe90sPolicy::default();
        let store = &policy.store;
        let inventory = OtDeluxeInventory {
            cash_cents: 10_000,
            ..OtDeluxeInventory::default()
        };
        let oxen = OtDeluxeOxenState::default();

        let lines = [
            OtDeluxeStoreLineItem {
                item: OtDeluxeStoreItem::ClothesSet,
                quantity: 2,
            },
            OtDeluxeStoreLineItem {
                item: OtDeluxeStoreItem::ClothesSet,
                quantity: 1,
            },
            OtDeluxeStoreLineItem {
                item: OtDeluxeStoreItem::FoodLb,
                quantity: 0,
            },
        ];
        let receipt = quote_purchase(store, 0, &inventory, oxen, &lines).expect("quote");
        assert_eq!(receipt.lines.len(), 1);
        assert_eq!(
            receipt.lines[0],
            OtDeluxeStoreLineItem {
                item: OtDeluxeStoreItem::ClothesSet,
                quantity: 3,
            }
        );
        let expected = price_cents_at_node(store, OtDeluxeStoreItem::ClothesSet, 0) * 3;
        assert_eq!(receipt.total_cost_cents, expected);
    }

    #[test]
    fn apply_purchase_rejects_when_cash_insufficient() {
        let policy = OtDeluxe90sPolicy::default();
        let store = &policy.store;
        let mut inventory = OtDeluxeInventory {
            cash_cents: 100,
            ..OtDeluxeInventory::default()
        };
        let mut oxen = OtDeluxeOxenState::default();

        let lines = [OtDeluxeStoreLineItem {
            item: OtDeluxeStoreItem::Oxen,
            quantity: 1,
        }];
        let err = apply_purchase(store, 0, &mut inventory, &mut oxen, &lines)
            .expect_err("expected cash error");
        match err {
            OtDeluxeStoreError::InsufficientCash {
                required_cents,
                available_cents,
            } => {
                assert!(required_cents > available_cents);
                assert_eq!(available_cents, 100);
            }
            OtDeluxeStoreError::ExceedsCap { .. } => panic!("unexpected error"),
        }
    }

    #[test]
    fn apply_purchase_updates_all_item_kinds() {
        let policy = OtDeluxe90sPolicy::default();
        let store = &policy.store;
        let mut inventory = OtDeluxeInventory {
            cash_cents: 30_000,
            bullets: 10,
            clothes_sets: 1,
            food_lbs: 10,
            ..OtDeluxeInventory::default()
        };
        let mut oxen = OtDeluxeOxenState {
            healthy: 2,
            sick: 0,
        };

        let lines = [
            OtDeluxeStoreLineItem {
                item: OtDeluxeStoreItem::Oxen,
                quantity: 2,
            },
            OtDeluxeStoreLineItem {
                item: OtDeluxeStoreItem::ClothesSet,
                quantity: 1,
            },
            OtDeluxeStoreLineItem {
                item: OtDeluxeStoreItem::AmmoBox,
                quantity: 1,
            },
            OtDeluxeStoreLineItem {
                item: OtDeluxeStoreItem::FoodLb,
                quantity: 25,
            },
            OtDeluxeStoreLineItem {
                item: OtDeluxeStoreItem::Wheel,
                quantity: 1,
            },
            OtDeluxeStoreLineItem {
                item: OtDeluxeStoreItem::Axle,
                quantity: 1,
            },
            OtDeluxeStoreLineItem {
                item: OtDeluxeStoreItem::Tongue,
                quantity: 1,
            },
        ];

        let receipt =
            apply_purchase(store, 0, &mut inventory, &mut oxen, &lines).expect("purchase succeeds");

        let expected_cost = lines.iter().fold(0_u32, |acc, line| {
            acc.saturating_add(
                price_cents_at_node(store, line.item, 0).saturating_mul(u32::from(line.quantity)),
            )
        });
        assert_eq!(receipt.total_cost_cents, expected_cost);
        assert_eq!(oxen.healthy, 4);
        assert_eq!(inventory.clothes_sets, 2);
        assert_eq!(inventory.bullets, 30);
        assert_eq!(inventory.food_lbs, 35);
        assert_eq!(inventory.spares_wheels, 1);
        assert_eq!(inventory.spares_axles, 1);
        assert_eq!(inventory.spares_tongues, 1);
    }
}
