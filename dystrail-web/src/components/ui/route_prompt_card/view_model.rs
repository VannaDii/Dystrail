use crate::game::{OtDeluxeRouteDecision, OtDeluxeRoutePrompt};
use crate::i18n;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutePromptViewModel {
    pub title: String,
    pub prompt: String,
    pub primary_label: String,
    pub primary_desc: String,
    pub primary_decision: OtDeluxeRouteDecision,
    pub secondary_label: String,
    pub secondary_desc: String,
    pub secondary_decision: OtDeluxeRouteDecision,
}

#[must_use]
pub fn build_route_prompt_viewmodel(prompt: OtDeluxeRoutePrompt) -> RoutePromptViewModel {
    match prompt {
        OtDeluxeRoutePrompt::SubletteCutoff => RoutePromptViewModel {
            title: i18n::t("route_prompt.sublette.title"),
            prompt: i18n::t("route_prompt.sublette.prompt"),
            primary_label: i18n::t("route_prompt.sublette.options.stay"),
            primary_desc: i18n::t("route_prompt.sublette.desc.stay"),
            primary_decision: OtDeluxeRouteDecision::StayOnTrail,
            secondary_label: i18n::t("route_prompt.sublette.options.take"),
            secondary_desc: i18n::t("route_prompt.sublette.desc.take"),
            secondary_decision: OtDeluxeRouteDecision::SubletteCutoff,
        },
        OtDeluxeRoutePrompt::DallesShortcut => RoutePromptViewModel {
            title: i18n::t("route_prompt.dalles_shortcut.title"),
            prompt: i18n::t("route_prompt.dalles_shortcut.prompt"),
            primary_label: i18n::t("route_prompt.dalles_shortcut.options.stay"),
            primary_desc: i18n::t("route_prompt.dalles_shortcut.desc.stay"),
            primary_decision: OtDeluxeRouteDecision::StayOnTrail,
            secondary_label: i18n::t("route_prompt.dalles_shortcut.options.take"),
            secondary_desc: i18n::t("route_prompt.dalles_shortcut.desc.take"),
            secondary_decision: OtDeluxeRouteDecision::DallesShortcut,
        },
        OtDeluxeRoutePrompt::DallesFinal => RoutePromptViewModel {
            title: i18n::t("route_prompt.dalles_final.title"),
            prompt: i18n::t("route_prompt.dalles_final.prompt"),
            primary_label: i18n::t("route_prompt.dalles_final.options.raft"),
            primary_desc: i18n::t("route_prompt.dalles_final.desc.raft"),
            primary_decision: OtDeluxeRouteDecision::RaftColumbia,
            secondary_label: i18n::t("route_prompt.dalles_final.options.barlow"),
            secondary_desc: i18n::t("route_prompt.dalles_final.desc.barlow"),
            secondary_decision: OtDeluxeRouteDecision::BarlowRoad,
        },
    }
}
