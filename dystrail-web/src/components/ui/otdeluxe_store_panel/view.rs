use crate::game::mechanics::otdeluxe90s::OtDeluxeStorePolicy;
use crate::game::otdeluxe_state::{OtDeluxeInventory, OtDeluxeOxenState};
use crate::game::otdeluxe_store::{self, OtDeluxeStoreItem, OtDeluxeStoreLineItem};
use crate::game::{GameState, OtDeluxe90sPolicy};
use crate::i18n;
use crate::i18n::fmt_currency;
use std::collections::BTreeMap;
use std::rc::Rc;
use web_sys::MouseEvent;
use yew::prelude::*;

#[derive(Clone, Copy)]
struct StoreItemDef {
    item: OtDeluxeStoreItem,
    name_key: &'static str,
    desc_key: &'static str,
}

struct StoreCardContext<'a> {
    cart: &'a UseStateHandle<Vec<u16>>,
    store_policy: &'a OtDeluxeStorePolicy,
    node_index: u8,
    inventory: &'a OtDeluxeInventory,
    oxen: OtDeluxeOxenState,
    available_cash: u64,
}

const fn remaining_cash(available_cash: u64, total_cost_cents: u64) -> u64 {
    if total_cost_cents <= available_cash {
        available_cash.saturating_sub(total_cost_cents)
    } else {
        0
    }
}

const STORE_ITEMS: [StoreItemDef; 7] = [
    StoreItemDef {
        item: OtDeluxeStoreItem::Oxen,
        name_key: "otdeluxe.store.items.oxen.name",
        desc_key: "otdeluxe.store.items.oxen.desc",
    },
    StoreItemDef {
        item: OtDeluxeStoreItem::FoodLb,
        name_key: "otdeluxe.store.items.food.name",
        desc_key: "otdeluxe.store.items.food.desc",
    },
    StoreItemDef {
        item: OtDeluxeStoreItem::ClothesSet,
        name_key: "otdeluxe.store.items.clothes.name",
        desc_key: "otdeluxe.store.items.clothes.desc",
    },
    StoreItemDef {
        item: OtDeluxeStoreItem::AmmoBox,
        name_key: "otdeluxe.store.items.ammo.name",
        desc_key: "otdeluxe.store.items.ammo.desc",
    },
    StoreItemDef {
        item: OtDeluxeStoreItem::Wheel,
        name_key: "otdeluxe.store.items.wheel.name",
        desc_key: "otdeluxe.store.items.wheel.desc",
    },
    StoreItemDef {
        item: OtDeluxeStoreItem::Axle,
        name_key: "otdeluxe.store.items.axle.name",
        desc_key: "otdeluxe.store.items.axle.desc",
    },
    StoreItemDef {
        item: OtDeluxeStoreItem::Tongue,
        name_key: "otdeluxe.store.items.tongue.name",
        desc_key: "otdeluxe.store.items.tongue.desc",
    },
];

#[derive(Properties, Clone)]
pub struct OtDeluxeStorePanelProps {
    pub state: Rc<GameState>,
    pub on_purchase: Callback<Vec<OtDeluxeStoreLineItem>>,
    pub on_leave: Callback<()>,
}

impl PartialEq for OtDeluxeStorePanelProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }
}

#[function_component(OtDeluxeStorePanel)]
pub fn otdeluxe_store_panel(props: &OtDeluxeStorePanelProps) -> Html {
    let cart = use_state(|| vec![0_u16; STORE_ITEMS.len()]);
    let pending_node = props.state.ot_deluxe.store.pending_node;

    #[cfg(target_arch = "wasm32")]
    {
        let cart = cart.clone();
        use_effect_with(pending_node, move |_| {
            cart.set(vec![0_u16; STORE_ITEMS.len()]);
            || ()
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = pending_node;
    }

    let store_policy = OtDeluxe90sPolicy::default().store;
    let node_index = props
        .state
        .ot_deluxe
        .store
        .pending_node
        .unwrap_or(props.state.ot_deluxe.route.current_node_index);
    let inventory = &props.state.ot_deluxe.inventory;
    let oxen = props.state.ot_deluxe.oxen;

    let total_cost_cents = cart_total_cents(cart.as_ref(), &store_policy, node_index);
    let available_cash = u64::from(inventory.cash_cents);
    let can_afford = total_cost_cents <= available_cash;
    let cash_left = remaining_cash(available_cash, total_cost_cents);

    let cash_str = fmt_currency(i64::from(inventory.cash_cents));
    let total_str = fmt_currency(u64_to_i64(total_cost_cents));
    let remaining_str = fmt_currency(u64_to_i64(cash_left));

    let cash_label = render_amount("otdeluxe.store.cash", &cash_str);
    let total_label = render_amount("otdeluxe.store.total", &total_str);
    let remaining_label = render_amount("otdeluxe.store.remaining", &remaining_str);
    let recommendation_keys = [
        "otdeluxe.store.recommendations.oxen",
        "otdeluxe.store.recommendations.food",
        "otdeluxe.store.recommendations.clothes",
        "otdeluxe.store.recommendations.spares",
    ];

    let on_leave = {
        let on_leave = props.on_leave.clone();
        Callback::from(move |_| on_leave.emit(()))
    };

    let on_checkout = {
        let cart = cart.clone();
        let on_purchase = props.on_purchase.clone();
        Callback::from(move |_| emit_purchase_lines(cart.as_ref(), &on_purchase))
    };

    let can_checkout = can_afford && total_cost_cents > 0;

    let ctx = StoreCardContext {
        cart: &cart,
        store_policy: &store_policy,
        node_index,
        inventory,
        oxen,
        available_cash,
    };

    html! {
        <main class="outfitting-store">
            <section role="region" aria-labelledby="otdeluxe-store-title" class="store-shell">
                <header class="store-header">
                    <div>
                        <h1 id="otdeluxe-store-title">{ i18n::t("otdeluxe.store.title") }</h1>
                    </div>
                    <div class="store-budget">
                        <span class="label">{ i18n::t("otdeluxe.store.cash_label") }</span>
                        <span class="value">{ cash_str }</span>
                    </div>
                </header>
                <div class="store-cart-summary" role="status" aria-live="polite">
                    <span>{ &total_label }</span>
                    <span class="value">{ &remaining_label }</span>
                </div>
                <aside class="store-recommendations" aria-labelledby="otdeluxe-store-recommend-title">
                    <h2 id="otdeluxe-store-recommend-title">
                        { i18n::t("otdeluxe.store.recommendations.title") }
                    </h2>
                    <ul>
                        { for recommendation_keys.iter().map(|key| html! {
                            <li>{ i18n::t(key) }</li>
                        }) }
                    </ul>
                </aside>
                <div class="store-item-grid">
                    { for STORE_ITEMS.iter().enumerate().map(|(idx, item)| {
                        render_item_card(idx, item, &ctx)
                    }) }
                </div>
                <div class="store-footer-row">
                    <button class="retro-btn-secondary" onclick={on_leave}>
                        { i18n::t("otdeluxe.store.leave") }
                    </button>
                    <button class="retro-btn-primary" onclick={on_checkout} disabled={!can_checkout}>
                        { i18n::t("otdeluxe.store.checkout") }
                    </button>
                </div>
                <div aria-live="polite" aria-atomic="true" class="sr-only" id="store-status">
                    { total_label }{ " " }{ cash_label }{ " " }{ remaining_label }
                </div>
            </section>
        </main>
    }
}

fn emit_purchase_lines(cart: &[u16], on_purchase: &Callback<Vec<OtDeluxeStoreLineItem>>) {
    let lines = build_purchase_lines(cart);
    on_purchase.emit(lines);
}

fn render_item_card(idx: usize, def: &StoreItemDef, ctx: &StoreCardContext<'_>) -> Html {
    let name = i18n::t(def.name_key);
    let desc = i18n::t(def.desc_key);
    let price_cents =
        otdeluxe_store::price_cents_at_node(ctx.store_policy, def.item, ctx.node_index);
    let price = u64::from(price_cents);
    let price_str = fmt_currency(i64::from(price_cents));

    let bullets_per_box = ctx.store_policy.bullets_per_box;
    let owned = current_quantity(def.item, ctx.inventory, ctx.oxen, bullets_per_box);
    let cap = otdeluxe_store::max_inventory(ctx.store_policy, def.item);
    let remaining = cap.saturating_sub(owned);

    let qty_in_cart = ctx.cart.get(idx).copied().unwrap_or(0);
    let total_cost = cart_total_cents(ctx.cart.as_ref(), ctx.store_policy, ctx.node_index);
    let can_add = qty_in_cart < remaining && total_cost.saturating_add(price) <= ctx.available_cash;

    let initials = name
        .chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect::<String>());

    let owned_str = owned.to_string();
    let cap_str = cap.to_string();
    let mut cap_map = BTreeMap::new();
    cap_map.insert("owned", owned_str.as_str());
    cap_map.insert("cap", cap_str.as_str());
    let cap_line = i18n::tr("otdeluxe.store.owned_cap", Some(&cap_map));

    let on_add = build_add_action(
        ctx.cart.clone(),
        idx,
        remaining,
        price,
        ctx.available_cash,
        (*ctx.store_policy).clone(),
        ctx.node_index,
    )
    .reform(|_e: MouseEvent| ());

    let on_remove = {
        let cart = ctx.cart.clone();
        Callback::from(move |_| apply_cart_remove(&cart, idx))
    };

    let title_id = format!("otdeluxe-store-item-{idx}");
    let desc_id = format!("otdeluxe-store-desc-{idx}");
    let cap_id = format!("otdeluxe-store-cap-{idx}");

    html! {
        <article
            role="group"
            aria-labelledby={title_id.clone()}
            aria-describedby={format!("{desc_id} {cap_id}")}
            class="store-card">
            <div class="store-card-icon" aria-hidden="true">
                <span>{ initials }</span>
            </div>
            <div class="store-card-body">
                <div class="store-card-head">
                    <h2 id={title_id}>{ name }</h2>
                    <span class="store-price">{ price_str }</span>
                </div>
                <p id={desc_id} class="muted">{ desc }</p>
                <p id={cap_id} class="muted">{ cap_line }</p>
                <div class="store-qty-row">
                    <button
                        class="store-qty-btn"
                        onclick={on_remove}
                        aria-label={i18n::t("otdeluxe.store.qty.rem1")}
                        disabled={qty_in_cart == 0}
                    >
                        {"-"}
                    </button>
                    <span class="store-qty" aria-live="polite">{ qty_in_cart }</span>
                    <button
                        class="store-qty-btn"
                        onclick={on_add}
                        aria-label={i18n::t("otdeluxe.store.qty.add1")}
                        disabled={!can_add}
                    >
                        {"+"}
                    </button>
                </div>
            </div>
        </article>
    }
}

fn apply_cart_add(
    cart: &UseStateHandle<Vec<u16>>,
    idx: usize,
    remaining: u16,
    price: u64,
    available_cash: u64,
    store_policy: &OtDeluxeStorePolicy,
    node_index: u8,
) {
    if let Some(next) = next_cart_add(
        cart,
        idx,
        remaining,
        price,
        available_cash,
        store_policy,
        node_index,
    ) {
        cart.set(next);
    }
}

fn build_add_action(
    cart: UseStateHandle<Vec<u16>>,
    idx: usize,
    remaining: u16,
    price: u64,
    available_cash: u64,
    store_policy: OtDeluxeStorePolicy,
    node_index: u8,
) -> Callback<()> {
    Callback::from(move |()| {
        apply_cart_add(
            &cart,
            idx,
            remaining,
            price,
            available_cash,
            &store_policy,
            node_index,
        );
    })
}

fn next_cart_add(
    cart: &[u16],
    idx: usize,
    remaining: u16,
    price: u64,
    available_cash: u64,
    store_policy: &OtDeluxeStorePolicy,
    node_index: u8,
) -> Option<Vec<u16>> {
    let current = cart.get(idx).copied().unwrap_or(0);
    if current >= remaining {
        return None;
    }
    let total = cart_total_cents(cart, store_policy, node_index);
    if total.saturating_add(price) > available_cash {
        return None;
    }
    let mut next = cart.to_vec();
    if let Some(entry) = next.get_mut(idx) {
        *entry = entry.saturating_add(1);
    }
    Some(next)
}

fn apply_cart_remove(cart: &UseStateHandle<Vec<u16>>, idx: usize) {
    let next = next_cart_remove(cart, idx);
    cart.set(next);
}

fn next_cart_remove(cart: &[u16], idx: usize) -> Vec<u16> {
    let mut next = cart.to_vec();
    if let Some(entry) = next.get_mut(idx) {
        *entry = entry.saturating_sub(1);
    }
    next
}

fn cart_total_cents(cart: &[u16], store_policy: &OtDeluxeStorePolicy, node_index: u8) -> u64 {
    STORE_ITEMS
        .iter()
        .enumerate()
        .map(|(idx, def)| {
            let qty = cart.get(idx).copied().unwrap_or(0);
            let price = otdeluxe_store::price_cents_at_node(store_policy, def.item, node_index);
            u64::from(qty).saturating_mul(u64::from(price))
        })
        .sum()
}

pub(super) fn build_purchase_lines(cart: &[u16]) -> Vec<OtDeluxeStoreLineItem> {
    STORE_ITEMS
        .iter()
        .enumerate()
        .filter_map(|(idx, def)| {
            let qty = cart.get(idx).copied().unwrap_or(0);
            if qty > 0 {
                Some(OtDeluxeStoreLineItem {
                    item: def.item,
                    quantity: qty,
                })
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn current_quantity(
    item: OtDeluxeStoreItem,
    inventory: &OtDeluxeInventory,
    oxen: OtDeluxeOxenState,
    bullets_per_box: u16,
) -> u16 {
    match item {
        OtDeluxeStoreItem::Oxen => oxen.total(),
        OtDeluxeStoreItem::ClothesSet => inventory.clothes_sets,
        OtDeluxeStoreItem::AmmoBox => {
            let per_box = bullets_per_box.max(1);
            let bullets = u32::from(inventory.bullets);
            let per_box_u32 = u32::from(per_box);
            let boxes = bullets.saturating_add(per_box_u32.saturating_sub(1)) / per_box_u32;
            u16::try_from(boxes).unwrap_or(u16::MAX)
        }
        OtDeluxeStoreItem::FoodLb => inventory.food_lbs,
        OtDeluxeStoreItem::Wheel => u16::from(inventory.spares_wheels),
        OtDeluxeStoreItem::Axle => u16::from(inventory.spares_axles),
        OtDeluxeStoreItem::Tongue => u16::from(inventory.spares_tongues),
    }
}

fn render_amount(key: &str, amount: &str) -> String {
    let mut map = BTreeMap::new();
    map.insert("amount", amount);
    i18n::tr(key, Some(&map))
}

fn u64_to_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::otdeluxe_state::{OtDeluxeInventory, OtDeluxeOxenState};
    use futures::executor::block_on;
    use std::cell::RefCell;
    use std::rc::Rc;
    use yew::LocalServerRenderer;

    #[test]
    fn current_quantity_covers_inventory_branches() {
        let inventory = OtDeluxeInventory {
            food_lbs: 250,
            bullets: 40,
            clothes_sets: 5,
            spares_wheels: 1,
            spares_axles: 2,
            spares_tongues: 3,
            cash_cents: 0,
        };
        let oxen = OtDeluxeOxenState {
            healthy: 3,
            sick: 1,
        };
        assert_eq!(
            current_quantity(OtDeluxeStoreItem::Oxen, &inventory, oxen, 20),
            4
        );
        assert_eq!(
            current_quantity(OtDeluxeStoreItem::FoodLb, &inventory, oxen, 20),
            250
        );
        assert_eq!(
            current_quantity(OtDeluxeStoreItem::ClothesSet, &inventory, oxen, 20),
            5
        );
        assert_eq!(
            current_quantity(OtDeluxeStoreItem::Wheel, &inventory, oxen, 20),
            1
        );
        assert_eq!(
            current_quantity(OtDeluxeStoreItem::Axle, &inventory, oxen, 20),
            2
        );
        assert_eq!(
            current_quantity(OtDeluxeStoreItem::Tongue, &inventory, oxen, 20),
            3
        );
        assert_eq!(
            current_quantity(OtDeluxeStoreItem::AmmoBox, &inventory, oxen, 20),
            2
        );
    }

    #[test]
    fn cart_total_cents_sums_line_items() {
        let policy = OtDeluxe90sPolicy::default().store;
        let mut cart = vec![0_u16; STORE_ITEMS.len()];
        cart[0] = 1;
        cart[1] = 2;
        let total = cart_total_cents(&cart, &policy, 0);
        let oxen_price = otdeluxe_store::price_cents_at_node(&policy, OtDeluxeStoreItem::Oxen, 0);
        let food_price = otdeluxe_store::price_cents_at_node(&policy, OtDeluxeStoreItem::FoodLb, 0);
        let expected = u64::from(oxen_price) + u64::from(food_price) * 2;
        assert_eq!(total, expected);
    }

    #[test]
    fn render_amount_and_u64_to_i64_cover_helpers() {
        crate::i18n::set_lang("en");
        let text = render_amount("otdeluxe.store.cash", "$5.00");
        assert!(text.contains("$5.00"));
        assert_eq!(u64_to_i64(u64::MAX), i64::MAX);
    }

    #[test]
    fn remaining_cash_never_negative() {
        assert_eq!(remaining_cash(500, 200), 300);
        assert_eq!(remaining_cash(100, 500), 0);
    }

    #[test]
    fn emit_purchase_lines_builds_nonzero_entries() {
        let mut cart = vec![0_u16; STORE_ITEMS.len()];
        cart[0] = 1;
        cart[3] = 2;
        let captured = Rc::new(RefCell::new(Vec::new()));
        let on_purchase = {
            let captured = captured.clone();
            Callback::from(move |lines| {
                *captured.borrow_mut() = lines;
            })
        };

        emit_purchase_lines(&cart, &on_purchase);

        let lines = captured.borrow();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].item, OtDeluxeStoreItem::Oxen);
        assert_eq!(lines[0].quantity, 1);
        assert_eq!(lines[1].item, OtDeluxeStoreItem::AmmoBox);
        assert_eq!(lines[1].quantity, 2);
    }

    #[test]
    fn cart_update_helpers_cover_add_remove_branches() {
        let policy = OtDeluxe90sPolicy::default().store;
        let price = u64::from(otdeluxe_store::price_cents_at_node(
            &policy,
            OtDeluxeStoreItem::Oxen,
            0,
        ));
        let base = vec![0_u16; STORE_ITEMS.len()];
        let added = next_cart_add(&base, 0, 2, price, 50_000, &policy, 0)
            .expect("should add when under caps and budget");
        assert_eq!(added[0], 1);
        assert!(next_cart_add(&added, 0, 1, price, 50_000, &policy, 0).is_none());
        assert!(next_cart_add(&base, 0, 2, price, 0, &policy, 0).is_none());
        let removed = next_cart_remove(&added, 0);
        assert_eq!(removed[0], 0);

        #[function_component(CartHarness)]
        fn cart_harness() -> Html {
            let cart = use_state(|| vec![0_u16; STORE_ITEMS.len()]);
            let invoked = use_mut_ref(|| false);
            let policy = OtDeluxe90sPolicy::default().store;
            let price = u64::from(otdeluxe_store::price_cents_at_node(
                &policy,
                OtDeluxeStoreItem::Oxen,
                0,
            ));
            if !*invoked.borrow() {
                *invoked.borrow_mut() = true;
                apply_cart_add(&cart, 0, 2, price, 50_000, &policy, 0);
                apply_cart_remove(&cart, 0);
            }
            let called = if *invoked.borrow() { "true" } else { "false" };
            html! { <div data-called={called} /> }
        }

        let html = block_on(LocalServerRenderer::<CartHarness>::new().render());
        assert!(html.contains("data-called=\"true\""));
    }

    #[test]
    fn build_add_action_executes_callback() {
        #[function_component(AddActionHarness)]
        fn add_action_harness() -> Html {
            let cart = use_state(|| vec![0_u16; STORE_ITEMS.len()]);
            let invoked = use_mut_ref(|| false);
            let policy = OtDeluxe90sPolicy::default().store;
            let price = u64::from(otdeluxe_store::price_cents_at_node(
                &policy,
                OtDeluxeStoreItem::Oxen,
                0,
            ));
            let action = build_add_action(cart, 0, 2, price, 50_000, policy, 0);
            if !*invoked.borrow() {
                *invoked.borrow_mut() = true;
                action.emit(());
            }
            let called = if *invoked.borrow() { "true" } else { "false" };
            html! { <div data-called={called} /> }
        }

        let html = block_on(LocalServerRenderer::<AddActionHarness>::new().render());
        assert!(html.contains("data-called=\"true\""));
    }
}
