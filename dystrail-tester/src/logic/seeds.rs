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
    pub fn from_numeric(seed: u64) -> Self {
        Self {
            seed,
            code: None,
            source_mode: None,
        }
    }

    #[must_use]
    pub fn from_share_code(seed: u64, mode: GameMode, code: String) -> Self {
        Self {
            seed,
            code: Some(code),
            source_mode: Some(mode),
        }
    }

    #[must_use]
    pub fn matches_mode(&self, mode: GameMode) -> bool {
        match self.source_mode {
            Some(source_mode) => source_mode == mode,
            None => true,
        }
    }

    #[must_use]
    pub fn share_code_for_mode(&self, mode: GameMode) -> String {
        if let (Some(code), Some(source_mode)) = (&self.code, self.source_mode)
            && source_mode == mode
        {
            return code.clone();
        }

        dystrail_game::encode_friendly(matches!(mode, GameMode::Deep), self.seed)
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
        if let Some(existing) = index.get(&(info.seed, mode_tag)) {
            let entry = deduped
                .get_mut(*existing)
                .expect("index map points to existing entry");
            if entry.code.is_none() && info.code.is_some() {
                *entry = info;
            }
        } else {
            index.insert((info.seed, mode_tag), deduped.len());
            deduped.push(info);
        }
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

fn mode_tag(mode: Option<GameMode>) -> u8 {
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
}
