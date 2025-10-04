use crate::game::{GameState, PacingConfig};
use crate::i18n;
use crate::input::numeric_key_to_index;
use std::collections::HashMap;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties)]
pub struct PaceDietPanelProps {
    pub game_state: Rc<GameState>,
    pub pacing_config: Rc<PacingConfig>,
    pub on_pace_change: Callback<String>,
    pub on_diet_change: Callback<String>,
    pub on_back: Callback<()>,
}

impl PartialEq for PaceDietPanelProps {
    fn eq(&self, other: &Self) -> bool {
        // Compare the relevant fields for re-rendering decisions
        self.game_state.pace == other.game_state.pace
            && self.game_state.diet == other.game_state.diet
    }
}

#[function_component(PaceDietPanel)]
pub fn pace_diet_panel(props: &PaceDietPanelProps) -> Html {
    let focused_index = use_state(|| 1u8);
    let status_message = use_state(String::new);

    let on_activate = {
        let pacing_config = props.pacing_config.clone();
        let on_pace_change = props.on_pace_change.clone();
        let on_diet_change = props.on_diet_change.clone();
        let on_back = props.on_back.clone();
        let status_message = status_message.clone();

        Callback::from(move |idx: u8| match idx {
            1 => {
                let pace_id = "steady";
                let pace_cfg = pacing_config.get_pace_safe(pace_id);
                let sanity_str = format!("{:+}", pace_cfg.sanity);
                let pants_str = format!("{:+}", pace_cfg.pants);
                let chance_str = format!("{:+.0}%", pace_cfg.encounter_chance_delta * 100.0);
                let mut args = HashMap::new();
                args.insert("pace", pace_cfg.name.as_str());
                args.insert("sanity", sanity_str.as_str());
                args.insert("pants", pants_str.as_str());
                args.insert("chance", chance_str.as_str());
                let msg = i18n::tr("pacediet.announce.pace_set", Some(&args));
                status_message.set(msg);
                on_pace_change.emit(pace_id.to_string());
            }
            2 => {
                let pace_id = "heated";
                let pace_cfg = pacing_config.get_pace_safe(pace_id);
                let sanity_str = format!("{:+}", pace_cfg.sanity);
                let pants_str = format!("{:+}", pace_cfg.pants);
                let chance_str = format!("{:+.0}%", pace_cfg.encounter_chance_delta * 100.0);
                let mut args = HashMap::new();
                args.insert("pace", pace_cfg.name.as_str());
                args.insert("sanity", sanity_str.as_str());
                args.insert("pants", pants_str.as_str());
                args.insert("chance", chance_str.as_str());
                let msg = i18n::tr("pacediet.announce.pace_set", Some(&args));
                status_message.set(msg);
                on_pace_change.emit(pace_id.to_string());
            }
            3 => {
                let pace_id = "blitz";
                let pace_cfg = pacing_config.get_pace_safe(pace_id);
                let sanity_str = format!("{:+}", pace_cfg.sanity);
                let pants_str = format!("{:+}", pace_cfg.pants);
                let chance_str = format!("{:+.0}%", pace_cfg.encounter_chance_delta * 100.0);
                let mut args = HashMap::new();
                args.insert("pace", pace_cfg.name.as_str());
                args.insert("sanity", sanity_str.as_str());
                args.insert("pants", pants_str.as_str());
                args.insert("chance", chance_str.as_str());
                let msg = i18n::tr("pacediet.announce.pace_set", Some(&args));
                status_message.set(msg);
                on_pace_change.emit(pace_id.to_string());
            }
            4 => {
                let diet_id = "quiet";
                let diet_cfg = pacing_config.get_diet_safe(diet_id);
                let sanity_str = format!("{:+}", diet_cfg.sanity);
                let pants_str = format!("{:+}", diet_cfg.pants);
                let receipt_str = format!("{:+}%", diet_cfg.receipt_find_pct_delta);
                let mut args = HashMap::new();
                args.insert("diet", diet_cfg.name.as_str());
                args.insert("sanity", sanity_str.as_str());
                args.insert("pants", pants_str.as_str());
                args.insert("receipt", receipt_str.as_str());
                let msg = i18n::tr("pacediet.announce.diet_set", Some(&args));
                status_message.set(msg);
                on_diet_change.emit(diet_id.to_string());
            }
            5 => {
                let diet_id = "mixed";
                let diet_cfg = pacing_config.get_diet_safe(diet_id);
                let sanity_str = format!("{:+}", diet_cfg.sanity);
                let pants_str = format!("{:+}", diet_cfg.pants);
                let receipt_str = format!("{:+}%", diet_cfg.receipt_find_pct_delta);
                let mut args = HashMap::new();
                args.insert("diet", diet_cfg.name.as_str());
                args.insert("sanity", sanity_str.as_str());
                args.insert("pants", pants_str.as_str());
                args.insert("receipt", receipt_str.as_str());
                let msg = i18n::tr("pacediet.announce.diet_set", Some(&args));
                status_message.set(msg);
                on_diet_change.emit(diet_id.to_string());
            }
            6 => {
                let diet_id = "doom";
                let diet_cfg = pacing_config.get_diet_safe(diet_id);
                let sanity_str = format!("{:+}", diet_cfg.sanity);
                let pants_str = format!("{:+}", diet_cfg.pants);
                let receipt_str = format!("{:+}%", diet_cfg.receipt_find_pct_delta);
                let mut args = HashMap::new();
                args.insert("diet", diet_cfg.name.as_str());
                args.insert("sanity", sanity_str.as_str());
                args.insert("pants", pants_str.as_str());
                args.insert("receipt", receipt_str.as_str());
                let msg = i18n::tr("pacediet.announce.diet_set", Some(&args));
                status_message.set(msg);
                on_diet_change.emit(diet_id.to_string());
            }
            0 => {
                status_message.set(String::new());
                on_back.emit(());
            }
            _ => {}
        })
    };

    let on_keydown = {
        let on_activate = on_activate.clone();
        let focused_index = focused_index.clone();

        Callback::from(move |e: KeyboardEvent| match e.key().as_str() {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                if let Some(n) = numeric_key_to_index(e.key().as_str()) {
                    on_activate.emit(n);
                    e.prevent_default();
                }
            }
            "ArrowDown" => {
                let current = *focused_index;
                let next = if current >= 6 { 0 } else { current + 1 };
                focused_index.set(next);
                e.prevent_default();
            }
            "ArrowUp" => {
                let current = *focused_index;
                let next = if current == 0 { 6 } else { current - 1 };
                focused_index.set(next);
                e.prevent_default();
            }
            "Enter" | " " => {
                on_activate.emit(*focused_index);
                e.prevent_default();
            }
            "Escape" => {
                on_activate.emit(0);
                e.prevent_default();
            }
            _ => {}
        })
    };

    let on_click = {
        let on_activate = on_activate.clone();
        Callback::from(move |idx: u8| {
            on_activate.emit(idx);
        })
    };

    // Helper function to render a menu line with proper ARIA attributes
    let render_menu_line = |idx: u8, text: String, is_selected: bool, tooltip: String| {
        let is_focused = *focused_index == idx;
        let onclick = {
            let on_click = on_click.clone();
            Callback::from(move |_: MouseEvent| {
                on_click.emit(idx);
            })
        };

        let onfocus = {
            let focused_index = focused_index.clone();
            Callback::from(move |_: FocusEvent| {
                focused_index.set(idx);
            })
        };

        let class = format!(
            "pace-diet-line {}{}",
            if is_focused { "focused " } else { "" },
            if is_selected { "selected" } else { "" }
        );

        html! {
            <li
                class={class}
                role="menuitem"
                tabindex={if is_focused { "0" } else { "-1" }}
                aria-describedby={format!("tooltip-{idx}")}
                {onclick}
                {onfocus}
            >
                <span class="line-number">{format!("{idx})")}</span>
                <span class="line-text">{text}</span>
                <div id={format!("tooltip-{idx}")} class="sr-only">{tooltip}</div>
            </li>
        }
    };

    // Get current selections for bracket display
    let current_pace = &props.game_state.pace;
    let current_diet = &props.game_state.diet;

    // Render menu lines with brackets for current selections
    let pace_steady_text = if current_pace == "steady" {
        i18n::t("pacediet.menu.pace_steady")
    } else {
        "Pace:  Steady".to_string()
    };

    let pace_heated_text = if current_pace == "heated" {
        "Pace: [Heated]".to_string()
    } else {
        i18n::t("pacediet.menu.pace_heated")
    };

    let pace_blitz_text = if current_pace == "blitz" {
        "Pace: [Blitz]".to_string()
    } else {
        "Pace:  Blitz".to_string()
    };

    let diet_quiet_text = if current_diet == "quiet" {
        "Diet: [Quiet]".to_string()
    } else {
        i18n::t("pacediet.menu.diet_quiet")
    };

    let diet_mixed_text = if current_diet == "mixed" {
        i18n::t("pacediet.menu.diet_mixed")
    } else {
        "Diet:  Mixed".to_string()
    };

    let diet_doom_text = if current_diet == "doom" {
        "Diet: [Doomscroll]".to_string()
    } else {
        "Diet:  Doomscroll".to_string()
    };

    html! {
        <section
            role="region"
            aria-labelledby="pd-title"
            onkeydown={on_keydown}
            class="pace-diet-panel"
        >
            <h3 id="pd-title" class="pace-diet-title">
                {i18n::t("pacediet.title")}
            </h3>

            <ul
                role="menu"
                aria-label={i18n::t("pacediet.title")}
                class="pace-diet-menu"
            >
                {render_menu_line(1, pace_steady_text, current_pace == "steady", i18n::t("pacediet.tooltips.steady"))}
                {render_menu_line(2, pace_heated_text, current_pace == "heated", i18n::t("pacediet.tooltips.heated"))}
                {render_menu_line(3, pace_blitz_text, current_pace == "blitz", i18n::t("pacediet.tooltips.blitz"))}
                {render_menu_line(4, diet_quiet_text, current_diet == "quiet", i18n::t("pacediet.tooltips.quiet"))}
                {render_menu_line(5, diet_mixed_text, current_diet == "mixed", i18n::t("pacediet.tooltips.mixed"))}
                {render_menu_line(6, diet_doom_text, current_diet == "doom", i18n::t("pacediet.tooltips.doom"))}
                {render_menu_line(0, i18n::t("pacediet.menu.back"), false, "Return to previous menu".to_string())}
            </ul>

            <div
                id="pd-status"
                aria-live="polite"
                class="pace-diet-status"
                role="status"
            >
                {(*status_message).clone()}
            </div>
        </section>
    }
}
