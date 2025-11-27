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
        html! { <button onclick={cb} class="retro-btn-choice">{ format!("{}{}", i + 1, ") ") }{ c.label.clone() }</button> }
    });
    html! {
        <section class="panel retro-encounter encounter-panel" role="dialog" aria-modal="false" aria-labelledby="enc-title">
            <header class="section-header">
                <h2 id="enc-title">{ p.encounter.name.clone() }</h2>
            </header>
            <div class="encounter-desc">
                <p>{ p.encounter.desc.clone() }</p>
            </div>
            <footer class="panel-footer">
                { for buttons }
            </footer>
        </section>
    }
}
