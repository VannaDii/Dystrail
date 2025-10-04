use crate::game::data::Encounter;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub encounter: Encounter,
    pub on_choice: Callback<usize>,
}

#[function_component(EncounterCard)]
pub fn encounter_card(p: &Props) -> Html {
    let buttons = p.encounter.choices.iter().enumerate().map(|(i, c)| {
        let cb = {
            let on_choice = p.on_choice.clone();
            Callback::from(move |_| on_choice.emit(i))
        };
        html! { <button onclick={cb} class="retro-btn-choice">{ c.label.clone() }</button> }
    });
    html! {
        <section class="panel retro-encounter" role="dialog" aria-modal="false" aria-labelledby="enc-title">
            <h2 id="enc-title">{ p.encounter.name.clone() }</h2>
            <div class="encounter-desc">
                <p>{ p.encounter.desc.clone() }</p>
            </div>
            <div class="controls">{ for buttons }</div>
        </section>
    }
}
