use crate::mechanics::OtDeluxeOccupation;
use crate::mechanics::otdeluxe90s::OtDeluxe90sPolicy;

#[must_use]
pub fn otdeluxe_starting_cash_cents(
    occupation: OtDeluxeOccupation,
    policy: &OtDeluxe90sPolicy,
) -> u32 {
    let dollars = policy
        .occupations
        .iter()
        .find(|spec| spec.occupation == occupation)
        .map_or(0, |spec| spec.starting_cash_dollars);
    u32::from(dollars).saturating_mul(100)
}
