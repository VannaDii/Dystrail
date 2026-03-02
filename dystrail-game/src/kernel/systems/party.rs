use crate::otdeluxe_state::OtDeluxePartyMember;

#[must_use]
pub fn alive_member_indices(members: &[OtDeluxePartyMember]) -> Vec<usize> {
    let mut indices = Vec::new();
    for (idx, member) in members.iter().enumerate() {
        if member.alive {
            indices.push(idx);
        }
    }
    indices
}
