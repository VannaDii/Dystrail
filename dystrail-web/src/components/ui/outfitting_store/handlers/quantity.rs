use crate::game::store::{Cart, StoreItem, calculate_effective_price};
pub fn maximum_quantity(
    cart: &Cart,
    item: &StoreItem,
    budget_cents: i64,
    discount_pct: f64,
) -> i32 {
    let effective_price = calculate_effective_price(item.price_cents, discount_pct);
    let current_cost = effective_price * i64::from(cart.get_quantity(&item.id));
    let available = (budget_cents - cart.total_cents + current_cost).max(0);
    let affordable = if effective_price > 0 {
        i32::try_from(available / effective_price).unwrap_or(i32::MAX)
    } else {
        item.max_qty
    };
    affordable
        .min(if item.unique { 1 } else { item.max_qty })
        .max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacing_a_quantity_reuses_its_budget_and_honors_stock_limits() {
        let store = super::super::super::state::load_store_data().expect("store data");
        let item = store.find_item("rations").expect("rations");
        let mut cart = Cart::new();
        cart.add_item("rations", 4);
        cart.add_item("water", 2);
        cart.total_cents = 2800;
        assert_eq!(maximum_quantity(&cart, item, 3000, 0.0), 4);
        assert_eq!(maximum_quantity(&cart, item, 4000, 0.0), 6);
        let pass = store.find_item("press_pass").expect("pass");
        assert_eq!(maximum_quantity(&cart, pass, 20000, 0.0), 1);
        assert_eq!(maximum_quantity(&cart, item, 0, 0.0), 0);
    }
}
