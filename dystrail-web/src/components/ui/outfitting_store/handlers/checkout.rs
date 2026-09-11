use super::super::state::OutfittingStoreProps;
use crate::game::store::Grants;

pub fn handle_checkout(
    state: &crate::components::ui::outfitting_store::state::StoreState,
    props: &OutfittingStoreProps,
) {
    let mut total_grants = Grants::default();
    let mut all_tags = Vec::new();

    for line in &state.cart.lines {
        if let Some(item) = state.store_data.find_item(&line.item_id) {
            total_grants.supplies += item.grants.supplies * line.qty;
            total_grants.credibility += item.grants.credibility * line.qty;
            total_grants.spare_tire += item.grants.spare_tire * line.qty;
            total_grants.spare_battery += item.grants.spare_battery * line.qty;
            total_grants.spare_alt += item.grants.spare_alt * line.qty;
            total_grants.spare_pump += item.grants.spare_pump * line.qty;

            for tag in &item.tags {
                all_tags.push(tag.clone());
            }
        }
    }

    if props.game_state.stats.supplies + total_grants.supplies > 20
        || state.cart.total_cents > props.game_state.budget_cents
    {
        return;
    }
    let mut new_game_state = props.game_state.clone();
    new_game_state.apply_store_purchase(state.cart.total_cents, &total_grants, &all_tags);

    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let key = format!(
            "dystrail.cart.{}.{}.{:?}.{}",
            props.game_state.seed,
            props.game_state.day,
            props.game_state.persona_id,
            props.resupply
        );
        let _ = storage.remove_item(&key);
        let _ = storage.remove_item(&format!("{key}.view"));
    }
    props
        .on_continue
        .emit((new_game_state, total_grants, all_tags));
}
