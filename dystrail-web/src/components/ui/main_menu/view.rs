use super::interactions::{activate_handler, focus_effect, keydown_handler};
use super::item::MenuItem;
use crate::i18n;
use std::collections::BTreeMap;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct MainMenuProps {
    #[prop_or_default]
    pub seed_text: Option<String>,
    #[prop_or_default]
    pub on_select: Option<Callback<u8>>, // if not provided, only status is updated
}

#[function_component(MainMenu)]
pub fn main_menu(p: &MainMenuProps) -> Html {
    let focus_idx = use_state(|| 1_u8);
    let list_ref = use_node_ref();
    let setsize = 9_u8; // 1..8 and 0

    let activate = activate_handler(p.on_select.clone());
    focus_effect(list_ref.clone(), &focus_idx);
    let on_keydown = keydown_handler(activate.clone(), focus_idx.clone());

    let seed = p.seed_text.clone().unwrap_or_default();
    let mut vars = BTreeMap::new();
    vars.insert("seed", seed.as_str());
    let helper_text = i18n::tr("menu.help", Some(&vars));

    let items: Vec<(u8, String)> = vec![
        (1_u8, i18n::t("menu.travel")),
        (2, i18n::t("menu.camp")),
        (3, i18n::t("menu.status")),
        (4, i18n::t("menu.pace")),
        (5, i18n::t("menu.diet")),
        (6, i18n::t("menu.inventory")),
        (7, i18n::t("menu.share")),
        (8, i18n::t("menu.settings")),
        (0, i18n::t("menu.quit")),
    ];

    html! {
      <section role="region" aria-labelledby="menu-title" onkeydown={on_keydown} class="ot-menu">
        <h2 id="menu-title" class="sr-only">{ i18n::t("menu.title") }</h2>
        <ul role="menu" aria-label={i18n::t("menu.title")} id="main-menu" ref={list_ref}>
          { for items.iter().enumerate().map(|(i, (idx, label))| {
              let focused = *focus_idx == *idx;
              let posinset = u8::try_from(i).unwrap_or(0) + 1;
              html!{ <MenuItem index={*idx} posinset={posinset} label={AttrValue::from(label.clone())} focused={focused} setsize={setsize} on_activate={activate.clone()} /> }
          }) }
        </ul>
        <p id="menu-helper" aria-live="polite" class="muted">{ helper_text }</p>
      </section>
    }
}
