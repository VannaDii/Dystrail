mod helpers;
mod preview;
mod tile;

use crate::game::personas::{Persona, PersonasList};
#[cfg(target_arch = "wasm32")]
use crate::input::{numeric_code_to_index, numeric_key_to_index};
#[cfg(target_arch = "wasm32")]
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

fn build_selection_callback(
    personas: UseStateHandle<Vec<Persona>>,
    selected: UseStateHandle<Option<usize>>,
    live_msg: UseStateHandle<String>,
    on_selected: Option<Callback<Persona>>,
) -> Callback<usize> {
    Callback::from(move |idx: usize| {
        apply_selection_to_state(&personas, idx, &selected, &live_msg, on_selected.as_ref());
    })
}

#[function_component(PersonaSelect)]
pub fn persona_select(p: &PersonaSelectProps) -> Html {
    let personas = use_state(initial_personas);
    let selected = use_state(|| None::<usize>);
    let live_msg = use_state(String::new);
    let list_ref = use_node_ref();

    #[cfg(target_arch = "wasm32")]
    {
        let personas = personas.clone();
        use_effect_with((), move |()| {
            let data = include_str!("../../../../static/assets/data/personas.json");
            let list = PersonasList::from_json(data).unwrap_or_else(|_| PersonasList::empty());
            personas.set(list.0);
            || {}
        });
    }

    let select_idx = build_selection_callback(
        personas.clone(),
        selected.clone(),
        live_msg.clone(),
        p.on_selected.clone(),
    );

    #[cfg(target_arch = "wasm32")]
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
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = list_ref;
    }

    let on_keydown = {
        let selected = selected.clone();
        let select_idx = select_idx.clone();
        let on_continue = p.on_continue.clone();
        #[cfg(target_arch = "wasm32")]
        {
            Callback::from(move |e: KeyboardEvent| {
                let key = e.key();
                if let Some(n) =
                    numeric_key_to_index(&key).or_else(|| numeric_code_to_index(&e.code()))
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
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (selected, select_idx, on_continue);
            Callback::from(|_e: KeyboardEvent| {})
        }
    };

    let preview_persona = selected.and_then(|i| personas.get(i)).cloned();
    let on_continue_click = {
        let on = p.on_continue.clone();
        #[cfg(target_arch = "wasm32")]
        {
            Callback::from(move |_| {
                if let Some(cb) = on.clone() {
                    cb.emit(())
                }
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = on;
            Callback::from(|_| {})
        }
    };

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
          <button
            id="persona-continue"
            disabled={selected.is_none()}
            onclick={on_continue_click}
          >
            { crate::i18n::t("ui.continue") }
          </button>
        </div>
        <p id="persona-helper" aria-live="polite" class="muted">{ (*live_msg).clone() }</p>
      </section>
    }
}

fn initial_personas() -> Vec<Persona> {
    #[cfg(target_arch = "wasm32")]
    {
        Vec::new()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let data = include_str!("../../../../static/assets/data/personas.json");
        PersonasList::from_json(data)
            .unwrap_or_else(|_| PersonasList::empty())
            .0
    }
}

fn apply_selection(personas: &[Persona], idx: usize) -> Option<(Persona, String)> {
    let persona = personas.get(idx)?;
    Some((persona.clone(), selection_message(persona)))
}

fn apply_selection_to_state(
    personas: &UseStateHandle<Vec<Persona>>,
    idx: usize,
    selected: &UseStateHandle<Option<usize>>,
    live_msg: &UseStateHandle<String>,
    on_selected: Option<&Callback<Persona>>,
) {
    if let Some((persona, message)) = apply_selection(personas, idx) {
        selected.set(Some(idx));
        live_msg.set(message);
        if let Some(cb) = on_selected {
            cb.emit(persona);
        }
    }
}

fn selection_message(persona: &Persona) -> String {
    format!(
        "{} {}. {} ${}",
        crate::i18n::t("menu.selected"),
        persona.name,
        crate::i18n::t("persona.selected_budget_prefix"),
        persona.start.budget
    )
}

#[cfg(test)]
mod tests {
    use super::preview::PersonaPreview;
    use super::tile::{PersonaTile, PersonaTileProps};
    use super::*;
    use crate::game::personas::{PersonaMods, PersonaStart, PersonasList};
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    fn sample_persona() -> Persona {
        let data = include_str!("../../../../static/assets/data/personas.json");
        PersonasList::from_json(data)
            .unwrap_or_else(|_| PersonasList::empty())
            .0
            .first()
            .cloned()
            .unwrap_or_else(|| Persona {
                id: String::from("fallback"),
                name: String::from("Fallback"),
                desc: String::new(),
                score_mult: 1.0,
                start: PersonaStart::default(),
                mods: PersonaMods::default(),
            })
    }

    #[test]
    fn persona_preview_renders_placeholder_when_empty() {
        crate::i18n::set_lang("en");
        let html = block_on(
            LocalServerRenderer::<PersonaPreview>::with_props(preview::PersonaPreviewProps {
                persona: None,
            })
            .render(),
        );
        assert!(html.contains("persona-preview-card"));
    }

    #[test]
    fn persona_preview_renders_persona_details() {
        crate::i18n::set_lang("en");
        let persona = sample_persona();
        let html = block_on(
            LocalServerRenderer::<PersonaPreview>::with_props(preview::PersonaPreviewProps {
                persona: Some(persona.clone()),
            })
            .render(),
        );
        assert!(html.contains("persona-preview"));
        assert!(html.contains(&persona.name));
    }

    #[test]
    fn persona_tile_renders_selected_state() {
        crate::i18n::set_lang("en");
        let persona = sample_persona();
        let props = PersonaTileProps {
            index: 0,
            persona,
            selected: true,
            on_select: Callback::noop(),
        };
        let html = block_on(LocalServerRenderer::<PersonaTile>::with_props(props).render());
        assert!(html.contains("persona-tile"));
        assert!(html.contains("selected"));
    }

    #[test]
    fn persona_select_renders_grid() {
        crate::i18n::set_lang("en");
        let props = PersonaSelectProps {
            on_selected: Some(Callback::noop()),
            on_continue: Some(Callback::noop()),
        };
        let html = block_on(LocalServerRenderer::<PersonaSelect>::with_props(props).render());
        assert!(html.contains("persona-select"));
    }

    #[test]
    fn apply_selection_builds_message_and_persona() {
        crate::i18n::set_lang("en");
        let persona = sample_persona();
        let personas = vec![persona.clone()];
        let selection = apply_selection(&personas, 0).expect("selection should resolve");
        assert_eq!(selection.0.id, persona.id);
        assert!(selection.1.contains(&persona.name));
    }

    #[test]
    fn selection_message_includes_budget() {
        crate::i18n::set_lang("en");
        let persona = sample_persona();
        let message = selection_message(&persona);
        assert!(message.contains(&persona.name));
        assert!(message.contains(&persona.start.budget.to_string()));
    }

    #[test]
    fn apply_selection_to_state_updates_selection_and_message() {
        #[function_component(SelectionHarness)]
        fn selection_harness() -> Html {
            crate::i18n::set_lang("en");
            let persona = sample_persona();
            let personas = use_state(|| vec![persona.clone()]);
            let selected = use_state(|| None::<usize>);
            let live_msg = use_state(String::new);
            let captured = use_mut_ref(|| None::<String>);
            let on_selected = {
                let captured = captured.clone();
                Some(Callback::from(move |p: Persona| {
                    *captured.borrow_mut() = Some(p.name);
                }))
            };

            let invoked = use_mut_ref(|| false);
            if !*invoked.borrow() {
                *invoked.borrow_mut() = true;
                apply_selection_to_state(&personas, 0, &selected, &live_msg, on_selected.as_ref());
            }

            let selected_label = (*selected).map_or_else(|| "none".to_string(), |v| v.to_string());
            let persona_label = captured
                .borrow()
                .clone()
                .unwrap_or_else(|| "none".to_string());
            html! {
                <div
                    data-selected={selected_label}
                    data-message={(*live_msg).clone()}
                    data-persona={persona_label}
                />
            }
        }

        let html = block_on(LocalServerRenderer::<SelectionHarness>::new().render());
        assert!(html.contains("data-persona=\""));
        assert!(!html.contains("data-persona=\"none\""));
    }

    #[test]
    fn selection_callback_emits_persona() {
        #[function_component(CallbackHarness)]
        fn callback_harness() -> Html {
            crate::i18n::set_lang("en");
            let persona = sample_persona();
            let personas = use_state(|| vec![persona.clone()]);
            let selected = use_state(|| None::<usize>);
            let live_msg = use_state(String::new);
            let captured = use_mut_ref(|| None::<String>);
            let on_selected = {
                let captured = captured.clone();
                Some(Callback::from(move |p: Persona| {
                    *captured.borrow_mut() = Some(p.name);
                }))
            };
            let invoked = use_mut_ref(|| false);
            let select_cb = build_selection_callback(personas, selected, live_msg, on_selected);
            if !*invoked.borrow() {
                *invoked.borrow_mut() = true;
                select_cb.emit(0);
            }
            let persona_label = captured
                .borrow()
                .clone()
                .unwrap_or_else(|| "none".to_string());
            html! { <div data-persona={persona_label} /> }
        }

        let html = block_on(LocalServerRenderer::<CallbackHarness>::new().render());
        assert!(!html.contains("data-persona=\"none\""));
    }
}
