use crate::game::GameState;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct InventoryPageProps {
    pub state: Rc<GameState>,
    pub on_back: Callback<()>,
}

impl PartialEq for InventoryPageProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }
}

#[function_component(InventoryPage)]
pub fn inventory_page(props: &InventoryPageProps) -> Html {
    let inventory = &props.state.inventory;
    let spares = &inventory.spares;
    let tags = inventory.tags.iter().cloned().collect::<Vec<_>>();
    let tag_items = tags
        .iter()
        .map(|tag| html! { <li>{ tag }</li> })
        .collect::<Html>();

    let on_back = props.on_back.clone();

    html! {
        <section class="panel retro-menu" aria-labelledby="inventory-title" data-testid="inventory-screen">
            <h2 id="inventory-title">{ crate::i18n::t("inventory.title") }</h2>
            <div class="stats-list" role="list">
                <div role="listitem">{ format!("{}: {}", crate::i18n::t("inventory.supplies"), props.state.stats.supplies) }</div>
                <div role="listitem">{ format!("{}: {}", crate::i18n::t("inventory.spare_tire"), spares.tire) }</div>
                <div role="listitem">{ format!("{}: {}", crate::i18n::t("inventory.spare_battery"), spares.battery) }</div>
                <div role="listitem">{ format!("{}: {}", crate::i18n::t("inventory.spare_alt"), spares.alt) }</div>
                <div role="listitem">{ format!("{}: {}", crate::i18n::t("inventory.spare_pump"), spares.pump) }</div>
            </div>
            <div class="inventory-tags">
                <h3 class="muted">{ crate::i18n::t("inventory.tags") }</h3>
                <ul>
                    { if tags.is_empty() {
                        html! { <li>{ crate::i18n::t("inventory.tags_none") }</li> }
                    } else {
                        tag_items
                    }}
                </ul>
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
    fn inventory_props_equality_tracks_shared_state() {
        let state = Rc::new(GameState::default());
        let props_a = InventoryPageProps {
            state: state.clone(),
            on_back: Callback::from(|()| ()),
        };
        let props_b = InventoryPageProps {
            state,
            on_back: Callback::from(|()| ()),
        };
        assert!(props_a == props_b);

        let props_c = InventoryPageProps {
            state: Rc::new(GameState::default()),
            on_back: Callback::from(|()| ()),
        };
        assert!(props_a != props_c);
    }
}
