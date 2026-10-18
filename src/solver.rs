//! El çözücü: bitiş kontrolü ve perlere girebilen azami taş sayısı.
//!
//! Elin çözüm için temsili: gerçek okeyler joker sayısına (`jokers`, en çok 2)
//! gider; sahte okeyler okey yüzünün yerine sayılır. `counts[f]` her yüzden
//! eldeki kopya sayısıdır (0..=2).

use crate::rules::Rules;
use crate::tile::{color, face, rank, Kind, FACES, FALSE_OKEY};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Packed {
    pub counts: [u8; FACES],
    pub jokers: u8,
}

pub fn pack(tiles: &[Kind], okey: Kind) -> Packed {
    let mut counts = [0u8; FACES];
    let mut jokers = 0;
    for &t in tiles {
        if t == okey {
            jokers += 1;
        } else if t == FALSE_OKEY {
            counts[okey as usize] += 1;
        } else {
            counts[t as usize] += 1;
        }
    }
    Packed { counts, jokers }
}

/// En fazla 5 yüzlük bir per adayı; `fixed` konumu daima doğal taşla doldurulur.
struct Template {
    faces: [u8; 5],
    len: usize,
    fixed: usize,
}

/// `i` yüzünü içeren bütün per adaylarını (3-4'lü gruplar, 3-5'li seriler) üretir.
/// Uzun seriler 3-5'lik parçalara bölünebildiği için bu uzunluklar yeterlidir.
fn templates(i: usize, rules: &Rules, mut f: impl FnMut(&Template) -> bool) -> bool {
    let col = color(i as u8);
    let r = rank(i as u8);
    // Gruplar: diğer üç renkten 2 ya da 3'ü.
    let mut others = [0u8; 3];
    let mut n = 0;
    for c in 0..4 {
        if c != col {
            others[n] = c;
            n += 1;
        }
    }
    for skip in 0..4 {
        // skip == 3: dört renk; aksi halde others[skip] dışarıda.
        let mut t = Template { faces: [0; 5], len: 0, fixed: 0 };
        t.faces[0] = i as u8;
        t.len = 1;
        for (k, &c) in others.iter().enumerate() {
            if k != skip {
                t.faces[t.len] = face(c, r);
                t.len += 1;
            }
        }
        if f(&t) {
            return true;
        }
    }
    // Seriler: genişletilmiş eksende 1..=14 (14 = 13'ten sonraki 1).
    let top = if rules.wrap_13_1 { 14 } else { 13 };
    let positions = [r, 14];
    let np = if r == 1 && rules.wrap_13_1 { 2 } else { 1 };
    for &p in &positions[..np] {
        for len in 3..=5u8 {
            let lo = p.saturating_sub(len - 1).max(1);
            for s in lo..=p {
                let e = s + len - 1;
                if e > top {
                    break;
                }
                let mut t = Template { faces: [0; 5], len: len as usize, fixed: (p - s) as usize };
                for k in 0..len {
                    let pr = s + k;
                    t.faces[k as usize] = face(col, if pr == 14 { 1 } else { pr });
                }
                if f(&t) {
                    return true;
                }
            }
        }
    }
    false
}

/// Şablonu eldeki taşlarla doldurmanın bütün yolları (hangi konumlar joker).
/// `apply` her doldurma için perdeki doğal taş sayısıyla çağrılır; true dönerse arama durur.
fn fills(
    t: &Template,
    p: &mut Packed,
    rules: &Rules,
    mut apply: impl FnMut(&mut Packed, usize) -> bool,
) -> bool {
    let n = t.len;
    let max_j = p.jokers.min(rules.max_okeys_per_meld) as u32;
    for mask in 0u32..(1 << n) {
        if mask & (1 << t.fixed) != 0 || mask.count_ones() > max_j {
            continue;
        }
        let ok = (0..n).all(|k| mask & (1 << k) != 0 || p.counts[t.faces[k] as usize] > 0);
        if !ok {
            continue;
        }
        for k in 0..n {
            if mask & (1 << k) == 0 {
                p.counts[t.faces[k] as usize] -= 1;
            }
        }
        let used = mask.count_ones() as u8;
        p.jokers -= used;
        let stop = apply(p, n - used as usize);
        p.jokers += used;
        for k in 0..n {
            if mask & (1 << k) == 0 {
                p.counts[t.faces[k] as usize] += 1;
            }
        }
        if stop {
            return true;
        }
    }
    false
}

/// Bütün taşlar (jokerler dahil) seri ve gruplara ayrılabiliyor mu?
pub fn is_normal(p: &mut Packed, rules: &Rules) -> bool {
    let i = match p.counts.iter().position(|&c| c > 0) {
        Some(i) => i,
        None => return p.jokers == 0,
    };
    templates(i, rules, |t| fills(t, p, rules, |q, _| is_normal(q, rules)))
}

/// 7 çift: her tekin eşi bir okeyle tamamlanır, artan okeyler kendi aralarında çifttir.
pub fn is_pairs(p: &Packed) -> bool {
    let total: u32 = p.counts.iter().map(|&c| c as u32).sum::<u32>() + p.jokers as u32;
    if total != 14 {
        return false;
    }
    let singles = p.counts.iter().filter(|&&c| c == 1).count() as u8;
    singles <= p.jokers
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Finish14 {
    pub normal: bool,
    pub pairs: bool,
}

pub fn finish14(tiles: &[Kind], okey: Kind, rules: &Rules) -> Finish14 {
    debug_assert_eq!(tiles.len(), 14);
    let mut p = pack(tiles, okey);
    let pairs = is_pairs(&p);
    let normal = is_normal(&mut p, rules);
    Finish14 { normal, pairs }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Win {
    pub discard: Kind,
    pub pairs: bool,
    pub okey_discard: bool,
    pub points: u8,
}

pub fn points(pairs: bool, okey_discard: bool) -> u8 {
    2 * if pairs { 2 } else { 1 } * if okey_discard { 2 } else { 1 }
}

/// 15 taşlık elden en çok puan getiren bitiş. `forbidden` atılamayacak taş.
pub fn best_win15(tiles: &[Kind], okey: Kind, rules: &Rules, forbidden: Option<Kind>) -> Option<Win> {
    debug_assert_eq!(tiles.len(), 15);
    let mut best: Option<Win> = None;
    let mut rest = Vec::with_capacity(14);
    for (idx, &d) in tiles.iter().enumerate() {
        if Some(d) == forbidden || tiles[..idx].contains(&d) {
            continue;
        }
        rest.clear();
        rest.extend(tiles.iter().enumerate().filter(|&(k, _)| k != idx).map(|(_, &t)| t));
        let f = finish14(&rest, okey, rules);
        if !f.normal && !f.pairs {
            continue;
        }
        let okey_discard = d == okey;
        let w = Win { discard: d, pairs: f.pairs, okey_discard, points: points(f.pairs, okey_discard) };
        if best.map_or(true, |b| w.points > b.points) {
            best = Some(w);
        }
    }
    best
}

/// Perlere girebilen azami taş sayısı; jokerler her zaman işe yarar sayılır. Faz 0 açgözlü
/// botunun sezgiseliydi; botlar artık `eval` kullanıyor.
pub fn max_melded(p: &Packed, rules: &Rules) -> u8 {
    let mut q = *p;
    let remaining: u8 = q.counts.iter().sum();
    let mut best = 0;
    melded_rec(&mut q, rules, 0, remaining, &mut best);
    best + p.jokers
}

fn melded_rec(p: &mut Packed, rules: &Rules, used: u8, remaining: u8, best: &mut u8) {
    if used + remaining <= *best {
        return;
    }
    let i = match p.counts.iter().position(|&c| c > 0) {
        Some(i) => i,
        None => {
            *best = (*best).max(used);
            return;
        }
    };
    templates(i, rules, |t| {
        fills(t, p, rules, |q, nat| {
            melded_rec(q, rules, used + nat as u8, remaining - nat as u8, best);
            false
        })
    });
    // i'yi artık bırak.
    p.counts[i] -= 1;
    melded_rec(p, rules, used, remaining - 1, best);
    p.counts[i] += 1;
}
