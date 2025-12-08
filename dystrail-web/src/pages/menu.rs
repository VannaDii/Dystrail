use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    StartRun,
    CampPreview,
    OpenSave,
    OpenSettings,
    Reset,
}

#[derive(Properties, Clone, PartialEq)]
pub struct MenuPageProps {
    pub code: AttrValue,
    pub logo_src: AttrValue,
    pub on_action: Callback<MenuAction>,
}

#[function_component(MenuPage)]
pub fn menu_page(props: &MenuPageProps) -> Html {
    let on_select = {
        let on_action = props.on_action.clone();
        Callback::from(move |idx: u8| match idx {
            1 => on_action.emit(MenuAction::StartRun),
            2 => on_action.emit(MenuAction::CampPreview),
            7 => on_action.emit(MenuAction::OpenSave),
            8 => on_action.emit(MenuAction::OpenSettings),
            0 => on_action.emit(MenuAction::Reset),
            _ => {}
        })
    };

    html! {
            <section class="panel retro-menu">
                <header class="retro-header" role="banner">
                    <div class="header-center">
                        <pre class="ascii-art">
    { "═══════════════════════════════" }<br/>
    { "D Y S T R A I L" }<br/>
    { "A Political Survival Adventure" }<br/>
    { "═══════════════════════════════" }
                        </pre>
                    </div>
                    <p class="muted" aria-live="polite">
                        { format!("{seed_label} {code}", seed_label = crate::i18n::t("game.seed_label"), code = props.code.clone()) }
                    </p>
                </header>
                <img src={props.logo_src.clone()} alt="Dystrail" loading="lazy" style="width:min(520px,80vw)"/>
                <crate::components::ui::main_menu::MainMenu seed_text={Some(props.code.to_string())} on_select={Some(on_select)} />
            </section>
        }
}
