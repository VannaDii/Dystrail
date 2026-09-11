use crate::game::store::{Cart, StoreItem, calculate_effective_price};
pub fn can_add_item(
    cart: &Cart,
    item: &StoreItem,
    qty_to_add: i32,
    budget_cents: i64,
    discount_pct: f64,
) -> bool {
    let current_qty = cart.get_quantity(&item.id);
    let new_qty = current_qty + qty_to_add;

    if new_qty > item.max_qty {
        return false;
    }

    if item.unique && new_qty > 1 {
        return false;
    }

    let effective_price = calculate_effective_price(item.price_cents, discount_pct);
    let additional_cost = effective_price * i64::from(qty_to_add);
    let new_total = cart.total_cents + additional_cost;

    new_total <= budget_cents
}
