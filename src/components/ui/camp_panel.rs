use crate::i18n;
use crate::game::GameState;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct Props {
    pub game_state: Rc<GameState>,
    pub on_repair_vehicle: Callback<()>,
    pub on_back: Callback<()>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        // Compare relevant fields for re-rendering
        Rc::ptr_eq(&self.game_state, &other.game_state)
    }
}

#[function_component(CampPanel)]
pub fn camp_panel(p: &Props) -> Html {
    let show_vehicle_repair = use_state(|| false);

    let on_vehicle_repair = {
        let show_repair = show_vehicle_repair.clone();
        Callback::from(move |_: MouseEvent| {
            show_repair.set(true);
        })
    };

    let on_back_to_camp = {
        let show_repair = show_vehicle_repair.clone();
        Callback::from(move |_| {
            show_repair.set(false);
        })
    };

    let on_repair_action = {
        let show_repair = show_vehicle_repair.clone();
        Callback::from(move |_action| {
            // For now, just close the repair menu after any action
            show_repair.set(false);
        })
    };

    let on_keydown = {
        let show_repair = show_vehicle_repair.clone();
        let on_back = p.on_back.clone();
        let on_vehicle_repair = on_vehicle_repair.clone();
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            match key.as_str() {
                "1" | "Digit1" => {
                    e.prevent_default();
                    if let Ok(fake_event) = MouseEvent::new("click") {
                        on_vehicle_repair.emit(fake_event);
                    }
                }
                "0" | "Digit0" => {
                    e.prevent_default();
                    on_back.emit(());
                }
                "Escape" => {
                    e.prevent_default();
                    if *show_repair {
                        show_repair.set(false);
                    } else {
                        on_back.emit(());
                    }
                }
                _ => {}
            }
        })
    };

    if *show_vehicle_repair {
        html! {
            <section role="region" aria-labelledby="camp-repair-title" onkeydown={on_keydown} class="ot-menu">
                <h2 id="camp-repair-title">{ i18n::t("camp.repair.title") }</h2>

                <crate::components::ui::vehicle_status::VehicleStatus
                    game_state={Some(p.game_state.clone())}
                    on_back={on_back_to_camp}
                    on_repair_action={on_repair_action}
                />
            </section>
        }
    } else {
        html! {
            <section role="region" aria-labelledby="camp-title" onkeydown={on_keydown} class="ot-menu">
                <h2 id="camp-title">{ i18n::t("camp.title") }</h2>

                <div role="menu" aria-labelledby="camp-title">
                    <div
                        role="menuitem"
                        aria-posinset="1"
                        aria-setsize="2"
                        tabindex="0"
                        class="menu-item"
                        onclick={on_vehicle_repair}
                    >
                        <span class="menu-number">{"1)"}</span>
                        <span class="menu-text">{ i18n::t("camp.repair_vehicle") }</span>
                    </div>

                    <div
                        role="menuitem"
                        aria-posinset="2"
                        aria-setsize="2"
                        tabindex="-1"
                        class="menu-item"
                        onclick={Callback::from({
                            let on_back = p.on_back.clone();
                            move |_: MouseEvent| on_back.emit(())
                        })}
                    >
                        <span class="menu-number">{"0)"}</span>
                        <span class="menu-text">{ i18n::t("vehicle.back") }</span>
                    </div>
                </div>

                <p class="help-text" role="status">
                    { i18n::t("camp.help") }
                </p>
            </section>
        }
    }
}