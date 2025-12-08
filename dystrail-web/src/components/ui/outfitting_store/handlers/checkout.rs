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

    let mut new_game_state = props.game_state.clone();
    new_game_state.apply_store_purchase(state.cart.total_cents, &total_grants, &all_tags);

    props
        .on_continue
        .emit((new_game_state, total_grants, all_tags));
}
