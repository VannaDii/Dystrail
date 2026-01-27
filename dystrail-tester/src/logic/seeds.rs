use anyhow::{Context, Result, bail};
use dystrail_game::{GameMode, parse_share_code};
use std::collections::HashMap;

/// Detailed seed metadata used for logic and playability analysis.
#[derive(Debug, Clone)]
pub struct SeedInfo {
    pub seed: u64,
    pub code: Option<String>,
    pub source_mode: Option<GameMode>,
}

impl SeedInfo {
    #[must_use]
    pub const fn from_numeric(seed: u64) -> Self {
        Self {
            seed,
            code: None,
            source_mode: None,
        }
    }

    #[must_use]
    pub const fn from_share_code(seed: u64, mode: GameMode, code: String) -> Self {
        Self {
            seed,
            code: Some(code),
            source_mode: Some(mode),
        }
    }

    #[must_use]
    pub fn matches_mode(&self, mode: GameMode) -> bool {
        self.source_mode
            .is_none_or(|source_mode| source_mode == mode)
    }

    #[must_use]
    pub fn label(&self) -> String {
        self.code.clone().unwrap_or_else(|| self.seed.to_string())
    }
}

/// Resolve a list of CLI seed arguments into canonical seed metadata.
///
/// Supports literal integers, share codes, and the special keywords
/// `all` / `available` which expand to every share-code seed.
pub fn resolve_seed_inputs(tokens: &[String]) -> Result<Vec<SeedInfo>> {
    let mut pending: Vec<SeedInfo> = Vec::new();
    let mut request_all = false;

    for token in tokens {
        if token.is_empty() {
            continue;
        }

        if token.eq_ignore_ascii_case("all") || token.eq_ignore_ascii_case("available") {
            request_all = true;
            continue;
        }

        if let Ok(value) = token.parse::<i64>() {
            pending.push(SeedInfo::from_numeric(value.unsigned_abs()));
            continue;
        }

        if let Ok(value) = token.parse::<u64>() {
            pending.push(SeedInfo::from_numeric(value));
            continue;
        }

        if let Some((mode, seed)) = parse_share_code(token) {
            pending.push(SeedInfo::from_share_code(seed, mode, token.to_uppercase()));
            continue;
        }

        bail!("Unrecognized seed token: {token}");
    }

    if request_all {
        pending.extend(generate_all_share_code_seeds()?);
    }

    let mut deduped: Vec<SeedInfo> = Vec::new();
    let mut index: HashMap<(u64, u8), usize> = HashMap::new();

    for info in pending {
        let mode_tag = mode_tag(info.source_mode);
        if index.contains_key(&(info.seed, mode_tag)) {
            continue;
        }

        if mode_tag == 0 {
            if index.contains_key(&(info.seed, 1)) || index.contains_key(&(info.seed, 2)) {
                continue;
            }
            index.insert((info.seed, mode_tag), deduped.len());
            deduped.push(info);
            continue;
        }

        if let Some(existing) = index.remove(&(info.seed, 0)) {
            let seed = info.seed;
            let entry = deduped
                .get_mut(existing)
                .expect("index map points to existing entry");
            *entry = info;
            index.insert((seed, mode_tag), existing);
            continue;
        }

        index.insert((info.seed, mode_tag), deduped.len());
        deduped.push(info);
    }

    if deduped.is_empty() {
        deduped.push(SeedInfo::from_numeric(1337));
    }

    Ok(deduped)
}

fn generate_all_share_code_seeds() -> Result<Vec<SeedInfo>> {
    use dystrail_game::seed::WORD_LIST;

    let mut seeds = Vec::with_capacity(WORD_LIST.len() * 100 * 2);

    for word in WORD_LIST {
        for suffix in 0..100 {
            let classic_code = format!("CL-{word}{suffix:02}");
            let (mode, seed) = parse_share_code_checked(&classic_code)?;
            seeds.push(SeedInfo::from_share_code(seed, mode, classic_code));

            let deep_code = format!("DP-{word}{suffix:02}");
            let (mode, seed) = parse_share_code_checked(&deep_code)?;
            seeds.push(SeedInfo::from_share_code(seed, mode, deep_code));
        }
    }

    Ok(seeds)
}

fn parse_share_code_checked(code: &str) -> Result<(GameMode, u64)> {
    parse_share_code(code).with_context(|| format!("failed to parse share code: {code}"))
}

const fn mode_tag(mode: Option<GameMode>) -> u8 {
    match mode {
        Some(GameMode::Classic) => 1,
        Some(GameMode::Deep) => 2,
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dystrail_game::GameMode;

    #[test]
    fn resolves_numeric_and_share_code() {
        let raw = vec![
            "42".to_string(),
            "-7".to_string(),
            "CL-ORANGE42".to_string(),
        ];
        let seeds = resolve_seed_inputs(&raw).unwrap();
        assert!(seeds.iter().any(|s| s.seed == 42 && s.code.is_none()));
        assert!(seeds.iter().any(|s| s.seed == 7 && s.code.is_none()));
        assert!(seeds.iter().any(|s| {
            s.code.as_deref() == Some("CL-ORANGE42") && s.source_mode == Some(GameMode::Classic)
        }));
    }

    #[test]
    fn expands_all_share_codes() {
        let seeds = resolve_seed_inputs(&["all".to_string()]).unwrap();
        let expected = dystrail_game::seed::WORD_LIST.len() * 100 * 2;
        assert_eq!(seeds.len(), expected);
        assert!(seeds.iter().all(|s| s.code.is_some()));
    }

    #[test]
    fn seed_info_matches_mode_accepts_unspecified() {
        let info = SeedInfo::from_numeric(42);
        assert!(info.matches_mode(GameMode::Classic));
    }

    #[test]
    fn resolve_seed_inputs_rejects_unrecognized_token() {
        let raw = vec!["not-a-seed".to_string()];
        let err = resolve_seed_inputs(&raw).expect_err("invalid token should fail");
        assert!(err.to_string().contains("Unrecognized seed token"));
    }

    #[test]
    fn resolve_seed_inputs_defaults_when_empty() {
        let seeds = resolve_seed_inputs(&[]).expect("default seed");
        assert!(seeds.iter().any(|s| s.seed == 1337));
    }

    #[test]
    fn resolve_seed_inputs_dedupes_duplicate_tokens() {
        let seeds = resolve_seed_inputs(&["42".to_string(), "42".to_string()]).unwrap();
        assert_eq!(seeds.iter().filter(|info| info.seed == 42).count(), 1);
    }

    #[test]
    fn resolve_seed_inputs_prefers_code_for_duplicate_seed() {
        let code = dystrail_game::seed::encode_friendly(false, 42);
        let seed = dystrail_game::parse_share_code(&code)
            .map(|(_, seed)| seed)
            .expect("seed should decode");
        let tokens = vec![seed.to_string(), code.clone()];
        let seeds = resolve_seed_inputs(&tokens).unwrap();
        assert_eq!(
            seeds.iter().filter(|info| info.seed == seed).count(),
            1,
            "seeds: {seeds:?}"
        );
        let entry = seeds
            .iter()
            .find(|info| info.seed == seed)
            .expect("seed entry");
        assert_eq!(entry.code.as_deref(), Some(code.as_str()));
    }

    #[test]
    fn resolve_seed_inputs_accepts_large_u64() {
        let expected = i64::MAX as u64 + 1;
        let seeds = resolve_seed_inputs(&[expected.to_string()]).expect("seed should parse");
        assert!(seeds.iter().any(|info| info.seed == expected));
    }
}
