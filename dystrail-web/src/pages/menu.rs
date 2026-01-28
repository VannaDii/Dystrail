use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    StartJourney,
    About,
    Settings,
    Quit,
}

#[derive(Properties, Clone, PartialEq)]
pub struct MenuPageProps {
    #[prop_or_default]
    pub logo_src: AttrValue,
    pub on_action: Callback<MenuAction>,
}

fn menu_action_callback(on_action: Callback<MenuAction>) -> Callback<u8> {
    Callback::from(move |idx: u8| {
        if let Some(action) = menu_action_for_index(idx) {
            on_action.emit(action);
        }
    })
}

#[function_component(MenuPage)]
pub fn menu_page(props: &MenuPageProps) -> Html {
    let on_select = menu_action_callback(props.on_action.clone());

    html! {
        <div class="min-h-screen flex items-center justify-center bg-base-300 font-sans shell-screen" data-testid="menu-screen">
            <div class="card border border-base-content bg-base-200 w-[420px] max-w-full rounded-none shadow-none shell-card">
                <div class="card-body items-center text-center gap-6">
                    <div class="space-y-1">
                        <h1 class="text-2xl font-bold tracking-tight">{ crate::i18n::t("app.title") }</h1>
                        <p class="text-xs opacity-60">{ crate::i18n::t("menu.subtitle") }</p>
                    </div>

                    <crate::components::ui::main_menu::MainMenu on_select={Some(on_select)} />

                    <div class="text-xs opacity-50">
                        { crate::i18n::t("menu.footer") }
                    </div>
                </div>
            </div>
        </div>
    }
}

const fn menu_action_for_index(idx: u8) -> Option<MenuAction> {
    match idx {
        1 => Some(MenuAction::StartJourney),
        2 => Some(MenuAction::About),
        3 => Some(MenuAction::Settings),
        4 => Some(MenuAction::Quit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{MenuAction, menu_action_callback, menu_action_for_index};
    use std::cell::RefCell;
    use std::rc::Rc;
    use yew::prelude::Callback;

    #[test]
    fn menu_action_mapping_covers_expected_indices() {
        assert!(matches!(
            menu_action_for_index(1),
            Some(MenuAction::StartJourney)
        ));
        assert!(matches!(menu_action_for_index(2), Some(MenuAction::About)));
        assert!(matches!(
            menu_action_for_index(3),
            Some(MenuAction::Settings)
        ));
        assert!(matches!(menu_action_for_index(4), Some(MenuAction::Quit)));
        assert!(menu_action_for_index(9).is_none());
    }

    #[test]
    fn menu_action_callback_emits_for_known_index() {
        let captured = Rc::new(RefCell::new(Vec::new()));
        let captured_ref = captured.clone();
        let on_action = Callback::from(move |action| {
            captured_ref.borrow_mut().push(action);
        });
        let on_select = menu_action_callback(on_action);
        on_select.emit(1);
        on_select.emit(9);
        let captured = captured.borrow();
        assert_eq!(captured.len(), 1);
        assert!(matches!(captured[0], MenuAction::StartJourney));
    }
}
