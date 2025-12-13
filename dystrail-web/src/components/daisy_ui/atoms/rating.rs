use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct RatingProps {
    #[prop_or(5)]
    pub max: u32,
    #[prop_or_default]
    pub value: u32,
    #[prop_or_default]
    pub read_only: bool,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_change: f::Callback<u32>,
}

#[f::function_component(Rating)]
pub fn rating(props: &RatingProps) -> f::Html {
    let max = props.max.max(1);
    let class = f::class_list(&["rating"], &props.class);
    f::html! {
        <div class={class} role="radiogroup" aria-label="Rating">
            { for (1..=max).map(|idx| {
                let checked = idx == props.value;
                let on_change = {
                    let cb = props.on_change.clone();
                    f::Callback::from(move |_| cb.emit(idx))
                };
                f::html! {
                    <input
                        type="radio"
                        name="rating"
                        class="mask mask-star-2 bg-amber-400"
                        checked={checked}
                        disabled={props.read_only}
                        aria-label={format!("{idx} star rating")}
                        onchange={on_change}
                    />
                }
            }) }
        </div>
    }
}
