//! Store data structures and logic for the Outfitting Store.
//!
//! This module provides the data structures needed for the store
//! functionality, including items, categories, cart management,
//! and pricing calculations with persona discounts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single item available in the store.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StoreItem {
    pub id: String,
    pub name: String,
    pub desc: String,
    /// Price in cents to avoid floating-point issues
    pub price_cents: i64,
    /// Whether this item can only be purchased once
    pub unique: bool,
    /// Maximum quantity that can be purchased
    pub max_qty: i32,
    /// Stats and inventory grants when purchased
    pub grants: Grants,
    /// Tags applied to the player when purchased
    pub tags: Vec<String>,
}

/// Category of items in the store.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StoreCategory {
    pub id: String,
    pub name: String,
    pub items: Vec<StoreItem>,
}

/// Complete store data structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Store {
    pub categories: Vec<StoreCategory>,
}

/// Grants applied to the player when purchasing an item.
/// All fields default to 0 if not specified in JSON.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Grants {
    #[serde(default)]
    pub supplies: i32,
    #[serde(default)]
    pub credibility: i32,
    #[serde(default)]
    pub spare_tire: i32,
    #[serde(default)]
    pub spare_battery: i32,
    #[serde(default)]
    pub spare_alt: i32,
    #[serde(default)]
    pub spare_pump: i32,
}

/// A line item in the shopping cart.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CartLine {
    pub item_id: String,
    pub qty: i32,
}

/// Shopping cart state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cart {
    pub lines: Vec<CartLine>,
    /// Total cost in cents (updated when cart changes)
    pub total_cents: i64,
}

impl Cart {
    /// Create a new empty cart.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Find a cart line by item ID.
    #[must_use]
    pub fn find_line(&self, item_id: &str) -> Option<&CartLine> {
        self.lines.iter().find(|line| line.item_id == item_id)
    }

    /// Find a mutable cart line by item ID.
    pub fn find_line_mut(&mut self, item_id: &str) -> Option<&mut CartLine> {
        self.lines.iter_mut().find(|line| line.item_id == item_id)
    }

    /// Add quantity to an item in the cart.
    /// Returns the new quantity for that item.
    pub fn add_item(&mut self, item_id: &str, qty_to_add: i32) -> i32 {
        if let Some(line) = self.find_line_mut(item_id) {
            line.qty += qty_to_add;
            line.qty
        } else {
            self.lines.push(CartLine {
                item_id: item_id.to_string(),
                qty: qty_to_add,
            });
            qty_to_add
        }
    }

    /// Remove quantity from an item in the cart.
    /// Returns the new quantity (0 if line is removed).
    pub fn remove_item(&mut self, item_id: &str, qty_to_remove: i32) -> i32 {
        if let Some(line) = self.find_line_mut(item_id) {
            line.qty = (line.qty - qty_to_remove).max(0);
            if line.qty == 0 {
                self.lines.retain(|l| l.item_id != item_id);
                0
            } else {
                line.qty
            }
        } else {
            0
        }
    }

    /// Remove all of an item from the cart.
    pub fn remove_all_item(&mut self, item_id: &str) {
        self.lines.retain(|line| line.item_id != item_id);
    }

    /// Clear the entire cart.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.total_cents = 0;
    }

    /// Get the current quantity of an item in the cart.
    #[must_use]
    pub fn get_quantity(&self, item_id: &str) -> i32 {
        self.find_line(item_id).map_or(0, |line| line.qty)
    }

    /// Check if the cart is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

impl Store {
    /// Find an item by ID across all categories.
    #[must_use]
    pub fn find_item(&self, item_id: &str) -> Option<&StoreItem> {
        for category in &self.categories {
            for item in &category.items {
                if item.id == item_id {
                    return Some(item);
                }
            }
        }
        None
    }

    /// Get all items as a flat map by ID.
    #[must_use]
    pub fn items_by_id(&self) -> HashMap<String, &StoreItem> {
        let mut map = HashMap::new();
        for category in &self.categories {
            for item in &category.items {
                map.insert(item.id.clone(), item);
            }
        }
        map
    }
}

/// Calculate the effective price after persona discount.
/// Returns price in cents, rounded up.
#[must_use]
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
pub fn calculate_effective_price(base_price_cents: i64, discount_pct: f64) -> i64 {
    if discount_pct <= 0.0 {
        return base_price_cents;
    }

    let multiplier = 1.0 - (discount_pct / 100.0);
    let discounted = base_price_cents as f64 * multiplier;
    discounted.ceil() as i64
}

/// Calculate the total cost of a cart with persona discount applied.
#[must_use]
pub fn calculate_cart_total(cart: &Cart, store: &Store, discount_pct: f64) -> i64 {
    let mut total = 0i64;

    for line in &cart.lines {
        if let Some(item) = store.find_item(&line.item_id) {
            let effective_price = calculate_effective_price(item.price_cents, discount_pct);
            total += effective_price * i64::from(line.qty);
        }
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_effective_price() {
        // No discount
        assert_eq!(calculate_effective_price(1000, 0.0), 1000);

        // 10% discount on $10.00 = $9.00
        assert_eq!(calculate_effective_price(1000, 10.0), 900);

        // 10% discount on $10.01 should round up to $9.01
        assert_eq!(calculate_effective_price(1001, 10.0), 901);

        // Test edge case with larger discount
        assert_eq!(calculate_effective_price(2500, 20.0), 2000);

        // Test very small prices
        assert_eq!(calculate_effective_price(1, 10.0), 1); // ceil(0.9) = 1

        // Test negative discount (should be no-op)
        assert_eq!(calculate_effective_price(1000, -5.0), 1000);
    }

    #[test]
    fn test_cart_operations() {
        let mut cart = Cart::new();

        // Add items
        assert_eq!(cart.add_item("rations", 3), 3);
        assert_eq!(cart.add_item("water", 2), 2);
        assert_eq!(cart.add_item("rations", 1), 4); // Add to existing

        // Check quantities
        assert_eq!(cart.get_quantity("rations"), 4);
        assert_eq!(cart.get_quantity("water"), 2);
        assert_eq!(cart.get_quantity("nonexistent"), 0);

        // Remove items
        assert_eq!(cart.remove_item("rations", 2), 2);
        assert_eq!(cart.remove_item("water", 5), 0); // Remove more than exists

        // Check final state
        assert_eq!(cart.get_quantity("rations"), 2);
        assert_eq!(cart.get_quantity("water"), 0);
        assert_eq!(cart.lines.len(), 1); // water line should be removed

        // Test remove all
        cart.add_item("spare_tire", 3);
        cart.remove_all_item("spare_tire");
        assert_eq!(cart.get_quantity("spare_tire"), 0);

        // Clear cart
        cart.clear();
        assert!(cart.is_empty());
    }

    #[test]
    fn test_cart_total_calculation() {
        let store = Store {
            categories: vec![StoreCategory {
                id: "test".to_string(),
                name: "Test".to_string(),
                items: vec![
                    StoreItem {
                        id: "item1".to_string(),
                        name: "Item 1".to_string(),
                        desc: "Test item".to_string(),
                        price_cents: 1000, // $10.00
                        unique: false,
                        max_qty: 10,
                        grants: Grants::default(),
                        tags: vec![],
                    },
                    StoreItem {
                        id: "item2".to_string(),
                        name: "Item 2".to_string(),
                        desc: "Test item 2".to_string(),
                        price_cents: 500, // $5.00
                        unique: false,
                        max_qty: 10,
                        grants: Grants::default(),
                        tags: vec![],
                    },
                ],
            }],
        };

        let mut cart = Cart::new();
        cart.add_item("item1", 2); // 2 × $10.00 = $20.00
        cart.add_item("item2", 3); // 3 × $5.00 = $15.00
        // Total: $35.00 = 3500 cents

        // No discount
        assert_eq!(calculate_cart_total(&cart, &store, 0.0), 3500);

        // 10% discount: $31.50 = 3150 cents
        assert_eq!(calculate_cart_total(&cart, &store, 10.0), 3150);
    }

    #[test]
    fn test_unique_items() {
        let _unique_item = StoreItem {
            id: "press_pass".to_string(),
            name: "Press Pass".to_string(),
            desc: "Unique item".to_string(),
            price_cents: 1800,
            unique: true,
            max_qty: 1,
            grants: Grants::default(),
            tags: vec!["permit".to_string()],
        };

        let mut cart = Cart::new();
        cart.add_item("press_pass", 1);

        // Should only have 1 press pass
        assert_eq!(cart.get_quantity("press_pass"), 1);

        // Adding more should increase the count in cart (validation happens in UI)
        cart.add_item("press_pass", 1);
        assert_eq!(cart.get_quantity("press_pass"), 2);
    }

    #[test]
    fn test_grants_aggregation() {
        let grants = Grants {
            supplies: 3,
            credibility: 1,
            spare_tire: 2,
            spare_battery: 0,
            spare_alt: 1,
            spare_pump: 0,
        };

        // Test that grants contain expected values
        assert_eq!(grants.supplies, 3);
        assert_eq!(grants.credibility, 1);
        assert_eq!(grants.spare_tire, 2);
        assert_eq!(grants.spare_alt, 1);
    }

    #[test]
    fn test_store_item_find() {
        let store = Store {
            categories: vec![
                StoreCategory {
                    id: "cat1".to_string(),
                    name: "Category 1".to_string(),
                    items: vec![
                        StoreItem {
                            id: "item1".to_string(),
                            name: "Item 1".to_string(),
                            desc: "Test".to_string(),
                            price_cents: 1000,
                            unique: false,
                            max_qty: 10,
                            grants: Grants::default(),
                            tags: vec![],
                        },
                    ],
                },
                StoreCategory {
                    id: "cat2".to_string(),
                    name: "Category 2".to_string(),
                    items: vec![
                        StoreItem {
                            id: "item2".to_string(),
                            name: "Item 2".to_string(),
                            desc: "Test".to_string(),
                            price_cents: 500,
                            unique: false,
                            max_qty: 5,
                            grants: Grants::default(),
                            tags: vec![],
                        },
                    ],
                },
            ],
        };

        // Test finding items across categories
        assert!(store.find_item("item1").is_some());
        assert!(store.find_item("item2").is_some());
        assert!(store.find_item("nonexistent").is_none());

        // Test items_by_id map
        let items_map = store.items_by_id();
        assert_eq!(items_map.len(), 2);
        assert!(items_map.contains_key("item1"));
        assert!(items_map.contains_key("item2"));
    }

    #[test]
    fn test_budget_edge_cases() {
        // Test that very large budget calculations don't overflow
        let large_price = 1_000_000_000; // $10 million in cents

        // Should be able to handle large calculations
        let discounted = calculate_effective_price(large_price, 10.0);
        assert_eq!(discounted, 900_000_000);

        // Test zero price
        assert_eq!(calculate_effective_price(0, 50.0), 0);
    }

    #[test]
    fn test_cart_equality() {
        let mut cart1 = Cart::new();
        let mut cart2 = Cart::new();

        // Empty carts should be equal
        assert_eq!(cart1, cart2);

        // Add same items to both
        cart1.add_item("item1", 3);
        cart2.add_item("item1", 3);

        // Should still be equal (ignoring total_cents for now)
        cart1.total_cents = 1000;
        cart2.total_cents = 1000;
        assert_eq!(cart1, cart2);

        // Different quantities should not be equal
        cart2.add_item("item1", 1);
        assert_ne!(cart1, cart2);
    }
}