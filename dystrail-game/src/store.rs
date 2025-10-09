//! Store management and shopping cart
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
    pub category: String,
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
    pub items: Vec<StoreItem>,
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
    pub enabled: bool,
}

/// A line item in the shopping cart.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CartLine {
    pub item_id: String,
    pub item_name: String,
    pub quantity: i32,
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
            line.quantity += qty_to_add;
            line.qty
        } else {
            self.lines.push(CartLine {
                item_id: item_id.to_string(),
                item_name: item_id.to_string(), // Placeholder
                quantity: qty_to_add,
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
            line.quantity = line.qty;
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
        // Also check items list for backward compatibility
        self.items.iter().find(|item| item.id == item_id)
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
        // Also add items from items list
        for item in &self.items {
            map.insert(item.id.clone(), item);
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
