use super::helpers::{
    initial_for, localized_desc, localized_name, multiplier_label, multiplier_value, stats_row,
};
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
pub fn persona_tile(props: &PersonaTileProps) -> Html {
    let idx_display = props.index + 1;
    let name = localized_name(&props.persona);
    let desc = localized_desc(&props.persona);
    let mult_value = multiplier_value(&props.persona);
    let mult_label = multiplier_label(&props.persona);

    let on_click = {
        let on_select = props.on_select.clone();
        let idx = props.index;
        Callback::from(move |_| on_select.emit(idx))
    };

    let sel_class = classes!("persona-tile", props.selected.then_some("selected"));

    html! {
      <div role="radio"
          class={sel_class}
          aria-checked={props.selected.to_string()}
          aria-describedby={if props.selected { Some(AttrValue::from("persona-preview")) } else { None }}
          tabindex={if props.selected { "0" } else { "-1" }}
          onclick={on_click}
          data-key={idx_display.to_string()}>
        <div class="persona-portrait" aria-hidden="true">
          <span class="portrait-initial">{ initial_for(&name) }</span>
        </div>
        <div class="persona-details">
          <div class="persona-title-row">
            <span class="persona-name">{ format!("{}{}) {}", idx_display, ")", name) }</span>
            <span class="persona-mult" aria-label={mult_label.clone()}>{ mult_value }</span>
          </div>
          <p class="persona-desc muted">{ desc }</p>
          <div class="persona-stats-row">
            { stats_row(&props.persona) }
          </div>
        </div>
      </div>
    }
}
