use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct MockupPhoneProps {
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(MockupPhone)]
pub fn mockup_phone(props: &MockupPhoneProps) -> f::Html {
    let class = f::class_list(&["mockup-phone"], &props.class);
    f::html! {
        <div class={class}>
            <div class="camera"></div>
            <div class="display">
                <div class="artboard phone-1">
                    { for props.children.iter() }
                </div>
            </div>
        </div>
    }
}
