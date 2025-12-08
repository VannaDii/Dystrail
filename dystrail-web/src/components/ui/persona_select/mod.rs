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
}

#[function_component(PersonaSelect)]
pub fn persona_select(p: &PersonaSelectProps) -> Html {
    let personas = use_state(Vec::<Persona>::new);
    let selected = use_state(|| None::<usize>);
    let live_msg = use_state(String::new);
    let list_ref = use_node_ref();

    {
        let personas = personas.clone();
        use_effect_with((), move |()| {
            let data = include_str!("../../../../static/assets/data/personas.json");
            let list = PersonasList::from_json(data).unwrap_or_else(|_| PersonasList::empty());
            personas.set(list.0);
            || {}
        });
    }

    let select_idx = {
        let selected = selected.clone();
        let personas_state = personas.clone();
        let on_selected = p.on_selected.clone();
        let live_msg = live_msg.clone();
        Callback::from(move |idx: usize| {
            if idx < personas_state.len() {
                selected.set(Some(idx));
                let per = &personas_state[idx];
                let msg = format!(
                    "{} {}. {} ${}",
                    crate::i18n::t("menu.selected"),
                    per.name,
                    crate::i18n::t("persona.selected_budget_prefix"),
                    per.start.budget
                );
                live_msg.set(msg);
                if let Some(cb) = on_selected.clone() {
                    cb.emit(per.clone());
                }
            }
        })
    };

    {
        let list_ref = list_ref.clone();
        use_effect_with(*selected, move |sel| {
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
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
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
                    select_idx.emit(idx);
                }
                e.prevent_default();
            }
        })
    };

    let preview_persona = selected.and_then(|i| personas.get(i)).cloned();

    html! {
      <section class="panel retro-menu persona-select" aria-labelledby="persona-title" onkeydown={on_keydown}>
        <h2 id="persona-title">{ crate::i18n::t("persona.choose") }</h2>
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
        <div class="controls">
          <button id="persona-continue" disabled={selected.is_none()} onclick={
            let on = p.on_continue.clone();
            Callback::from(move |_| if let Some(cb)=on.clone(){ cb.emit(()); })
          }>{ crate::i18n::t("ui.continue") }</button>
        </div>
        <p id="persona-helper" aria-live="polite" class="muted">{ (*live_msg).clone() }</p>
      </section>
    }
}
