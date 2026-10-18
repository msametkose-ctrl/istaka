//! Çözücüyü bağımsız, kaba kuvvet bir denetleyiciyle karşılaştırır.
//!
//! Denetleyici jokerleri önce bütün olası yüzlere yerleştirir, sonra jokersiz eli
//! en basit yöntemle (ilk taş ya grupta ya da kendisiyle başlayan seride) çözer.
//! Joker sınırı 2 iken bu, kuralın doğrudan tanımıdır.

use istaka::rng::Rng;
use istaka::rules::Rules;
use istaka::solver::{is_normal, pack};
use istaka::tile::{face, okey_of, Kind, FALSE_OKEY};

fn brute_plain(c: &mut [u8; 52], wrap: bool) -> bool {
    let i = match c.iter().position(|&x| x > 0) {
        Some(i) => i,
        None => return true,
    };
    let col = i / 13;
    let r = i % 13 + 1;
    let others: Vec<usize> = (0..4).filter(|&x| x != col).map(|x| x * 13 + r - 1).collect();
    // Gruplar: i + diğer renklerden 2 ya da 3'ü.
    for mask in 0u8..8 {
        if mask.count_ones() < 2 {
            continue;
        }
        let sel: Vec<usize> = (0..3).filter(|&k| mask & (1 << k) != 0).map(|k| others[k]).collect();
        if sel.iter().all(|&f| c[f] > 0) {
            c[i] -= 1;
            sel.iter().for_each(|&f| c[f] -= 1);
            let ok = brute_plain(c, wrap);
            c[i] += 1;
            sel.iter().for_each(|&f| c[f] += 1);
            if ok {
                return true;
            }
        }
    }
    // i ile başlayan seriler ve wrap açıksa i (=1) ile biten seriler.
    let mut runs: Vec<Vec<usize>> = Vec::new();
    for len in 3..=5 {
        if r + len - 1 <= 13 {
            runs.push((0..len).map(|k| i + k).collect());
        }
        if wrap && r == 1 {
            let mut v: Vec<usize> = (14 - (len - 1)..=13).map(|x| col * 13 + x - 1).collect();
            v.push(i);
            runs.push(v);
        }
    }
    for run in runs {
        if run.iter().all(|&f| c[f] > 0) {
            run.iter().for_each(|&f| c[f] -= 1);
            let ok = brute_plain(c, wrap);
            run.iter().for_each(|&f| c[f] += 1);
            if ok {
                return true;
            }
        }
    }
    false
}

fn brute(tiles: &[Kind], okey: Kind, wrap: bool) -> bool {
    let p = pack(tiles, okey);
    let mut c = p.counts;
    match p.jokers {
        0 => brute_plain(&mut c, wrap),
        1 => (0..52).any(|f| {
            c[f] += 1;
            let ok = brute_plain(&mut c, wrap);
            c[f] -= 1;
            ok
        }),
        _ => (0..52).any(|f| {
            (f..52).any(|g| {
                c[f] += 1;
                c[g] += 1;
                let ok = brute_plain(&mut c, wrap);
                c[f] -= 1;
                c[g] -= 1;
                ok
            })
        }),
    }
}

/// Rastgele perlerden kurulmuş (çoğu bitmiş) bir el; sonra bazen bozulur.
fn constructed(rng: &mut Rng, okey: Kind, wrap: bool) -> Option<Vec<Kind>> {
    let mut left = [2u8; 53];
    left[okey as usize] = 0; // okey yüzü joker olarak ayrıca eklenir
    let mut hand: Vec<Kind> = Vec::new();
    let mut guard = 0;
    while hand.len() < 14 {
        guard += 1;
        if guard > 200 {
            return None;
        }
        let need = 14 - hand.len();
        if need < 3 {
            return None;
        }
        let col = rng.below(4) as u8;
        let r = rng.below(13) as u8 + 1;
        let meld: Vec<Kind> = if rng.below(2) == 0 {
            let len = 3 + rng.below(3.min(need - 2)) as u8;
            let top = if wrap { 14 } else { 13 };
            if r + len - 1 > top {
                continue;
            }
            (0..len).map(|k| face(col, if r + k == 14 { 1 } else { r + k })).collect()
        } else {
            let mut cols: Vec<u8> = (0..4).collect();
            rng.shuffle(&mut cols);
            let n = if need >= 4 && rng.below(2) == 0 { 4 } else { 3 };
            cols[..n].iter().map(|&c| face(c, r)).collect()
        };
        if meld.len() > need || (need - meld.len() > 0 && need - meld.len() < 3) {
            continue;
        }
        // Okey yüzü gerekiyorsa sahte okeyle karşılanır.
        let mut use_: Vec<Kind> = Vec::new();
        let mut l2 = left;
        let mut ok = true;
        for &f in &meld {
            let k = if f == okey { FALSE_OKEY } else { f };
            if l2[k as usize] == 0 {
                ok = false;
                break;
            }
            l2[k as usize] -= 1;
            use_.push(k);
        }
        if !ok {
            continue;
        }
        left = l2;
        hand.extend(use_);
    }
    // 0-2 taşı gerçek okeyle değiştir, bazen bir taşı rastgele bir taşla değiştir.
    let jokers = rng.below(3);
    for k in 0..jokers {
        let idx = rng.below(14);
        if hand[idx] != okey {
            hand[idx] = okey;
        } else if k == 0 {
            continue;
        }
    }
    if rng.below(2) == 0 {
        let idx = rng.below(14);
        hand[idx] = rng.below(53) as Kind;
    }
    // Her türden en çok 2 taş.
    let mut cnt = [0u8; 53];
    for &t in &hand {
        cnt[t as usize] += 1;
        if cnt[t as usize] > 2 {
            return None;
        }
    }
    Some(hand)
}

#[test]
fn solver_matches_brute_force() {
    let mut rng = Rng::new(2026);
    let mut checked = 0;
    let mut positives = 0;
    for wrap in [true, false] {
        let rules = Rules { wrap_13_1: wrap, ..Rules::default() };
        let mut n = 0;
        while n < 15_000 {
            let indicator = rng.below(52) as Kind;
            let okey = okey_of(indicator);
            let hand = match constructed(&mut rng, okey, wrap) {
                Some(h) => h,
                None => continue,
            };
            let want = brute(&hand, okey, wrap);
            let mut p = pack(&hand, okey);
            let got = is_normal(&mut p, &rules);
            assert_eq!(
                got,
                want,
                "uyuşmazlık (wrap={wrap}): okey {} el {}",
                istaka::tile::name(okey),
                istaka::tile::names(&hand)
            );
            n += 1;
            checked += 1;
            positives += want as u32;
        }
    }
    assert!(positives > 5_000, "yeterince bitmiş el üretilmedi: {positives}/{checked}");
}

/// Tablo değerlendiricisi: uzaklık 0 ⇔ bitmiş el (kaba kuvvet ve `is_normal`/`is_pairs`),
/// 15 taşta uzaklık 0 ⇔ bir atışla bitiş, normal uzaklık <= 1 ⇔ tek taş değişimiyle bitiş;
/// `discard_distances` her atış için `distance` ile aynı.
#[test]
fn evaluator_matches_solver() {
    use istaka::eval::distance;
    use istaka::solver::{is_pairs, Packed};

    fn normal_after_swap(p: &Packed, rules: &Rules) -> bool {
        // Bir doğal taşı at, herhangi bir yüzü (kopya sınırı olmadan) çek.
        (0..52).filter(|&d| p.counts[d] > 0).any(|d| {
            (0..52).any(|t| {
                let mut q = *p;
                q.counts[d] -= 1;
                q.counts[t] += 1;
                is_normal(&mut q, rules)
            })
        })
    }

    let mut rng = Rng::new(77);
    let mut finished = 0;
    let mut near = 0;
    for wrap in [true, false] {
        let rules = Rules { wrap_13_1: wrap, ..Rules::default() };
        let mut n = 0;
        while n < 12_000 {
            let okey = okey_of(rng.below(52) as Kind);
            // Yarısı perlerden kurulmuş, yarısı tamamen rastgele el.
            let hand = if n % 2 == 0 {
                match constructed(&mut rng, okey, wrap) {
                    Some(h) => h,
                    None => continue,
                }
            } else {
                let mut all: Vec<Kind> = (0..53u8).flat_map(|k| [k, k]).collect();
                rng.shuffle(&mut all);
                all.truncate(14);
                all
            };
            let p = pack(&hand, okey);
            let d = distance(&p, &rules);
            let names = istaka::tile::names(&hand);
            let normal = brute(&hand, okey, wrap);
            assert_eq!(d.normal == 0, normal, "normal (wrap={wrap}) okey {} el {names}", istaka::tile::name(okey));
            assert_eq!(d.pairs == 0, is_pairs(&p), "çift: el {names}");
            finished += (d.min() == 0) as u32;

            // 15 taş: rastgele bir taş ekle.
            let mut h15 = hand.clone();
            h15.push(rng.below(53) as Kind);
            if h15.iter().filter(|&&t| t == *h15.last().unwrap()).count() <= 2 {
                let p15 = pack(&h15, okey);
                let d15 = distance(&p15, &rules);
                let win = istaka::solver::best_win15(&h15, okey, &rules, None).is_some();
                assert_eq!(d15.min() == 0, win, "15 taş (wrap={wrap}) okey {} el {}", istaka::tile::name(okey), istaka::tile::names(&h15));
                let all = istaka::eval::discard_distances(&p15, &rules);
                for x in 0..53 {
                    let mut q = p15;
                    let present = if x == 52 { q.jokers > 0 } else { q.counts[x] > 0 };
                    if present {
                        if x == 52 {
                            q.jokers -= 1;
                        } else {
                            q.counts[x] -= 1;
                        }
                    }
                    assert_eq!(all[x], present.then(|| distance(&q, &rules)), "atış {x}: el {}", istaka::tile::names(&h15));
                }
            }

            // Uzaklık <= 1 denetimi pahalı; bir kısmında yapılır.
            if n % 6 == 0 {
                let one = d.normal <= 1;
                assert_eq!(one, d.normal == 0 || normal_after_swap(&p, &rules), "uzaklık 1 (wrap={wrap}) okey {} el {names}", istaka::tile::name(okey));
                near += one as u32;
            }
            n += 1;
        }
    }
    assert!(finished > 3_000 && near > 1_000, "yeterince bitmiş/yakın el yok: {finished} {near}");
}

/// Hızlı ukeire (seri iyileştiricileri, 15'lik süzgeci, örnek örtü) her taşı tek tek
/// ekleyip uzaklığa bakan doğrudan tanımla aynı sonucu vermeli.
#[test]
fn ukeire_matches_definition() {
    use istaka::eval::{distance, ukeire, ukeire_within, Superset, PACKED_KINDS};
    use istaka::solver::Packed;

    fn direct(p: &Packed, rules: &Rules, unseen: &[u8; PACKED_KINDS]) -> u32 {
        let now = distance(p, rules).min();
        if now == 0 {
            return 0;
        }
        (0..PACKED_KINDS)
            .filter(|&f| unseen[f] > 0)
            .filter(|&f| {
                let mut q = *p;
                if f == 52 {
                    if q.jokers >= 2 {
                        return false;
                    }
                    q.jokers += 1;
                } else {
                    if q.counts[f] >= 2 {
                        return false;
                    }
                    q.counts[f] += 1;
                }
                distance(&q, rules).min() < now
            })
            .map(|f| unseen[f] as u32)
            .sum()
    }

    let mut rng = Rng::new(9);
    let mut nonzero = 0;
    for wrap in [true, false] {
        let rules = Rules { wrap_13_1: wrap, ..Rules::default() };
        let mut n = 0;
        while n < 3_000 {
            let okey = okey_of(rng.below(52) as Kind);
            // 15 taş: perlerden kurulmuş 14'lük + bir taş ya da tamamen rastgele.
            let mut h15 = if n % 2 == 0 {
                match constructed(&mut rng, okey, wrap) {
                    Some(h) => h,
                    None => continue,
                }
            } else {
                let mut all: Vec<Kind> = (0..53u8).flat_map(|k| [k, k]).collect();
                rng.shuffle(&mut all);
                all.truncate(14);
                all
            };
            h15.push(rng.below(53) as Kind);
            if h15.iter().filter(|&&t| t == h15[14]).count() > 2 {
                continue;
            }
            let full = pack(&h15, okey);
            let mut unseen = [0u8; PACKED_KINDS];
            for u in unseen.iter_mut() {
                *u = rng.below(3) as u8;
            }
            let mut sup = Superset::new(&full, &rules);
            for x in 0..PACKED_KINDS {
                let mut p = full;
                if x == 52 {
                    if p.jokers == 0 {
                        continue;
                    }
                    p.jokers -= 1;
                } else {
                    if p.counts[x] == 0 {
                        continue;
                    }
                    p.counts[x] -= 1;
                }
                let want = direct(&p, &rules, &unseen);
                let d = distance(&p, &rules);
                assert_eq!(ukeire(&p, &rules, &unseen, d), want, "ukeire (wrap={wrap}): el {}", istaka::tile::names(&h15));
                assert_eq!(
                    ukeire_within(&p, &rules, &unseen, d, Some((&mut sup, x))),
                    want,
                    "ukeire süzgeçli (wrap={wrap}): el {} atılan {x}",
                    istaka::tile::names(&h15)
                );
                nonzero += (want > 0) as u32;
            }
            n += 1;
        }
    }
    assert!(nonzero > 10_000, "çok az ilerleyebilen el: {nonzero}");
}
