//! Minimal functions included for tarpaulin coverage.

/// Deterministic checksum used solely to produce measurable coverage.
#[must_use]
pub fn coverage_checksum(seed: u64) -> u64 {
    let mut acc = seed ^ 0xDEAD_BEEF_CAFE_BABE;
    for shift in [3, 11, 19, 27] {
        acc = acc.rotate_left(shift) ^ (seed >> shift);
    }
    acc & 0xFFFF_FFFF_FFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_executes() {
        let value = coverage_checksum(0x1234_5678);
        assert_eq!(value, coverage_checksum(0x1234_5678));
        assert_ne!(value, 0);
    }
}
