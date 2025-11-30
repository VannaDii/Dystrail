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

fn initial_for(name: &str) -> String {
    name.chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect::<String>())
}

fn mini_stat(label: String, value: i32) -> Html {
    html! {
        <span class="mini-stat" role="text">
            <span class="mini-stat-icon" aria-hidden="true">{ label }</span>
            <span class="mini-stat-value">{ value }</span>
        </span>
    }
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
            let data = include_str!("../../../static/assets/data/personas.json");
            let list = PersonasList::from_json(data).unwrap_or_else(|_| PersonasList::empty());
            personas.set(list.0);
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

    // Right side preview content derived from selected persona
    let preview = selected.and_then(|i| personas.get(i)).cloned();

    // Build i18n preview line and multiplier
    let preview_line = preview.as_ref().map_or_else(String::new, |per| {
        let sup = per.start.supplies.to_string();
        let cred = per.start.credibility.to_string();
        let san = per.start.sanity.to_string();
        let mor = per.start.morale.to_string();
        let allies = per.start.allies.to_string();
        let budget = per.start.budget.to_string();
        let mut m = std::collections::BTreeMap::new();
        m.insert("sup", sup.as_str());
        m.insert("cred", cred.as_str());
        m.insert("san", san.as_str());
        m.insert("mor", mor.as_str());
        m.insert("allies", allies.as_str());
        m.insert("budget", budget.as_str());
        crate::i18n::tr("persona.preview", Some(&m))
    });
    let mult_line = preview.as_ref().map_or_else(String::new, |per| {
        let mult = format!("{:.1}", per.score_mult);
        let mut m = std::collections::BTreeMap::new();
        m.insert("mult", mult.as_str());
        crate::i18n::tr("persona.mult", Some(&m))
    });
    let mods_text = preview.as_ref().map_or_else(String::new, |per| {
        let mut lines: Vec<String> = vec![];
        if per.mods.receipt_find_pct != 0 {
            let pct = per.mods.receipt_find_pct.to_string();
            let mut m = std::collections::BTreeMap::new();
            m.insert("pct", pct.as_str());
            lines.push(crate::i18n::tr("persona.mods.receipts_pct", Some(&m)));
        }
        if per.mods.store_discount_pct != 0 {
            let pct = per.mods.store_discount_pct.to_string();
            let mut m = std::collections::BTreeMap::new();
            m.insert("pct", pct.as_str());
            lines.push(crate::i18n::tr("persona.mods.store_discount_pct", Some(&m)));
        }
        if per.mods.bribe_discount_pct != 0 {
            let pct = per.mods.bribe_discount_pct.to_string();
            let mut m = std::collections::BTreeMap::new();
            m.insert("pct", pct.as_str());
            lines.push(crate::i18n::tr("persona.mods.bribe_discount_pct", Some(&m)));
        }
        if per.mods.eo_heat_pct != 0 {
            let pct = per.mods.eo_heat_pct.to_string();
            let mut m = std::collections::BTreeMap::new();
            m.insert("pct", pct.as_str());
            lines.push(crate::i18n::tr("persona.mods.eo_heat_pct", Some(&m)));
        }
        if per.mods.satire_sustain {
            lines.push(crate::i18n::t("persona.mods.satire_sustain"));
        }
        lines.join(" · ")
    });

    html! {
      <section class="panel retro-menu persona-select" aria-labelledby="persona-title" onkeydown={on_keydown}>
        <h2 id="persona-title">{ crate::i18n::t("persona.choose") }</h2>
        <div class="persona-layout">
          <div class="persona-grid" role="radiogroup" aria-labelledby="persona-title" id="persona-radios" ref={list_ref}>
            { for personas.iter().enumerate().map(|(i, per)| {
                let idx = i + 1; // 1-based for UI
                let is_sel = Some(i) == (*selected);
                let sel_class = classes!("persona-tile", if is_sel { Some("selected") } else { None });
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
                let mult_value = format!("×{:.1}", per.score_mult);
                let mult_label = {
                    let mut m = std::collections::BTreeMap::new();
                    m.insert("mult", mult_value.as_str());
                    crate::i18n::tr("persona.mult", Some(&m))
                };
                html!{
                  <div role="radio"
                      class={sel_class}
                      aria-checked={is_sel.to_string()}
                      aria-describedby={if is_sel { Some(AttrValue::from("persona-preview")) } else { None }}
                      tabindex={if is_sel { "0" } else { "-1" }}
                      onclick={on_click}
                      data-key={idx.to_string()}>
                    <div class="persona-portrait" aria-hidden="true">
                      <span class="portrait-initial">{ initial_for(&disp_name) }</span>
                    </div>
                    <div class="persona-details">
                      <div class="persona-title-row">
                        <span class="persona-name">{ format!("{}{}) {}", idx, ")", disp_name) }</span>
                        <span class="persona-mult" aria-label={mult_label.clone()}>{ mult_value }</span>
                      </div>
                      <p class="persona-desc muted">{ disp_desc }</p>
                      <div class="persona-stats-row">
                        { mini_stat(crate::i18n::t("stats.sup_short"), per.start.supplies) }
                        { mini_stat(crate::i18n::t("stats.sanity_short"), per.start.sanity) }
                        { mini_stat(crate::i18n::t("stats.cred_short"), per.start.credibility) }
                        { mini_stat(crate::i18n::t("stats.mor_short"), per.start.morale) }
                        { mini_stat(crate::i18n::t("stats.allies_short"), per.start.allies) }
                        { mini_stat(crate::i18n::t("persona.selected_budget_prefix"), per.start.budget) }
                      </div>
                    </div>
                  </div>
                }
            })}
          </div>
          <aside id="persona-preview" class="persona-preview-card" aria-live="polite">
            { if let Some(per) = preview {
                html!{
                  <>
                    <div class="persona-preview-header">
                      <div class="persona-portrait" aria-hidden="true">
                        <span class="portrait-initial">{ initial_for(&per.name) }</span>
                      </div>
                      <div>
                        <h3 class="persona-name">{ {
                            let key = format!("persona.{}.name", per.id);
                            let v = crate::i18n::t(&key); if v == key { per.name.clone() } else { v }
                          } }
                        </h3>
                        <p class="muted">{ {
                            let key = format!("persona.{}.desc", per.id);
                            let v = crate::i18n::t(&key); if v == key { per.desc.clone() } else { v }
                          } }
                        </p>
                      </div>
                    </div>
                    <div class="persona-preview-stats">
                      { mini_stat(crate::i18n::t("stats.sup_short"), per.start.supplies) }
                      { mini_stat(crate::i18n::t("stats.sanity_short"), per.start.sanity) }
                      { mini_stat(crate::i18n::t("stats.cred_short"), per.start.credibility) }
                      { mini_stat(crate::i18n::t("stats.mor_short"), per.start.morale) }
                      { mini_stat(crate::i18n::t("stats.allies_short"), per.start.allies) }
                      { mini_stat(crate::i18n::t("persona.selected_budget_prefix"), per.start.budget) }
                    </div>
                    <p class="muted">{ preview_line }</p>
                    <p class="muted">{ mult_line }</p>
                    <div class="persona-mods">{ mods_text }</div>
                  </>
                }
            } else { html!{ <p class="muted">{ crate::i18n::t("persona.preview_wait") }</p> } } }
          </aside>
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
