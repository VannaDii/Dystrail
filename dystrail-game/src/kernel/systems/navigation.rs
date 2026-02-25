use rand::Rng;

use crate::journey::{EventDecisionTrace, RollValue, WeightedCandidate};
use crate::mechanics::otdeluxe90s::{OtDeluxeNavigationDelay, OtDeluxeNavigationPolicy};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtDeluxeNavigationEvent {
    LostTrail,
    WrongTrail,
    Impassable,
    Snowbound,
}

#[must_use]
pub const fn otdeluxe_navigation_reason_tag(event: OtDeluxeNavigationEvent) -> &'static str {
    match event {
        OtDeluxeNavigationEvent::LostTrail => "otdeluxe.nav_lost",
        OtDeluxeNavigationEvent::WrongTrail => "otdeluxe.nav_wrong",
        OtDeluxeNavigationEvent::Impassable => "otdeluxe.nav_impassable",
        OtDeluxeNavigationEvent::Snowbound => "otdeluxe.nav_snowbound",
    }
}

#[must_use]
pub const fn otdeluxe_navigation_delay_tag(blocked: bool) -> &'static str {
    if blocked {
        "otdeluxe.nav_blocked"
    } else {
        "otdeluxe.nav_delay"
    }
}

#[must_use]
pub const fn otdeluxe_navigation_event_id(event: OtDeluxeNavigationEvent) -> &'static str {
    match event {
        OtDeluxeNavigationEvent::LostTrail => "lost_trail",
        OtDeluxeNavigationEvent::WrongTrail => "wrong_trail",
        OtDeluxeNavigationEvent::Impassable => "impassable",
        OtDeluxeNavigationEvent::Snowbound => "snowbound",
    }
}

#[must_use]
pub const fn otdeluxe_navigation_is_blocked(event: OtDeluxeNavigationEvent) -> bool {
    matches!(
        event,
        OtDeluxeNavigationEvent::Impassable | OtDeluxeNavigationEvent::Snowbound
    )
}

#[must_use]
pub const fn otdeluxe_navigation_delay_for(
    event: OtDeluxeNavigationEvent,
    policy: &OtDeluxeNavigationPolicy,
) -> OtDeluxeNavigationDelay {
    match event {
        OtDeluxeNavigationEvent::LostTrail => policy.lost_delay,
        OtDeluxeNavigationEvent::WrongTrail => policy.wrong_delay,
        OtDeluxeNavigationEvent::Impassable => policy.impassable_delay,
        OtDeluxeNavigationEvent::Snowbound => policy.snowbound_delay,
    }
}

#[must_use]
pub fn roll_otdeluxe_navigation_delay_days<R: Rng>(
    delay: OtDeluxeNavigationDelay,
    rng: &mut R,
) -> u8 {
    if delay.max_days == 0 {
        return 0;
    }
    let min_days = delay.min_days.min(delay.max_days);
    let max_days = delay.max_days.max(delay.min_days);
    rng.gen_range(min_days..=max_days)
}

#[must_use]
pub fn roll_otdeluxe_navigation_event_with_trace<R: Rng>(
    policy: &OtDeluxeNavigationPolicy,
    snow_depth: f32,
    rng: &mut R,
) -> (Option<OtDeluxeNavigationEvent>, Option<EventDecisionTrace>) {
    let chance = policy.chance_per_day.clamp(0.0, 1.0);
    if chance <= 0.0 {
        return (None, None);
    }
    if rng.r#gen::<f32>() >= chance {
        return (None, None);
    }

    let snow_weight = if snow_depth >= policy.snowbound_min_depth_in {
        policy.snowbound_weight
    } else {
        0
    };
    let lost_weight = u32::from(policy.lost_weight);
    let wrong_weight = u32::from(policy.wrong_weight);
    let impassable_weight = u32::from(policy.impassable_weight);
    let lost = (OtDeluxeNavigationEvent::LostTrail, lost_weight);
    let wrong = (OtDeluxeNavigationEvent::WrongTrail, wrong_weight);
    let impassable = (OtDeluxeNavigationEvent::Impassable, impassable_weight);
    let snowbound = (OtDeluxeNavigationEvent::Snowbound, u32::from(snow_weight));
    let options = [lost, wrong, impassable, snowbound];
    let total_weight: u32 = options.iter().map(|(_, weight)| *weight).sum();
    if total_weight == 0 {
        return (None, None);
    }

    let roll = rng.gen_range(0..total_weight);
    let selected_event = pick_navigation_event(&options, roll);

    let total_weight_f64 = f64::from(total_weight);
    let mut candidates = Vec::new();
    for (event, weight) in &options {
        candidates.push(WeightedCandidate {
            id: otdeluxe_navigation_event_id(*event).to_string(),
            base_weight: f64::from(*weight),
            multipliers: Vec::new(),
            final_weight: f64::from(*weight) / total_weight_f64,
        });
    }

    let trace = Some(EventDecisionTrace {
        pool_id: String::from("otdeluxe.navigation"),
        roll: RollValue::U32(roll),
        candidates,
        chosen_id: otdeluxe_navigation_event_id(selected_event).to_string(),
    });
    (Some(selected_event), trace)
}

fn pick_navigation_event(
    options: &[(OtDeluxeNavigationEvent, u32); 4],
    roll: u32,
) -> OtDeluxeNavigationEvent {
    let mut remaining = roll;
    for (event, weight) in options {
        if *weight > 0 {
            if remaining < *weight {
                return *event;
            }
            remaining = remaining.saturating_sub(*weight);
        }
    }
    OtDeluxeNavigationEvent::LostTrail
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn navigation_roll_returns_none_when_chance_is_zero() {
        let policy = OtDeluxeNavigationPolicy {
            chance_per_day: 0.0,
            ..OtDeluxeNavigationPolicy::default()
        };
        let mut rng = SmallRng::seed_from_u64(1);
        let (event, trace) = roll_otdeluxe_navigation_event_with_trace(&policy, 0.0, &mut rng);
        assert!(event.is_none());
        assert!(trace.is_none());
    }

    #[test]
    fn navigation_roll_skips_zero_weight_events_and_records_trace() {
        let policy = OtDeluxeNavigationPolicy {
            chance_per_day: 1.0,
            lost_weight: 0,
            wrong_weight: 1,
            impassable_weight: 0,
            snowbound_weight: 0,
            snowbound_min_depth_in: 100.0,
            ..OtDeluxeNavigationPolicy::default()
        };
        let mut rng = SmallRng::seed_from_u64(9);
        let (event, trace) = roll_otdeluxe_navigation_event_with_trace(&policy, 0.0, &mut rng);
        assert_eq!(event, Some(OtDeluxeNavigationEvent::WrongTrail));
        let trace = trace.expect("trace should be emitted when an event is selected");
        assert_eq!(trace.chosen_id, "wrong_trail");
    }

    #[test]
    fn navigation_bucket_selection_maps_roll_to_event() {
        let options = [
            (OtDeluxeNavigationEvent::LostTrail, 2),
            (OtDeluxeNavigationEvent::WrongTrail, 3),
            (OtDeluxeNavigationEvent::Impassable, 0),
            (OtDeluxeNavigationEvent::Snowbound, 1),
        ];
        assert_eq!(
            pick_navigation_event(&options, 0),
            OtDeluxeNavigationEvent::LostTrail
        );
        assert_eq!(
            pick_navigation_event(&options, 2),
            OtDeluxeNavigationEvent::WrongTrail
        );
        assert_eq!(
            pick_navigation_event(&options, 5),
            OtDeluxeNavigationEvent::Snowbound
        );
    }

    #[test]
    fn navigation_bucket_selection_falls_back_when_all_weights_zero() {
        let options = [
            (OtDeluxeNavigationEvent::LostTrail, 0),
            (OtDeluxeNavigationEvent::WrongTrail, 0),
            (OtDeluxeNavigationEvent::Impassable, 0),
            (OtDeluxeNavigationEvent::Snowbound, 0),
        ];
        assert_eq!(
            pick_navigation_event(&options, 0),
            OtDeluxeNavigationEvent::LostTrail
        );
    }
}
