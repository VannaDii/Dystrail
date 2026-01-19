mod interactions;
mod option;
mod view;
mod view_model;

pub use view::{RoutePromptCard, RoutePromptCardProps};

#[cfg(test)]
mod tests {
    use super::view_model::build_route_prompt_viewmodel;
    use super::{RoutePromptCard, RoutePromptCardProps};
    use crate::game::{OtDeluxeRouteDecision, OtDeluxeRoutePrompt};
    use futures::executor::block_on;
    use yew::prelude::*;
    use yew::{Callback, LocalServerRenderer};

    #[test]
    fn route_prompt_viewmodel_maps_decisions() {
        crate::i18n::set_lang("en");
        let vm = build_route_prompt_viewmodel(OtDeluxeRoutePrompt::SubletteCutoff);
        assert_eq!(vm.primary_decision, OtDeluxeRouteDecision::StayOnTrail);
        assert_eq!(vm.secondary_decision, OtDeluxeRouteDecision::SubletteCutoff);

        let vm = build_route_prompt_viewmodel(OtDeluxeRoutePrompt::DallesFinal);
        assert_eq!(vm.primary_decision, OtDeluxeRouteDecision::RaftColumbia);
        assert_eq!(vm.secondary_decision, OtDeluxeRouteDecision::BarlowRoad);

        let vm = build_route_prompt_viewmodel(OtDeluxeRoutePrompt::DallesShortcut);
        assert_eq!(vm.primary_decision, OtDeluxeRouteDecision::StayOnTrail);
        assert_eq!(vm.secondary_decision, OtDeluxeRouteDecision::DallesShortcut);
    }

    #[test]
    fn route_prompt_card_renders_prompt() {
        crate::i18n::set_lang("en");
        let props = RoutePromptCardProps {
            prompt: OtDeluxeRoutePrompt::SubletteCutoff,
            on_choice: Callback::noop(),
        };
        let html = block_on(LocalServerRenderer::<RoutePromptCard>::with_props(props).render());
        assert!(html.contains("route-prompt-title"));
    }

    #[derive(Properties, Clone, PartialEq)]
    struct ActivateHarnessProps {
        idx: u8,
        start_resolved: bool,
    }

    #[function_component(ActivateHarness)]
    fn activate_harness(props: &ActivateHarnessProps) -> Html {
        use super::interactions::activate_handler;

        let resolved = use_state(|| props.start_resolved);
        let choice = use_mut_ref(|| None);
        let invoked = use_mut_ref(|| false);
        let on_choice = {
            let choice = choice.clone();
            Callback::from(move |decision: OtDeluxeRouteDecision| {
                *choice.borrow_mut() = Some(decision);
            })
        };
        let handler = activate_handler(
            on_choice,
            resolved.clone(),
            OtDeluxeRouteDecision::StayOnTrail,
            OtDeluxeRouteDecision::DallesShortcut,
        );

        if !*invoked.borrow() {
            *invoked.borrow_mut() = true;
            handler.emit(props.idx);
        }

        let choice_label = choice
            .borrow()
            .map(|d| format!("{d:?}"))
            .unwrap_or_else(|| "none".to_string());

        html! {
            <div
                data-resolved={(*resolved).to_string()}
                data-choice={choice_label}
            />
        }
    }

    #[test]
    fn activate_handler_emits_choice_and_resolves() {
        let html = block_on(
            LocalServerRenderer::<ActivateHarness>::with_props(ActivateHarnessProps {
                idx: 2,
                start_resolved: false,
            })
            .render(),
        );
        assert!(html.contains("DallesShortcut"));
    }

    #[test]
    fn activate_handler_ignores_when_resolved_or_invalid() {
        let html = block_on(
            LocalServerRenderer::<ActivateHarness>::with_props(ActivateHarnessProps {
                idx: 9,
                start_resolved: true,
            })
            .render(),
        );
        assert!(html.contains("data-resolved=\"true\""));
        assert!(html.contains("data-choice=\"none\""));
    }
}
