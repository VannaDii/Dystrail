use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

use crate::game::personas::{Persona, PersonasList};
use crate::input::{numeric_code_to_index, numeric_key_to_index};

#[derive(Properties, PartialEq, Clone)]
pub struct PersonaSelectProps {
    #[prop_or_default]
    pub on_selected: Option<Callback<Persona>>, // called when selection changes
    #[prop_or_default]
    pub on_continue: Option<Callback<()>>, // called when user continues (0)
}

#[function_component(PersonaSelect)]
pub fn persona_select(p: &PersonaSelectProps) -> Html {
    let personas = use_state(Vec::<Persona>::new);
    let selected = use_state(|| None::<usize>);
    let live_msg = use_state(String::new);
    let list_ref = use_node_ref();

    // Load personas once
    {
        let personas = personas.clone();
        use_effect_with((), move |()| {
            wasm_bindgen_futures::spawn_local(async move {
                let list = PersonasList::load().await;
                personas.set(list.0);
            });
            || {}
        });
    }

    // Selection handler
    let select_idx = {
        let selected = selected.clone();
        let personas_state = personas.clone();
        let on_selected = p.on_selected.clone();
        let live_msg = live_msg.clone();
        Callback::from(move |idx: usize| {
            if idx < personas_state.len() {
                selected.set(Some(idx));
                // announce
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

    // Keyboard handling (1..6 select, 0 continue)
    // Move DOM focus to selected item
    {
        let list_ref = list_ref.clone();
        use_effect_with(*selected, move |sel| {
            if let Some(i) = *sel
                && let Some(list) = list_ref.cast::<web_sys::Element>() {
                let sel = format!("[role='radio'][data-key='{}']", i + 1);
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
        let selected = selected.clone();
        let select_idx = select_idx.clone();
        let on_continue = p.on_continue.clone();
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            if let Some(n) = numeric_key_to_index(&key).or_else(|| numeric_code_to_index(&e.code()))
            {
                if n == 0 {
                    if selected.is_some()
                        && let Some(cb) = on_continue.clone() {
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

    // Right side preview content derived from selected persona
    let preview = selected.and_then(|i| personas.get(i)).cloned();

    // Build i18n preview line and multiplier
    let preview_line = if let Some(per) = preview.as_ref() {
        let sup = per.start.supplies.to_string();
        let cred = per.start.credibility.to_string();
        let san = per.start.sanity.to_string();
        let mor = per.start.morale.to_string();
        let allies = per.start.allies.to_string();
        let budget = per.start.budget.to_string();
        let mut m = std::collections::HashMap::new();
        m.insert("sup", sup.as_str());
        m.insert("cred", cred.as_str());
        m.insert("san", san.as_str());
        m.insert("mor", mor.as_str());
        m.insert("allies", allies.as_str());
        m.insert("budget", budget.as_str());
        crate::i18n::tr("persona.preview", Some(&m))
    } else {
        String::new()
    };
    let mult_line = if let Some(per) = preview.as_ref() {
        let mult = format!("{:.1}", per.score_mult);
        let mut m = std::collections::HashMap::new();
        m.insert("mult", mult.as_str());
        crate::i18n::tr("persona.mult", Some(&m))
    } else {
        String::new()
    };
    let mods_text = if let Some(per) = preview.as_ref() {
        let mut lines: Vec<String> = vec![];
        if per.mods.receipt_find_pct != 0 {
            let pct = per.mods.receipt_find_pct.to_string();
            let mut m = std::collections::HashMap::new();
            m.insert("pct", pct.as_str());
            lines.push(crate::i18n::tr("persona.mods.receipts_pct", Some(&m)));
        }
        if per.mods.store_discount_pct != 0 {
            let pct = per.mods.store_discount_pct.to_string();
            let mut m = std::collections::HashMap::new();
            m.insert("pct", pct.as_str());
            lines.push(crate::i18n::tr("persona.mods.store_discount_pct", Some(&m)));
        }
        if per.mods.bribe_discount_pct != 0 {
            let pct = per.mods.bribe_discount_pct.to_string();
            let mut m = std::collections::HashMap::new();
            m.insert("pct", pct.as_str());
            lines.push(crate::i18n::tr("persona.mods.bribe_discount_pct", Some(&m)));
        }
        if per.mods.eo_heat_pct != 0 {
            let pct = per.mods.eo_heat_pct.to_string();
            let mut m = std::collections::HashMap::new();
            m.insert("pct", pct.as_str());
            lines.push(crate::i18n::tr("persona.mods.eo_heat_pct", Some(&m)));
        }
        if per.mods.satire_sustain {
            lines.push(crate::i18n::t("persona.mods.satire_sustain"));
        }
        lines.join(" · ")
    } else {
        String::new()
    };

    html! {
      <section class="panel retro-menu" aria-labelledby="persona-title" onkeydown={on_keydown}>
        <h2 id="persona-title">{ crate::i18n::t("persona.choose") }</h2>
        <div class="ot-menu" style="display:flex; gap:20px; align-items: flex-start;">
          <div style="flex:1;">
            <ul role="radiogroup" aria-labelledby="persona-title" id="persona-radios" ref={list_ref} style="list-style:none; padding:0; margin:0;">
              { for personas.iter().enumerate().map(|(i, per)| {
                  let idx = i + 1; // 1-based for UI
                  let is_sel = Some(i) == (*selected);
                  let sel_class = if is_sel { "ot-menuitem selected" } else { "ot-menuitem" };
                  let on_click = {
                    let select_idx = select_idx.clone();
                    Callback::from(move |_| select_idx.emit(i))
                  };
                  // Resolve localized name/desc, fallback to data
                  let name_key = format!("persona.{}.name", per.id);
                  let desc_key = format!("persona.{}.desc", per.id);
                  let lname = crate::i18n::t(&name_key);
                  let ldesc = crate::i18n::t(&desc_key);
                  let disp_name = if lname == name_key { per.name.clone() } else { lname };
                  let disp_desc = if ldesc == desc_key { per.desc.clone() } else { ldesc };
                  html!{
                    <li role="radio"
                        class={sel_class}
                        aria-checked={is_sel.to_string()}
                        aria-describedby={if is_sel { Some(AttrValue::from("persona-preview")) } else { None }}
                        tabindex={if is_sel { "0" } else { "-1" }}
                        onclick={on_click}
                        data-key={idx.to_string()}>
                      <span class="num">{ format!("{}{})", idx, ")") }</span>
                      <span class="label">{ format!("{disp_name} — {disp_desc}") }</span>
                    </li>
                  }
              })}
            </ul>
            <div class="controls" style="margin-top:16px;">
              <button id="persona-continue" disabled={selected.is_none()} onclick={
                let on = p.on_continue.clone();
                Callback::from(move |_| if let Some(cb)=on.clone(){ cb.emit(()); })
              }>{ crate::i18n::t("ui.continue") }</button>
            </div>
            <p id="persona-helper" aria-live="polite" class="muted">{ (*live_msg).clone() }</p>
          </div>
          <aside id="persona-preview" style="flex:1; border-left:2px solid var(--panel-border); padding-left:12px;">
            { if let Some(per) = preview {
                html!{
                  <>
                    <h3>{ {
                        let key = format!("persona.{}.name", per.id);
                        let v = crate::i18n::t(&key); if v == key { per.name.clone() } else { v }
                      } }
                    </h3>
                    <p class="muted">{ {
                        let key = format!("persona.{}.desc", per.id);
                        let v = crate::i18n::t(&key); if v == key { per.desc.clone() } else { v }
                      } }
                    </p>
                    <p>{ preview_line }</p>
                    <p>{ mult_line }</p>
                    <div><p>{ mods_text }</p></div>
                  </>
                }
            } else { html!{ <p class="muted">{ crate::i18n::t("persona.preview_wait") }</p> } } }
          </aside>
        </div>
      </section>
    }
}
