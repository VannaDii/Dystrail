//! Simple, consistent pictograms for the journey controls and status strip.
use yew::prelude::*;

pub fn render(kind: &str) -> Html {
    let drawing = match kind {
        "save" => {
            html! {<><path d="M5 3h12l4 4v14H3V3h2ZM7 3v6h10V3M7 21v-8h10v8"/><path d="M14 5v2"/></>}
        }
        "load" => {
            html! {<><path d="M3 18V6a2 2 0 0 1 2-2h4l2 3h8a2 2 0 0 1 2 2v2"/><path d="M3 18a2 2 0 0 0 2 2h13a2 2 0 0 0 1.9-1.4L22 11H8l-3 7H3Z"/></>}
        }
        "download" => html! {<><path d="M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5"/></>},
        "offline-ready" => {
            html! {<><rect x="3" y="3" width="18" height="13" rx="2"/><path d="M8 21h8m-4-5v5M8 9l3 3 5-5"/></>}
        }
        "status-warning" => {
            html! {<><path d="M12 3 2 21h20L12 3ZM12 9v5m0 3v.5"/></>}
        }
        "sync" => {
            html! {<><path d="M20 8a8 8 0 0 0-13-2L4 9m0-6v6h6M4 16a8 8 0 0 0 13 2l3-3m0 6v-6h-6"/></>}
        }
        "upload" => html! {<><path d="M12 16V4m-5 5 5-5 5 5M4 16v5h16v-5"/></>},
        "copy" => {
            html! {<><rect x="8" y="8" width="13" height="13" rx="2"/><path d="M16 5V3H3v13h2"/></>}
        }
        "close" => html! {<path d="m6 6 12 12M6 18 18 6"/>},
        "camp" => html! {<><path d="M3 20 12 4l9 16H3ZM8 20l4-8 4 8M10 3l4 4" /></>},
        "route" => {
            html! {<><path d="m3 5 6-2 6 2 6-2v16l-6 2-6-2-6 2V5ZM9 3v16M15 5v16" /><path d="m5 13 4-3 6 3 4-4" stroke-dasharray="2 2" /></>}
        }
        "return" => html! {<path d="m9 4-6 6 6 6M3 10h11a6 6 0 0 1 0 12" />},
        "location" => {
            html! {<><path d="M19 10c0 5-7 11-7 11S5 15 5 10a7 7 0 0 1 14 0Z"/><circle cx="12" cy="10" r="2.5"/></>}
        }
        "travel" | "vehicle" => {
            html! {<><path d="M3 7h12l5 5v6h-3M7 18h6M3 18V7M13 7v6h7M4 13h5" /><circle cx="5" cy="18" r="2"/><circle cx="15" cy="18" r="2"/></>}
        }
        "repair" => {
            html! {<><path d="M14 3a6 6 0 0 0-7 7L3 16a3 3 0 0 0 5 4l6-6a6 6 0 0 0 7-7l-4 4-4-4 4-4Z" /></>}
        }
        "care" => {
            html! {<><path d="M12 21 3 12a5 5 0 0 1 9-6 5 5 0 0 1 9 6L12 21Z" /><path d="M8 12h8M12 8v8" /></>}
        }
        "town" => html! {<><path d="M3 10h18L19 4H5l-2 6ZM5 10v11h14V10M9 21v-7h6v7" /></>},
        "encounter" => html! {<><path d="M4 4h16v12H10l-6 4V4Z" /><path d="M8 8h8M8 12h5" /></>},
        "cash" => {
            html! {<><rect x="2" y="5" width="20" height="14" rx="2" /><circle cx="12" cy="12" r="3" /><path d="M5 9v6M19 9v6" /></>}
        }
        "clock" => html! {<><circle cx="12" cy="12" r="9"/><path d="M12 6v6l4 2"/></>},
        "steady" => html! {<><path d="M5 20 8 4h8l3 16M12 5v3m0 4v3m0 4v1"/></>},
        "heated" => html! {<><path d="m4 7 6 5-6 5m9-10 6 5-6 5"/></>},
        "blitz" => html! {<path d="M14 2 4 14h7l-1 8L21 9h-8l1-7Z"/>},
        "quiet" => {
            html! {<><path d="M8 10a4 4 0 0 1 8 0c0 5 3 5 3 7H5c0-2 3-2 3-7M10 21h4M3 3l18 18"/></>}
        }
        "mixed" => {
            html! {<><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M7 8h5v5H7zM15 8h3m-3 4h3M7 16h11"/></>}
        }
        "doom" => {
            html! {<><rect x="6" y="2" width="12" height="20" rx="2"/><path d="M10 5h4m-4 14h4M10 10l2 3 2-3"/></>}
        }
        _ => Html::default(),
    };
    html! {<svg class="journey-icon" data-icon={kind.to_owned()} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{drawing}</svg>}
}

#[must_use]
pub const fn action_kind(scene: &super::journey_scene::SceneStage) -> &'static str {
    use super::journey_scene::SceneStage;
    match scene {
        SceneStage::Breakdown => "repair",
        SceneStage::Camp => "camp",
        SceneStage::Town => "town",
        SceneStage::Care | SceneStage::CareIncident { .. } => "care",
        SceneStage::Encounter(_) | SceneStage::EncounterOutcome { .. } | SceneStage::Boss => "encounter",
        SceneStage::Ending(_) | SceneStage::Setup | SceneStage::Travel(_) => "travel",
    }
}
