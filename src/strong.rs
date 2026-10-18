//! Güçlü bot: GreedyBot'a taş sayma, savunma, çift yolu ve okey atarak bitme kararı ekler.

use crate::bots::{forbidden, remove, slot, unseen};
use crate::eval::{discard_distances, distance, ukeire_within, Distance, Superset, PACKED_KINDS};
use crate::game::{Action, Agent, View};
use crate::solver::{best_win15, pack, Packed};
use crate::tile::{Kind, FACES};

// Sabitler `istaka duel strong greedy` ölçümleriyle seçildi (README, Faz 1).

/// Çift yoluna bu kadar çiftle girilir (15 taşta).
const PAIR_MIN: usize = 5;
/// Okey atarak bitmek için el bitmişken beklenecek en az deste.
const WAIT_MIN_STOCK: usize = 8;
/// Beklerken bir rakibin bir turda bitme olasılığı tahmini. Beklemenin bir turluk
/// kazancı p·6 (okeyin yerine geçecek taşı çekip 4 puanla bitmek, 2 yerine), kaybı
/// 3·q·8 (üç rakipten biri biterse 6 yerine -2): p > 4q ise beklenir.
const WAIT_RISK: f32 = 0.03;
/// Deste bittiğinde tehlike ağırlığı (ukeire birimiyle); deste azaldıkça karesiyle artar.
const DEFENSE: f32 = 2.0;

/// GreedyBot'a ek olarak: görünen taşları sayar (iki kopyası görünen yüz ölüdür),
/// sağdaki oyuncunun (atışımı alabilecek kişi) aldığı ve almadan geçtiği taşlardan ona
/// yarayacak taşları atmaktan kaçınır, çok çifti olunca çift yoluna girer, el bitmişken
/// okeyi atarak bitebilecekse ve deste yeterliyse bekler.
#[derive(Default)]
pub struct StrongBot {
    /// Yeni eli tanımak için: son görülen gösterge ve deste.
    last: Option<(Kind, usize)>,
    /// Son atışım ve atıştan sonraki yığın uzunluğum.
    mine: Option<(Kind, usize)>,
    /// Sağdakinin benden aldığı ve almadan geçtiği taşlar.
    taken: Vec<Kind>,
    passed: Vec<Kind>,
}

/// İki yüzün aynı pere girme yakınlığı: aynı renkte bir ya da iki sayı ötesi, aynı sayıda
/// başka renk; aynı yüz çift olabilir.
fn kinship(a: usize, b: usize) -> f32 {
    let (ca, ra, cb, rb) = (a / 13, (a % 13) as i32, b / 13, (b % 13) as i32);
    if ca == cb {
        match (ra - rb).abs().min(13 - (ra - rb).abs()) {
            0 => 0.5,
            1 => 1.0,
            2 => 0.5,
            _ => 0.0,
        }
    } else if ra == rb {
        0.8
    } else {
        0.0
    }
}

impl StrongBot {
    fn observe(&mut self, v: &View) {
        if self.last.map_or(true, |(ind, stock)| ind != v.indicator || v.stock_left > stock) {
            *self = StrongBot::default();
        }
        self.last = Some((v.indicator, v.stock_left));
        if let Some((t, len)) = self.mine.take() {
            if v.discards[v.seat].len() < len {
                self.taken.push(t);
            } else {
                self.passed.push(t);
            }
        }
    }

    /// `t`'yi atmanın sağdaki oyuncuya yarama ölçüsü.
    fn danger(&self, t: Kind, v: &View) -> f32 {
        if t == v.okey {
            return 10.0;
        }
        let f = slot(t, v.okey);
        let rel = |x: Kind| if x == v.okey { None } else { Some(slot(x, v.okey)) };
        let mut d = 1.0;
        for g in self.taken.iter().filter_map(|&x| rel(x)) {
            d += 2.0 * kinship(f, g);
        }
        let next = (v.seat + 1) % 4;
        for g in v.discards[next].iter().filter_map(|&x| rel(x)) {
            d -= if g == f { 1.0 } else { 0.3 * kinship(f, g) };
        }
        for g in self.passed.iter().filter_map(|&x| rel(x)) {
            d -= if g == f { 0.7 } else { 0.3 * kinship(f, g) };
        }
        d.max(0.0)
    }

    fn pairs_mode(p: &Packed) -> bool {
        p.counts.iter().filter(|&&c| c == 2).count() >= PAIR_MIN
    }

    /// El bitmişken okeyi atarak bitmek için atılacak taş: kalan 14'lük bitmiş, okeyin
    /// yerine geçecek doğal taşın canlı kopyaları beklemeye değecek kadar ve deste yeterli.
    fn wait_discard(&self, v: &View, full: &Packed, u: &[u8; PACKED_KINDS]) -> Option<Kind> {
        if full.jokers == 0 || v.stock_left < WAIT_MIN_STOCK {
            return None;
        }
        let mut best: Option<(Kind, u32)> = None;
        for (idx, &t) in v.hand.iter().enumerate() {
            if t == v.okey || Some(t) == forbidden(v) || v.hand[..idx].contains(&t) {
                continue;
            }
            let mut p = *full;
            remove(&mut p, t, v.okey);
            if distance(&p, v.rules).min() != 0 {
                continue;
            }
            // Okeyi çıkarıp yerine her yüzü dene.
            p.jokers -= 1;
            let live: u32 = (0..FACES)
                .filter(|&f| u[f] > 0 && p.counts[f] < 2)
                .filter(|&f| {
                    let mut q = p;
                    q.counts[f] += 1;
                    distance(&q, v.rules).min() == 0
                })
                .map(|f| u[f] as u32)
                .sum();
            // Çekilecek taşın o yüzlerden biri olma olasılığı: canlı kopya / görünmeyen taş.
            let hidden: u32 = u.iter().map(|&x| x as u32).sum();
            if live as f32 / hidden.max(1) as f32 <= 4.0 * WAIT_RISK {
                continue;
            }
            if best.map_or(true, |b| live > b.1) {
                best = Some((t, live));
            }
        }
        best.map(|b| b.0)
    }

    /// Bitemiyorken atış: en küçük uzaklık (çift yolunda çift uzaklığı); eşitlerde
    /// canlı ukeire eksi deste azaldıkça artan ağırlıkla tehlike.
    fn choose(&self, v: &View, full: &Packed, dist: &[Option<Distance>; PACKED_KINDS], u: &[u8; PACKED_KINDS]) -> Kind {
        let pairs = StrongBot::pairs_mode(full);
        let key = |d: &Distance| if pairs { d.pairs } else { d.min() };
        let forbid = forbidden(v);
        let mut cands: Vec<(Kind, Packed, Distance)> = Vec::with_capacity(15);
        for (idx, &t) in v.hand.iter().enumerate() {
            if Some(t) == forbid || v.hand[..idx].contains(&t) {
                continue;
            }
            let mut p = *full;
            remove(&mut p, t, v.okey);
            cands.push((t, p, dist[slot(t, v.okey)].expect("eldeki taş")));
        }
        let low = cands.iter().map(|c| key(&c.2)).min().expect("atılabilecek taş yok");
        let late = 1.0 - v.stock_left as f32 / 48.0;
        let w = DEFENSE * late * late;
        cands.retain(|c| key(&c.2) == low);
        if cands.len() == 1 {
            return cands[0].0;
        }
        let mut sup = Superset::new(full, v.rules);
        let mut best = (cands[0].0, f32::MIN);
        for (t, p, d) in &cands {
            let k = if pairs {
                (0..FACES).filter(|&f| p.counts[f] == 1).map(|f| u[f] as u32).sum::<u32>() + u[FACES] as u32
            } else {
                ukeire_within(p, v.rules, u, *d, Some((&mut sup, slot(*t, v.okey))))
            };
            let s = k as f32 - w * self.danger(*t, v);
            // Eşitlikte okeyi atmamak için okey en sona kalır.
            if s > best.1 || (s == best.1 && best.0 == v.okey) {
                best = (*t, s);
            }
        }
        best.0
    }
}

impl Agent for StrongBot {
    fn take_offer(&mut self, v: &View) -> bool {
        self.observe(v);
        let t = match v.offer {
            Some(t) => t,
            None => return false,
        };
        let mut h = v.hand.to_vec();
        h.push(t);
        let forbid = if v.rules.rediscard_taken { None } else { Some(t) };
        let now_p = pack(v.hand, v.okey);
        let now = distance(&now_p, v.rules);
        if let Some(w) = best_win15(&h, v.okey, v.rules, forbid) {
            // Okeyi atarak bitmek için bekliyorsak yalnızca o bitişi al.
            return w.okey_discard || now.min() > 0;
        }
        let after = distance(&pack(&h, v.okey), v.rules);
        if StrongBot::pairs_mode(&now_p) {
            after.pairs < now.pairs
        } else {
            after.min() < now.min()
        }
    }

    fn act(&mut self, v: &View) -> Action {
        self.observe(v);
        let full = pack(v.hand, v.okey);
        let u = unseen(v);
        let dist = discard_distances(&full, v.rules);
        let choice = if dist.iter().flatten().any(|d| d.min() == 0) {
            match best_win15(v.hand, v.okey, v.rules, forbidden(v)) {
                Some(w) if w.okey_discard => return Action::Win(w.discard),
                Some(w) => match self.wait_discard(v, &full, &u) {
                    Some(t) => t,
                    None => return Action::Win(w.discard),
                },
                None => self.choose(v, &full, &dist, &u),
            }
        } else {
            self.choose(v, &full, &dist, &u)
        };
        self.mine = Some((choice, v.discards[v.seat].len() + 1));
        Action::Discard(choice)
    }
}
