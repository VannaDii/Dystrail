mod helpers;
mod preview;
mod tile;

use crate::game::personas::{Persona, PersonasList};
use crate::input::{numeric_code_to_index, numeric_key_to_index};
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct PersonaSelectProps {
    #[prop_or_default]
    pub on_selected: Option<Callback<Persona>>,
    #[prop_or_default]
    pub on_continue: Option<Callback<()>>,
    #[prop_or_default]
    pub on_back: Option<Callback<()>>,
    #[prop_or_default]
    pub initial_id: Option<String>,
}

#[function_component(PersonaSelect)]
pub fn persona_select(p: &PersonaSelectProps) -> Html {
    crate::i18n::use_language();
    let personas = use_state(Vec::<Persona>::new);
    let selected = use_state(|| None::<usize>);
    let list_ref = use_node_ref();
    let focus_selection = use_mut_ref(|| false);

    {
        let personas = personas.clone();
        let selected = selected.clone();
        let initial_id = p.initial_id.clone();
        let on_selected = p.on_selected.clone();
        use_effect_with((), move |()| {
            let data = include_str!("../../../../static/assets/data/personas.json");
            let list = PersonasList::from_json(data).unwrap_or_else(|_| PersonasList::empty());
            let index = helpers::initial_selection(
                &list.0,
                initial_id.as_deref(),
                js_sys::Math::random().to_bits(),
            );
            selected.set(index);
            if let Some(per) = index.and_then(|i| list.0.get(i))
                && initial_id.as_deref() != Some(per.id.as_str())
                && let Some(cb) = on_selected
            {
                cb.emit(per.clone());
            }
            personas.set(list.0);
            || {}
        });
    }

    let select_idx = {
        let selected = selected.clone();
        let personas_state = personas.clone();
        let on_selected = p.on_selected.clone();
        Callback::from(move |idx: usize| {
            if idx < personas_state.len() {
                selected.set(Some(idx));
                let per = &personas_state[idx];
                if let Some(cb) = on_selected.clone() {
                    cb.emit(per.clone());
                }
            }
        })
    };

    {
        let list_ref = list_ref.clone();
        let focus_selection = focus_selection.clone();
        use_effect_with(*selected, move |sel| {
            if !*focus_selection.borrow() {
                return;
            }
            *focus_selection.borrow_mut() = false;
            if let Some(first) = sel.as_ref().and_then(|i| {
                let selector = format!("[role='radio'][data-key='{}']", i + 1);
                list_ref
                    .cast::<web_sys::Element>()
                    .and_then(|list| list.query_selector(&selector).ok().flatten())
                    .and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
            }) {
                let _ = first.focus();
            }
        });
    }

    let on_keydown = {
        let selected = selected.clone();
        let select_idx = select_idx.clone();
        let on_continue = p.on_continue.clone();
        let count = personas.len();
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            if count > 0
                && matches!(
                    key.as_str(),
                    "ArrowRight" | "ArrowDown" | "ArrowLeft" | "ArrowUp"
                )
            {
                let current = selected.unwrap_or(0);
                let next = if matches!(key.as_str(), "ArrowRight" | "ArrowDown") {
                    (current + 1) % count
                } else {
                    (current + count - 1) % count
                };
                *focus_selection.borrow_mut() = true;
                select_idx.emit(next);
                e.prevent_default();
                return;
            }
            if let Some(n) = numeric_key_to_index(&key).or_else(|| numeric_code_to_index(&e.code()))
            {
                if n == 0 {
                    if selected.is_none() {
                        e.prevent_default();
                        return;
                    }
                    if let Some(cb) = on_continue.clone() {
                        cb.emit(());
                    }
                } else {
                    let idx = (n as usize).saturating_sub(1);
                    *focus_selection.borrow_mut() = true;
                    select_idx.emit(idx);
                }
                e.prevent_default();
            }
        })
    };

    let preview_persona = selected.and_then(|i| personas.get(i)).cloned();
    let live_msg = preview_persona
        .as_ref()
        .map_or_else(String::new, helpers::selection_summary);
    let on_back = {
        let on = p.on_back.clone();
        Callback::from(move |_| {
            if let Some(cb) = on.as_ref() {
                cb.emit(());
            }
        })
    };

    html! {
      <section class="panel retro-menu persona-select" aria-labelledby="persona-title" onkeydown={on_keydown}>
        <p class="eyebrow">{crate::i18n::t("ux.persona_intro")}</p>
        <h2 id="persona-title">{ crate::i18n::t("persona.choose") }</h2>
        <p class="journey-mission">{crate::i18n::t("journey.mission")}</p>
        <div class="persona-layout">
          <div class="persona-grid" role="radiogroup" aria-labelledby="persona-title" id="persona-radios" ref={list_ref}>
            { for personas.iter().enumerate().map(|(i, per)| {
                html! {
                    <tile::PersonaTile
                        key={per.id.clone()}
                        index={i}
                        persona={per.clone()}
                        selected={Some(i) == (*selected)}
                        on_select={select_idx.clone()}
                    />
                }
            }) }
          </div>
          <preview::PersonaPreview persona={preview_persona} />
        </div>
        <p id="persona-helper" class="sr-only" aria-live="polite" aria-atomic="true">{live_msg}</p>
        <div class="controls persona-actions setup-actions">
          if p.on_back.is_some() {
            <button type="button" class="retro-btn-secondary setup-back-desktop" onclick={on_back.clone()}>{crate::i18n::t("store.menu.back")}</button>
          }
          <button type="button" id="persona-continue" class="retro-btn-primary" disabled={selected.is_none()} onclick={
            let on = p.on_continue.clone();
            Callback::from(move |_| if let Some(cb)=on.clone(){ cb.emit(()); })
          }>{ crate::i18n::t("ui.continue") }</button>
          if p.on_back.is_some() {
            <button type="button" class="retro-btn-secondary setup-back-mobile" onclick={on_back}>{crate::i18n::t("store.menu.back")}</button>
          }
        </div>
      </section>
    }
}
