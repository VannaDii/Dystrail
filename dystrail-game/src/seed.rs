//! Reversible share-code scheme with 512-word list.
//! Code format: <MODE>-<WORD><NN>, e.g., CL-ORANGE42, DP-GATOR97

use crate::state::GameMode;

fn fnv1a64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0100_0000_01b3;
    let mut hash = FNV_OFFSET;
    for b in bytes {
        hash = (hash ^ u64::from(*b)).wrapping_mul(FNV_PRIME);
    }
    hash
}

fn sanitize_word(word: &str) -> String {
    word.chars()
        .filter(char::is_ascii_alphabetic)
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

// Word list for share codes
pub const WORD_LIST: [&str; 256] = [
    "ABBOTT", "ABSCAM", "AILEEN", "AILES", "ALEXJON", "ALITO", "AMNESTY", "AMYBAR", "ANTIFA",
    "ANTIVAX", "AOC", "ASYLUM", "AUDITAZ", "AZAR", "BANHAM", "BANNON", "BARR", "BARRON", "BECK",
    "BEDMIN", "BENGHAZ", "BERNIE", "BIDEN", "BIGLY", "BIGMAC", "BLEACH", "BLEXIT", "BLM",
    "BOEBERT", "BOLTON", "BONTA", "BORDER", "BOXES", "BREIT", "BUNKER", "BUTLER", "CANNON",
    "CAPRIOT", "CARAVAN", "CARSON", "CHEETO", "CHENEY", "CHESBRO", "CHINAV", "CLIMATE", "CLOWN",
    "COAL", "COATS", "COHEN", "COLLUSN", "CONWAY", "COTTON", "COVAX", "COVFEFE", "COVID", "CRISIS",
    "CRUZ", "CYBRNJA", "DACA", "DECLAS", "DEFUND", "DELTA", "DESANT", "DESOZA", "DEVOS", "DOCS",
    "DOMINON", "DONALD", "DOTARD", "DREAMER", "EASTMAN", "EMAILS", "EMOLUMT", "ENERGY", "FAKENEW",
    "FAUCI", "FEST", "FETTER", "FIRENFY", "FLYNN", "FOXNEWS", "FRAUD", "GAB", "GAETZ", "GASLIT",
    "GETTR", "GIULIA", "GORKA", "GORSUCH", "GRAHAM", "GREENE", "GRIFTER", "GUILFOY", "GULFOFA",
    "GUTFELD", "GUVMENT", "HAMBERD", "HANNITY", "HAWLEY", "HERSHEL", "HICKS", "HOAX", "HOAXER",
    "HOCKER", "HUNTER", "HYDROXY", "ICE", "IMPEACH", "INFRAWK", "INGRAHM", "INSURCT", "IVANA",
    "IVANKA", "JANUARY", "JERKS", "JORDAN", "KAG", "KAVANAH", "KELLY", "KELLYAN", "KRAKEN",
    "KRISTI", "KUNGFLU", "KUSHJR", "KUSHNR", "LARA", "LAWLESS", "LEVIN", "LIMBAUG", "LINWOOD",
    "LIZ", "LOCKDOW", "LOCKDWN", "LOCKHER", "LOSER", "LOSERS", "LOYALTY", "MAGA", "MAGARLY",
    "MANAFRT", "MANGO", "MARALAG", "MASKERS", "MASKOFF", "MCCAIN", "MCENANY", "MILLER", "MILLS",
    "MNUCHIN", "MOB", "MSM", "MTG", "MULLER", "MULVAN", "MURDOCH", "MURPHY", "MUSLBAN", "NADLER",
    "NAVARRO", "NEWSMAX", "NEWSOM", "NOEM", "NOOBSTR", "NUKHURR", "NUNES", "OANN", "OCASIO", "OIL",
    "OMAR", "OPIODS", "ORANGE", "OREILLY", "OSSOFF", "OWENS", "PARADE", "PARDON", "PARLER",
    "PAYOFF", "PELOSI", "PELTS", "PENCE", "PLANDEM", "POMPEO", "POWELL", "PRICE", "PRIEBUS",
    "PRUITT", "QANON", "QSHAMAN", "RAIMDO", "RALLY", "RAND", "RECOUNT", "RETURNS", "RETWEET",
    "RICK", "RICO", "RIGGED", "RIOT", "ROBERTS", "ROMNEY", "RON", "ROSEN", "ROSS", "RUDY",
    "RUMBLE", "RUSSIA", "RYAN", "SAD", "SARAH", "SAVIOR", "SCALISE", "SCHUMER", "SCOTT", "SESSNS",
    "SHAM", "SHARPIE", "SIDNEY", "SPACEF", "SPACEX", "SPICER", "SPYGATE", "STEVE", "STONE",
    "STOPCNT", "SUCKERS", "SUNUNU", "SYMPOS", "TAPPER", "TAXES", "THOMAS", "TIFFANY", "TIKTOK",
    "TILLER", "TLAIB", "TRADEWR", "TRAVELB", "TRUMP", "TRUTHSO", "TWEETS", "UKRCALL", "ULTRAMG",
    "UNMASK", "VACCINE", "VOTEFRA", "WALKER", "WALL", "WARNOCK", "WARPSPD", "WHITAKR", "WIN",
    "WITCH", "YOUREFI", "YUGE", "ZEROTOL", "ZINKE",
];

#[inline]
fn pack(word_index: u16, nn: u8) -> u16 {
    word_index & 0x01FF | ((u16::from(nn) & 0x7F) << 9)
}

#[inline]
fn unpack(packed: u16) -> (u16, u8) {
    (packed & 0x01FF, ((packed >> 9) & 0x7F) as u8)
}

fn compose_seed(is_deep: bool, word_index: u16, nn: u8) -> u64 {
    let packed = pack(word_index, nn);
    // Domain-separated FNV input
    let mut buf = [0u8; 10];
    buf[..6].copy_from_slice(b"DYSTR-");
    buf[6] = if is_deep { b'D' } else { b'C' };
    buf[7] = (packed & 0xFF) as u8;
    buf[8] = (packed >> 8) as u8;
    buf[9] = 0xA5;
    let h = fnv1a64(&buf);
    (h & 0xFFFF_FFFF_FFFF_0000) | u64::from(packed)
}

#[must_use]
pub fn encode_friendly(is_deep: bool, seed: u64) -> String {
    let mode = if is_deep { "DP" } else { "CL" };
    let packed = (seed & 0xFFFF) as u16;
    let (wi, mut nn) = unpack(packed);
    let word = WORD_LIST.get(wi as usize).copied().unwrap_or("ORANGE");
    if nn > 99 {
        nn %= 100;
    }
    format!("{mode}-{word}{nn:02}")
}

#[must_use]
pub fn decode_to_seed(code: &str) -> Option<(bool, u64)> {
    let s = code.trim();
    let (m, rest) = s.split_once('-')?;
    let is_deep = matches!(m.to_ascii_uppercase().as_str(), "DP");
    if rest.len() < 3 {
        return None;
    }
    let (word_part, nn_part) = rest.split_at(rest.len() - 2);
    let nn: u8 = nn_part.parse().ok()?;
    let word = sanitize_word(word_part);
    let idx = WORD_LIST.iter().position(|w| sanitize_word(w) == word)?;
    let wi = u16::try_from(idx).ok()?;
    let seed = compose_seed(is_deep, wi, nn);
    Some((is_deep, seed))
}

#[must_use]
pub fn generate_code_from_entropy(is_deep: bool, entropy: u64) -> String {
    let wi = u16::try_from(entropy % WORD_LIST.len() as u64).unwrap_or(0);
    let nn = ((entropy >> 17) % 100) as u8;
    let seed = compose_seed(is_deep, wi, nn);
    encode_friendly(is_deep, seed)
}

/// Parse a share code into `GameMode` and seed
#[must_use]
pub fn parse_share_code(code: &str) -> Option<(GameMode, u64)> {
    decode_to_seed(code).map(|(is_deep, seed)| {
        let mode = if is_deep {
            GameMode::Deep
        } else {
            GameMode::Classic
        };
        (mode, seed)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrips_code() {
        let seed = 0xDEAD_BEEF_CAFE_BABE;
        let code = encode_friendly(true, seed);
        let (deep, new_seed) = decode_to_seed(&code).unwrap();
        assert!(deep);
        assert_eq!(encode_friendly(true, new_seed), code);
    }

    #[test]
    fn dp_orange_42_stable() {
        let (deep, seed) = decode_to_seed("DP-ORANGE42").unwrap();
        assert!(deep);
        assert_eq!(encode_friendly(true, seed), "DP-ORANGE42");
    }

    #[test]
    fn test_parse_share_code() {
        let (mode, _seed) = parse_share_code("CL-ORANGE42").unwrap();
        assert_eq!(mode, GameMode::Classic);

        let (mode, _seed) = parse_share_code("DP-MANGO99").unwrap();
        assert_eq!(mode, GameMode::Deep);
    }
}
