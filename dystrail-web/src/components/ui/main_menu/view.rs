use super::interactions::{activate_handler, focus_effect, keydown_handler};
use super::item::MenuItem;
use crate::i18n;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct MainMenuProps {
    #[prop_or_default]
    pub on_select: Option<Callback<u8>>, // if not provided, only status is updated
}

#[function_component(MainMenu)]
pub fn main_menu(p: &MainMenuProps) -> Html {
    let focus_idx = use_state(|| 1_u8);
    let list_ref = use_node_ref();
    let setsize = 4_u8; // 1..=4

    let activate = activate_handler(p.on_select.clone());
    let on_focus = {
        let focus_idx = focus_idx.clone();
        Callback::from(move |idx: u8| focus_idx.set(idx))
    };
    focus_effect(list_ref.clone(), &focus_idx);
    let on_keydown = keydown_handler(activate.clone(), focus_idx.clone());

    let helper_text = i18n::t("menu.help");

    let items: Vec<(u8, String, bool)> = vec![
        (1_u8, i18n::t("menu.start_journey"), true),
        (2, i18n::t("menu.about"), false),
        (3, i18n::t("menu.accessibility"), false),
        (4, i18n::t("menu.quit"), false),
    ];

    html! {
      <section role="region" aria-labelledby="menu-title" onkeydown={on_keydown} class="menu-shell">
        <h2 id="menu-title" class="sr-only">{ i18n::t("menu.title") }</h2>
        <ul role="menu" aria-label={i18n::t("menu.title")} id="main-menu" ref={list_ref} class="menu-list">
          { for items.iter().enumerate().map(|(i, (idx, label, primary))| {
              let focused = *focus_idx == *idx;
              let posinset = u8::try_from(i).unwrap_or(0) + 1;
              html!{ <MenuItem index={*idx} posinset={posinset} label={AttrValue::from(label.clone())} focused={focused} setsize={setsize} primary={*primary} on_activate={activate.clone()} on_focus={on_focus.clone()} /> }
          }) }
        </ul>
        <p id="menu-helper" aria-live="polite" class="text-xs opacity-70">{ helper_text }</p>
      </section>
    }
}
