use super::helpers::localized_name;
use crate::game::personas::Persona;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct PersonaTileProps {
    pub index: usize,
    pub persona: Persona,
    pub selected: bool,
    pub on_select: Callback<usize>,
}

#[function_component(PersonaTile)]
pub fn persona_tile(p: &PersonaTileProps) -> Html {
    let cb = p.on_select.clone();
    let index = p.index;
    let on_click = Callback::from(move |_| cb.emit(index));
    html! { <button type="button" role="radio" class={classes!("persona-tile", p.selected.then_some("selected"))}
        aria-checked={p.selected.to_string()} onclick={on_click} data-key={(index+1).to_string()}>
        <span class={classes!("portrait-art", format!("portrait-{}", p.persona.id))} aria-hidden="true"></span>
        <span class="persona-name">{localized_name(&p.persona)}</span>
    </button> }
}
