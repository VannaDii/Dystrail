#[cfg(any(target_arch = "wasm32", test))]
use crate::a11y::set_status;
#[cfg(any(target_arch = "wasm32", test))]
use crate::game::store::StoreItem;
#[cfg(any(target_arch = "wasm32", test))]
use crate::i18n;
#[cfg(any(target_arch = "wasm32", test))]
use std::collections::BTreeMap;

#[cfg(any(target_arch = "wasm32", test))]
pub fn announce_quantity_change(
    item: &StoreItem,
    qty: i32,
    added: bool,
    state: &crate::components::ui::outfitting_store::state::StoreState,
    budget_cents: i64,
) {
    let item_name = i18n::t(&format!("store.items.{}.name", item.id));
    let effective_price =
        crate::game::store::calculate_effective_price(item.price_cents, state.discount_pct);
    let price_str = format_currency(if added {
        effective_price * i64::from(qty)
    } else {
        effective_price
    });
    let remaining = budget_cents - state.cart.total_cents;
    let remaining_str = format_currency(remaining);

    let message = if added {
        i18n::tr(
            "store.alerts.added",
            Some(&{
                let mut vars = BTreeMap::new();
                vars.insert("item", item_name.as_str());
                vars.insert("price", price_str.as_str());
                vars.insert("left", remaining_str.as_str());
                vars
            }),
        )
    } else {
        i18n::tr(
            "store.alerts.removed",
            Some(&{
                let mut vars = BTreeMap::new();
                vars.insert("item", item_name.as_str());
                vars.insert("left", remaining_str.as_str());
                vars
            }),
        )
    };

    set_status(&message);
}

#[cfg(any(target_arch = "wasm32", test))]
pub fn announce_cannot_add(item: &StoreItem) {
    let item_name = i18n::t(&format!("store.items.{}.name", item.id));
    let message = if item.unique {
        i18n::tr(
            "store.alerts.unique",
            Some(&{
                let mut vars = BTreeMap::new();
                vars.insert("item", item_name.as_str());
                vars
            }),
        )
    } else {
        i18n::tr(
            "store.alerts.max_qty",
            Some(&{
                let mut vars = BTreeMap::new();
                vars.insert("item", item_name.as_str());
                vars
            }),
        )
    };

    set_status(&message);
}

pub fn format_currency(cents: i64) -> String {
    crate::i18n::fmt_currency(cents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ui::outfitting_store::state::StoreState;
    use crate::game::store::{Grants, StoreItem};

    fn item(unique: bool) -> StoreItem {
        StoreItem {
            id: String::from("water"),
            name: String::from("Water"),
            desc: String::from("Desc"),
            price_cents: 100,
            unique,
            max_qty: 5,
            grants: Grants::default(),
            tags: Vec::new(),
            category: String::from("fuel_food"),
        }
    }

    #[test]
    fn announce_quantity_change_covers_branches() {
        crate::i18n::set_lang("en");
        let mut state = StoreState::default();
        state.cart.total_cents = 100;
        announce_quantity_change(&item(false), 1, true, &state, 500);
        announce_quantity_change(&item(false), 1, false, &state, 500);
        assert!(!format_currency(0).is_empty());
    }

    #[test]
    fn announce_cannot_add_covers_unique_and_max() {
        crate::i18n::set_lang("en");
        announce_cannot_add(&item(true));
        announce_cannot_add(&item(false));
    }
}
