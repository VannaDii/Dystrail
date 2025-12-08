use crate::a11y::set_status;
use crate::game::store::StoreItem;
use crate::i18n;
use std::collections::BTreeMap;

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
