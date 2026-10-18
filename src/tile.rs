//! Taş türleri. Bir tür 0..=51 arası `renk * 13 + (sayı - 1)`, 52 ise sahte okeydir.
//! Her türden destede 2 kopya vardır (toplam 106).

pub type Kind = u8;

pub const FALSE_OKEY: Kind = 52;
pub const KINDS: usize = 53;
pub const FACES: usize = 52;

/// Renk harfleri: Y sarı, K kırmızı, S siyah, M mavi.
pub const COLOR_LETTERS: [char; 4] = ['Y', 'K', 'S', 'M'];

#[inline]
pub fn color(k: Kind) -> u8 {
    k / 13
}

#[inline]
pub fn rank(k: Kind) -> u8 {
    k % 13 + 1
}

#[inline]
pub fn face(color: u8, rank: u8) -> Kind {
    color * 13 + (rank - 1)
}

/// Göstergeden okey: aynı renk, bir büyük sayı; 13'ten sonra 1.
pub fn okey_of(indicator: Kind) -> Kind {
    assert!(indicator < FALSE_OKEY, "gösterge sahte okey olamaz");
    face(color(indicator), rank(indicator) % 13 + 1)
}

/// "K5", "M13", "F" (sahte okey) biçimindeki adı çözer.
pub fn parse(s: &str) -> Option<Kind> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("F") {
        return Some(FALSE_OKEY);
    }
    let mut chars = s.chars();
    let c = chars.next()?.to_ascii_uppercase();
    let col = COLOR_LETTERS.iter().position(|&x| x == c)? as u8;
    let r: u8 = chars.as_str().parse().ok()?;
    if (1..=13).contains(&r) {
        Some(face(col, r))
    } else {
        None
    }
}

pub fn name(k: Kind) -> String {
    if k == FALSE_OKEY {
        "F".to_string()
    } else {
        format!("{}{}", COLOR_LETTERS[color(k) as usize], rank(k))
    }
}

pub fn names(tiles: &[Kind]) -> String {
    tiles.iter().map(|&t| name(t)).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn okey_wraps_after_13() {
        assert_eq!(okey_of(parse("K13").unwrap()), parse("K1").unwrap());
        assert_eq!(okey_of(parse("M5").unwrap()), parse("M6").unwrap());
    }

    #[test]
    fn parse_roundtrip() {
        for k in 0..KINDS as u8 {
            assert_eq!(parse(&name(k)), Some(k));
        }
    }
}
