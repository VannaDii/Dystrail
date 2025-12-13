use crate::simple_component;

simple_component!(
    Hover3d,
    hover3d_component,
    div,
    ["hover-3d", "transform", "transition-transform"],
    Option::<&'static str>::None
);
