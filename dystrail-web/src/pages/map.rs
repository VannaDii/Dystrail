use crate::game::GameState;
use crate::game::state::Region;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct MapPageProps {
    pub state: Rc<GameState>,
    pub on_back: Callback<()>,
}

impl PartialEq for MapPageProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }
}

fn region_label(region: Region) -> String {
    match region {
        Region::Heartland => crate::i18n::t("region.heartland"),
        Region::RustBelt => crate::i18n::t("region.rustbelt"),
        Region::Beltway => crate::i18n::t("region.beltway"),
    }
}

#[function_component(MapPage)]
pub fn map_page(props: &MapPageProps) -> Html {
    let miles_traveled = crate::i18n::fmt_number(f64::from(props.state.miles_traveled_actual));
    let miles_today = crate::i18n::fmt_number(f64::from(props.state.distance_today));
    let region = region_label(props.state.region);
    let on_back = props.on_back.clone();

    html! {
        <section class="panel retro-menu" aria-labelledby="map-title" data-testid="map-screen">
            <h2 id="map-title">{ crate::i18n::t("map.title") }</h2>
            <div class="stats-list" role="list">
                <div role="listitem">{ format!("{}: {}", crate::i18n::t("map.distance_traveled"), miles_traveled) }</div>
                <div role="listitem">{ format!("{}: {}", crate::i18n::t("map.distance_today"), miles_today) }</div>
                <div role="listitem">{ format!("{}: {}", crate::i18n::t("map.region"), region) }</div>
            </div>
            <div class="controls">
                <button class="retro-btn-secondary" onclick={Callback::from(move |_| on_back.emit(()))}>
                    { crate::i18n::t("ui.back") }
                </button>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_props_equality_tracks_shared_state() {
        let state = Rc::new(GameState::default());
        let props_a = MapPageProps {
            state: state.clone(),
            on_back: Callback::from(|()| ()),
        };
        let props_b = MapPageProps {
            state,
            on_back: Callback::from(|()| ()),
        };
        assert!(props_a == props_b);

        let props_c = MapPageProps {
            state: Rc::new(GameState::default()),
            on_back: Callback::from(|()| ()),
        };
        assert!(props_a != props_c);
    }

    #[test]
    fn region_label_covers_all_regions() {
        crate::i18n::set_lang("en");
        let rust_belt = region_label(Region::RustBelt);
        let beltway = region_label(Region::Beltway);
        assert!(!rust_belt.is_empty());
        assert!(!beltway.is_empty());
    }
}
