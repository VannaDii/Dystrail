use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    StartRun,
    CampPreview,
    OpenSave,
    OpenSettings,
    Reset,
}

#[derive(Properties, Clone, PartialEq)]
pub struct MenuPageProps {
    pub code: AttrValue,
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
            <section class="panel retro-menu">
                <header class="retro-header" role="banner">
                    <div class="header-center">
                        <pre class="ascii-art">
    { "═══════════════════════════════" }<br/>
    { "D Y S T R A I L" }<br/>
    { "A Political Survival Adventure" }<br/>
    { "═══════════════════════════════" }
                        </pre>
                    </div>
                    <p class="muted" aria-live="polite">
                        { format!("{seed_label} {code}", seed_label = crate::i18n::t("game.seed_label"), code = props.code.clone()) }
                    </p>
                </header>
                <img src={props.logo_src.clone()} alt="Dystrail" loading="lazy" style="width:min(520px,80vw)"/>
                <crate::components::ui::main_menu::MainMenu seed_text={Some(props.code.to_string())} on_select={Some(on_select)} />
            </section>
        }
}

const fn menu_action_for_index(idx: u8) -> Option<MenuAction> {
    match idx {
        1 => Some(MenuAction::StartRun),
        2 => Some(MenuAction::CampPreview),
        7 => Some(MenuAction::OpenSave),
        8 => Some(MenuAction::OpenSettings),
        0 => Some(MenuAction::Reset),
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
            Some(MenuAction::StartRun)
        ));
        assert!(matches!(
            menu_action_for_index(2),
            Some(MenuAction::CampPreview)
        ));
        assert!(matches!(
            menu_action_for_index(7),
            Some(MenuAction::OpenSave)
        ));
        assert!(matches!(
            menu_action_for_index(8),
            Some(MenuAction::OpenSettings)
        ));
        assert!(matches!(menu_action_for_index(0), Some(MenuAction::Reset)));
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
        assert!(matches!(captured[0], MenuAction::StartRun));
    }
}
