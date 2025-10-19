use yew::prelude::*;

use crate::a11y::set_status;
use crate::i18n;
use crate::input::{numeric_code_to_index, numeric_key_to_index};
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

#[derive(Properties, PartialEq, Clone)]
pub struct MenuItemProps {
    pub index: u8,        // 0..9
    pub label: AttrValue, // resolved string
    pub focused: bool,    // tabindex 0 vs -1
    pub posinset: u8,     // 1..=setsize
    pub setsize: u8,
    pub on_activate: Callback<u8>, // called with index
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[test]
    fn main_menu_renders_all_entries_with_seed() {
        crate::i18n::set_lang("en");
        let html = block_on(
            LocalServerRenderer::<MainMenu>::with_props(MainMenuProps {
                seed_text: Some("CL-TEST42".to_string()),
                on_select: None,
            })
            .render(),
        );

        // Ensure helper text interpolates seed and every menu item is present.
        assert!(
            html.contains("Seed: CL-TEST42"),
            "Rendered menu should include helper text with seed, got: {html}"
        );
        for key in [
            "data-key=\"1\"",
            "data-key=\"2\"",
            "data-key=\"3\"",
            "data-key=\"4\"",
            "data-key=\"5\"",
            "data-key=\"6\"",
            "data-key=\"7\"",
            "data-key=\"8\"",
            "data-key=\"0\"",
        ] {
            assert!(
                html.contains(key),
                "Expected menu item with {key} in rendered HTML: {html}"
            );
        }
        assert!(
            html.contains("Main Menu"),
            "Rendered markup should expose localized menu title: {html}"
        );
    }
}

#[function_component(MenuItem)]
pub fn menu_item(p: &MenuItemProps) -> Html {
    let idx = p.index;
    let on_click = {
        let on = p.on_activate.clone();
        Callback::from(move |_| on.emit(idx))
    };

    html! {
      <li role="menuitem"
          tabindex={ if p.focused { "0" } else { "-1" } }
          data-key={idx.to_string()}
          aria-posinset={p.posinset.to_string()}
          aria-setsize={p.setsize.to_string()}
          onclick={on_click}
          class="ot-menuitem">
         <span class="num">{ format!("{}{})", idx, ")") }</span>
         <span class="label">{ p.label.clone() }</span>
      </li>
    }
}

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

    let activate = {
        let on = p.on_select.clone();
        Callback::from(move |idx: u8| {
            let label_key = match idx {
                1 => "menu.travel",
                2 => "menu.camp",
                3 => "menu.status",
                4 => "menu.pace",
                5 => "menu.diet",
                6 => "menu.inventory",
                7 => "menu.share",
                8 => "menu.settings",
                0 => "menu.quit",
                _ => "",
            };
            let label = i18n::t(label_key);
            let msg = format!("{selected} {label}", selected = i18n::t("menu.selected"));
            set_status(&msg);
            if let Some(cb) = on.clone() {
                cb.emit(idx);
            }
        })
    };

    // When focus index changes, move DOM focus to the corresponding item
    {
        let list_ref = list_ref.clone();
        use_effect_with(*focus_idx, move |idx| {
            if let Some(list) = list_ref.cast::<web_sys::Element>() {
                let sel = format!("[role='menuitem'][data-key='{idx}']");
                if let Ok(Some(el)) = list.query_selector(&sel) {
                    let _ = el
                        .dyn_into::<web_sys::HtmlElement>()
                        .ok()
                        .map(|e| e.focus());
                }
            }
        });
    }

    let on_keydown = {
        let activate = activate.clone();
        let focus_idx = focus_idx.clone();
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            // Direct numeric activation
            if let Some(n) = numeric_key_to_index(&key) {
                activate.emit(n);
                e.prevent_default();
                return;
            }
            // Use code (DigitN/NumpadN) as fallback
            if let Some(n) = numeric_code_to_index(&e.code()) {
                activate.emit(n);
                e.prevent_default();
                return;
            }
            if key == "Enter" || key == " " {
                // Space
                activate.emit(*focus_idx);
                e.prevent_default();
            } else if key == "Escape" {
                // Leave handling to outer modal handlers by design
                e.prevent_default();
            } else if key == "ArrowDown" {
                let mut next = *focus_idx + 1;
                if next > 8 {
                    next = 0;
                }
                focus_idx.set(next);
                e.prevent_default();
            } else if key == "ArrowUp" {
                let mut prev = if *focus_idx == 0 { 8 } else { *focus_idx - 1 };
                if prev == 0 {
                    prev = 8;
                }
                focus_idx.set(prev);
                e.prevent_default();
            }
        })
    };

    let seed = p.seed_text.clone().unwrap_or_default();
    let mut vars = std::collections::HashMap::new();
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
