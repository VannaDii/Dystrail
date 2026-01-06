//! Trail graph helpers for Oregon Trail Deluxe parity.

use crate::mechanics::otdeluxe90s::{
    OtDeluxeStorePolicy, OtDeluxeTrailPolicy, OtDeluxeTrailVariant,
};

const OTDELUXE_TRAIL_NODE_COUNT: u8 = 18;
const OTDELUXE_TRAIL_MARKER_COUNT: u8 = 17;

const fn markers_for_variant(
    policy: &OtDeluxeTrailPolicy,
    variant: OtDeluxeTrailVariant,
) -> &[u16; 17] {
    match variant {
        OtDeluxeTrailVariant::Main => &policy.mile_markers_main,
        OtDeluxeTrailVariant::SubletteCutoff => &policy.mile_markers_sublette,
        OtDeluxeTrailVariant::DallesShortcut => &policy.mile_markers_dalles_shortcut,
        OtDeluxeTrailVariant::SubletteAndDallesShortcut => {
            &policy.mile_markers_sublette_and_dalles_shortcut
        }
    }
}

/// Return the mile marker for a node index, or None if the node is skipped.
#[must_use]
pub fn mile_marker_for_node(
    policy: &OtDeluxeTrailPolicy,
    variant: OtDeluxeTrailVariant,
    node_index: u8,
) -> Option<u16> {
    if node_index == 0 {
        return Some(0);
    }
    if node_index >= OTDELUXE_TRAIL_NODE_COUNT {
        return None;
    }
    let markers = markers_for_variant(policy, variant);
    let marker_index = usize::from(node_index.saturating_sub(1));
    if marker_index >= usize::from(OTDELUXE_TRAIL_MARKER_COUNT) {
        return None;
    }
    let marker = markers[marker_index];
    if marker == 0 { None } else { Some(marker) }
}

/// Return the current node index based on miles traveled.
#[must_use]
pub fn node_index_for_miles(
    policy: &OtDeluxeTrailPolicy,
    variant: OtDeluxeTrailVariant,
    miles_traveled: f32,
) -> u8 {
    if miles_traveled <= 0.0 {
        return 0;
    }
    let markers = markers_for_variant(policy, variant);
    let mut current = 0_u8;
    for (idx, marker) in markers.iter().enumerate() {
        if *marker == 0 {
            continue;
        }
        if f32::from(*marker) <= miles_traveled {
            current = u8::try_from(idx + 1).unwrap_or(current);
        }
    }
    current
}

/// Return the next reachable node index after the current node.
#[must_use]
pub fn next_node_index(
    policy: &OtDeluxeTrailPolicy,
    variant: OtDeluxeTrailVariant,
    current_node_index: u8,
) -> Option<u8> {
    let start = current_node_index.saturating_add(1);
    (start..OTDELUXE_TRAIL_NODE_COUNT)
        .find(|&node_index| mile_marker_for_node(policy, variant, node_index).is_some())
}

/// Determine the total trail miles for the selected variant.
#[must_use]
pub fn total_miles_for_variant(policy: &OtDeluxeTrailPolicy, variant: OtDeluxeTrailVariant) -> u16 {
    let markers = markers_for_variant(policy, variant);
    let mut max_marker = 0_u16;
    for marker in markers.iter().copied() {
        if marker > max_marker {
            max_marker = marker;
        }
    }
    if max_marker == 0 {
        policy.total_miles_main
    } else {
        max_marker
    }
}

/// Return the store price multiplier for a node index (0..17).
#[must_use]
pub fn price_multiplier_pct_for_node(store: &OtDeluxeStorePolicy, node_index: u8) -> u16 {
    price_multiplier_pct_for_stage(store, node_index.min(17))
}

/// Return the store price multiplier for a pricing stage (0..18).
///
/// Stage 18 is reserved for a post-arrival pricing tier in the extracted table.
#[must_use]
pub fn price_multiplier_pct_for_stage(store: &OtDeluxeStorePolicy, stage_index: u8) -> u16 {
    let idx = usize::from(stage_index.min(18));
    store
        .price_mult_pct_by_node
        .get(idx)
        .copied()
        .unwrap_or_else(|| store.price_mult_pct_by_node[18])
}

/// Determine whether a store is available at the given node index.
#[must_use]
pub fn store_available_at_node(
    trail: &OtDeluxeTrailPolicy,
    store: &OtDeluxeStorePolicy,
    variant: OtDeluxeTrailVariant,
    node_index: u8,
) -> bool {
    if !store.buy_only_at_forts {
        return mile_marker_for_node(trail, variant, node_index).is_some();
    }
    if !store.store_node_indices.contains(&node_index) {
        return false;
    }
    if node_index == 0 {
        return true;
    }
    mile_marker_for_node(trail, variant, node_index).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::otdeluxe90s::OtDeluxe90sPolicy;

    #[test]
    fn mile_markers_skip_sentinels() {
        let policy = OtDeluxe90sPolicy::default();
        let trail = &policy.trail;
        assert_eq!(
            mile_marker_for_node(trail, OtDeluxeTrailVariant::SubletteCutoff, 8),
            None
        );
        assert_eq!(
            mile_marker_for_node(trail, OtDeluxeTrailVariant::DallesShortcut, 15),
            None
        );
    }

    #[test]
    fn node_index_tracks_miles() {
        let policy = OtDeluxe90sPolicy::default();
        let trail = &policy.trail;
        assert_eq!(
            node_index_for_miles(trail, OtDeluxeTrailVariant::Main, 0.0),
            0
        );
        assert_eq!(
            node_index_for_miles(trail, OtDeluxeTrailVariant::Main, 150.0),
            1
        );
        assert_eq!(
            node_index_for_miles(trail, OtDeluxeTrailVariant::Main, 304.0),
            3
        );
    }

    #[test]
    fn store_availability_respects_variant_skips() {
        let policy = OtDeluxe90sPolicy::default();
        let trail = &policy.trail;
        let store = &policy.store;
        assert!(store_available_at_node(
            trail,
            store,
            OtDeluxeTrailVariant::Main,
            15
        ));
        assert!(!store_available_at_node(
            trail,
            store,
            OtDeluxeTrailVariant::DallesShortcut,
            15
        ));
    }
}
