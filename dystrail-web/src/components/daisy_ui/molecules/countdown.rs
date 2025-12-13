use crate::components::daisy_ui::foundation as f;

fn format_duration(mut millis: u64) -> String {
    let hours = millis / 3_600_000;
    millis -= hours * 3_600_000;
    let minutes = millis / 60_000;
    millis -= minutes * 60_000;
    let seconds = millis / 1_000;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct CountdownProps {
    pub millis: u64,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub label: Option<f::AttrValue>,
}

#[f::function_component(Countdown)]
pub fn countdown(props: &CountdownProps) -> f::Html {
    let class = f::class_list(&["countdown", "font-mono"], &props.class);
    let formatted = format_duration(props.millis);
    f::html! {
        <div class={class} role="timer" aria-live="polite">
            { props.label.as_ref().map(|label| f::html! { <span class="me-2">{ label.clone() }</span> }).unwrap_or_default() }
            <span aria-label={format!("{formatted} remaining")}>{ formatted }</span>
        </div>
    }
}
