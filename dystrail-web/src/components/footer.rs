use crate::i18n::t;
use yew::prelude::*;

#[function_component(Footer)]
pub fn footer() -> Html {
    let node = html! {
        <footer>{ t("footer.copyright") }</footer>
    };
    node
}
