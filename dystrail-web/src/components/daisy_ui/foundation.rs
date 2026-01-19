pub use wasm_bindgen::JsCast;
pub use wasm_bindgen::prelude::Closure;
pub use web_sys::{
    Event, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, InputEvent, MouseEvent,
};
pub use yew::classes;
pub use yew::function_component;
pub use yew::html::TargetCast;
pub use yew::prelude::{AttrValue, Callback, Children, Classes, Html, Properties, html};
pub use yew::{use_effect_with, use_state};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DaisyColor {
    Neutral,
    #[default]
    Primary,
    Secondary,
    Accent,
    Info,
    Success,
    Warning,
    Error,
}

impl DaisyColor {
    const fn suffix(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Accent => "accent",
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }

    #[must_use]
    pub fn class(self, prefix: &str) -> String {
        format!("{prefix}-{}", self.suffix())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DaisySize {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

impl DaisySize {
    const fn suffix(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }

    #[must_use]
    pub fn class(self, prefix: &str) -> String {
        format!("{prefix}-{}", self.suffix())
    }
}

#[derive(Properties, PartialEq, Clone, Default)]
pub struct BaseProps {
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub aria_label: Option<AttrValue>,
    #[prop_or_default]
    pub role: Option<AttrValue>,
    #[prop_or_default]
    pub children: Children,
}

#[must_use]
pub fn class_list(base: &[&'static str], extra: &Classes) -> Classes {
    let mut classes = Classes::new();
    for item in base {
        classes.push(*item);
    }
    classes.push(extra.clone());
    classes
}

#[must_use]
pub fn attr_value(opt: &Option<AttrValue>) -> Option<AttrValue> {
    opt.clone()
}

#[macro_export]
macro_rules! simple_component {
    ($component:ident, $func:ident, $tag:ident, [$($base:expr),*], $default_role:expr) => {
        #[ $crate::components::daisy_ui::foundation::function_component($component)]
        pub fn $func(props: &$crate::components::daisy_ui::foundation::BaseProps) -> yew::Html {
            use $crate::components::daisy_ui::foundation::{attr_value, class_list};
            let class = class_list(&[$($base),*], &props.class);
            let id = attr_value(&props.id);
            let aria_label = attr_value(&props.aria_label);
            let role: Option<yew::AttrValue> = props
                .role
                .clone()
                .or_else(|| $default_role.map(yew::AttrValue::from));
            yew::html! {
                <$tag id={id} class={class} aria-label={aria_label} role={role}>
                    { for props.children.iter() }
                </$tag>
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::{DaisyColor, DaisySize, attr_value, class_list};
    use yew::Classes;

    #[test]
    fn daisy_color_classes_include_prefix_and_suffix() {
        let colors = [
            DaisyColor::Neutral,
            DaisyColor::Primary,
            DaisyColor::Secondary,
            DaisyColor::Accent,
            DaisyColor::Info,
            DaisyColor::Success,
            DaisyColor::Warning,
            DaisyColor::Error,
        ];
        for color in colors {
            let class = color.class("text");
            assert!(class.starts_with("text-"));
        }
    }

    #[test]
    fn daisy_size_classes_include_prefix_and_suffix() {
        let sizes = [
            DaisySize::Xs,
            DaisySize::Sm,
            DaisySize::Md,
            DaisySize::Lg,
            DaisySize::Xl,
        ];
        for size in sizes {
            let class = size.class("btn");
            assert!(class.starts_with("btn-"));
        }
    }

    #[test]
    fn class_list_combines_base_and_extra() {
        let extra = Classes::from("mx-1");
        let classes = class_list(&["btn", "btn-primary"], &extra);
        let rendered = classes.to_string();
        assert!(rendered.contains("btn"));
        assert!(rendered.contains("btn-primary"));
        assert!(rendered.contains("mx-1"));
    }

    #[test]
    fn attr_value_clones_optional_attr() {
        let value = Some(yew::AttrValue::from("test"));
        let cloned = attr_value(&value);
        assert_eq!(cloned.as_deref(), Some("test"));
    }
}
