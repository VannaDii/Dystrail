use crate::simple_component;

simple_component!(
    HoverGallery,
    hover_gallery_component,
    div,
    ["hover-gallery", "grid", "gap-2"],
    Option::<&'static str>::None
);
