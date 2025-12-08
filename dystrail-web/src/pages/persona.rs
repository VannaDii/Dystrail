use crate::game::personas::Persona;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct PersonaPageProps {
    pub on_selected: Callback<Persona>,
    pub on_continue: Callback<()>,
}

impl PartialEq for PersonaPageProps {
    fn eq(&self, _other: &Self) -> bool {
        // Always re-render; callbacks are not comparable
        false
    }
}

#[function_component(PersonaPage)]
pub fn persona_page(props: &PersonaPageProps) -> Html {
    html! {
      <section class="panel retro-menu">
        <crate::components::ui::persona_select::PersonaSelect
            on_selected={Some(props.on_selected.clone())}
            on_continue={Some(props.on_continue.clone())}
        />
      </section>
    }
}
