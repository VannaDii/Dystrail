use crate::i18n::t;
use yew::prelude::*;

#[function_component(Footer)]
pub fn footer() -> Html {
    crate::i18n::use_language();
    let node = html! {
        <footer>{ t("footer.copyright") }</footer>
    };
    node
}
