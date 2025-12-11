use super::super::super::super::handlers::{can_add_item, format_currency};
use super::super::super::super::state::StoreState;
use crate::game::GameState;
use crate::game::store::StoreItem;
use crate::i18n;

#[derive(Debug, Clone)]
pub struct QuantityOption {
    pub idx: u8,
    pub label: String,
    pub preview: String,
}

pub fn build_quantity_options(
    item: &StoreItem,
    state: &StoreState,
    game_state: &GameState,
    effective_price: i64,
    current_qty: i32,
) -> Vec<QuantityOption> {
    let mut options = Vec::new();

    let can_add_1 = can_add_item(
        &state.cart,
        item,
        1,
        game_state.budget_cents,
        state.discount_pct,
    );
    options.push(if can_add_1 {
        let budget_preview = game_state.budget_cents - state.cart.total_cents - effective_price;
        let preview_str = format!(
            "[{}{}]",
            i18n::t("store.budget"),
            format_currency(budget_preview)
        );
        QuantityOption {
            idx: 1,
            label: i18n::t("store.qty_prompt.add1"),
            preview: preview_str,
        }
    } else {
        QuantityOption {
            idx: 1,
            label: i18n::t("store.qty_prompt.add1"),
            preview: String::from("[Max/Budget]"),
        }
    });

    if !item.unique {
        let can_add_5 = can_add_item(
            &state.cart,
            item,
            5,
            game_state.budget_cents,
            state.discount_pct,
        );
        let add5 = if can_add_5 {
            let budget_preview =
                game_state.budget_cents - state.cart.total_cents - (effective_price * 5);
            let preview_str = format!(
                "[{}{}]",
                i18n::t("store.budget"),
                format_currency(budget_preview)
            );
            QuantityOption {
                idx: 2,
                label: i18n::t("store.qty_prompt.add5"),
                preview: preview_str,
            }
        } else {
            QuantityOption {
                idx: 2,
                label: i18n::t("store.qty_prompt.add5"),
                preview: String::from("[Max/Budget]"),
            }
        };
        options.push(add5);
    }

    if current_qty > 0 {
        options.push(QuantityOption {
            idx: 3,
            label: i18n::t("store.qty_prompt.rem1"),
            preview: String::new(),
        });
        options.push(QuantityOption {
            idx: 4,
            label: i18n::t("store.qty_prompt.rem_all"),
            preview: String::new(),
        });
    }

    options.push(QuantityOption {
        idx: 0,
        label: i18n::t("store.menu.back"),
        preview: String::new(),
    });

    options
}
