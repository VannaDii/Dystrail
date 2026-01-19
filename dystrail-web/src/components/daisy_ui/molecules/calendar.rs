use crate::components::daisy_ui::foundation as f;

const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: u32) -> u32 {
    if month == 2 && is_leap_year(year) {
        29
    } else {
        let index = usize::try_from(month.saturating_sub(1)).unwrap_or(0);
        DAYS_IN_MONTH.get(index).copied().unwrap_or(30)
    }
}

fn weekday_index(year: i32, month: u32, day: u32) -> u32 {
    let adjusted_month = if month < 3 { month + 12 } else { month };
    let adjusted_year = if month < 3 { year - 1 } else { year };
    let year_of_century = adjusted_year % 100;
    let zero_based_century = adjusted_year / 100;
    let weekday = (i32::try_from(day).unwrap_or(0)
        + (13 * (i32::try_from(adjusted_month).unwrap_or(0) + 1)) / 5
        + year_of_century
        + year_of_century / 4
        + zero_based_century / 4
        + (5 * zero_based_century))
        % 7;
    u32::try_from((weekday + 6) % 7).unwrap_or(0)
}

#[derive(f::Properties, PartialEq, Clone)]
pub struct CalendarProps {
    pub year: i32,
    pub month: u32,
    #[prop_or_default]
    pub selected_day: Option<u32>,
    #[prop_or_default]
    pub on_select: f::Callback<(i32, u32, u32)>,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(Calendar)]
pub fn calendar(props: &CalendarProps) -> f::Html {
    let class = f::class_list(&["calendar", "bg-base-100", "rounded-box"], &props.class);
    let total_days = days_in_month(props.year, props.month);
    let first_weekday = weekday_index(props.year, props.month, 1);
    let weeks = usize::try_from((total_days + first_weekday).div_ceil(7)).unwrap_or(0);
    let selected = props.selected_day;
    let on_select = props.on_select.clone();
    let year = props.year;
    let month = props.month;
    let first_weekday_usize = usize::try_from(first_weekday).unwrap_or(0);
    let total_days_usize = usize::try_from(total_days).unwrap_or(0);
    f::html! {
        <div class={class} role="grid" aria-label="Calendar">
            <div class="grid grid-cols-7 text-center text-sm font-semibold">
                { for ["Su","Mo","Tu","We","Th","Fr","Sa"].iter().map(|day| f::html!{ <div role="columnheader">{ *day }</div> }) }
            </div>
            <div class="grid grid-cols-7 text-center">
                { for (0..weeks).flat_map(|week| {
                    (0..7).map(move |weekday| (week, weekday))
                }).map(|(week, weekday)| {
                    let day_number = week * 7 + weekday;
                    if day_number < first_weekday_usize || day_number >= first_weekday_usize + total_days_usize {
                        f::html! { <button class="btn btn-ghost btn-xs" disabled=true aria-hidden="true"></button> }
                    } else {
                        let day = u32::try_from(day_number - first_weekday_usize + 1).unwrap_or(0);
                        let is_selected = selected == Some(day);
                        let mut cell_classes = f::classes!("btn", "btn-ghost", "btn-xs", "m-1");
                        if is_selected {
                            cell_classes.push("btn-active");
                        }
                        let select_cb = {
                            let on_select = on_select.clone();
                            f::Callback::from(move |_| on_select.emit((year, month, day)))
                        };
                        f::html! {
                            <button class={cell_classes} aria-pressed={is_selected.to_string()} onclick={select_cb}>
                                { day }
                            </button>
                        }
                    }
                })}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{days_in_month, is_leap_year, weekday_index};

    #[test]
    fn leap_year_detection_matches_rules() {
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
    }

    #[test]
    fn days_in_month_handles_leap_and_fallbacks() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2023, 13), 30);
    }

    #[test]
    fn weekday_index_returns_valid_range() {
        let day = weekday_index(2024, 1, 1);
        assert!(day < 7);
    }
}
