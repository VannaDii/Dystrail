use yew_router::prelude::*;
#[derive(Routable, PartialEq, Eq, Clone, Debug)]
pub enum Route {
    #[at("/")] Home,
    #[at("/settings")] Settings,
    #[not_found] #[at("/404")] NotFound,
}
