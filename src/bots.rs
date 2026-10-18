//! Taban botlar: rastgele ve açgözlü.

use crate::game::{Action, Agent, View};
use crate::rng::Rng;
use crate::eval::{discard_distances, distance, ukeire_within, Distance, Superset, PACKED_KINDS};
use crate::solver::{best_win15, pack, Packed};
use crate::tile::{Kind, FALSE_OKEY};

fn forbidden(v: &View) -> Option<Kind> {
    if v.rules.rediscard_taken {
        None
    } else {
        v.taken
    }
}

/// Desteden çeker, bitebiliyorsa biter, yoksa rastgele atar.
pub struct RandomBot {
    rng: Rng,
}

impl RandomBot {
    pub fn new(seed: u64) -> Self {
        RandomBot { rng: Rng::new(seed) }
    }
}

impl Agent for RandomBot {
    fn take_offer(&mut self, _v: &View) -> bool {
        false
    }

    fn act(&mut self, v: &View) -> Action {
        if let Some(w) = best_win15(v.hand, v.okey, v.rules, forbidden(v)) {
            return Action::Win(w.discard);
        }
        loop {
            let t = v.hand[self.rng.below(v.hand.len())];
            if Some(t) != forbidden(v) {
                return Action::Discard(t);
            }
        }
    }
}

/// Bitişe uzaklığı en küçük tutan atışı seçer; eşitlikte eli ilerleten görünmeyen taş
/// sayısı (ukeire) en büyük olanı. Yerden taşı ancak uzaklığı azaltıyorsa alır.
/// Rakiplerin elini hiç dikkate almaz; yalnızca görünen taşları sayar.
pub struct GreedyBot;

/// Taşın Packed uzayındaki yeri: yüz 0..51, gerçek okey 52.
fn slot(t: Kind, okey: Kind) -> usize {
    if t == okey {
        PACKED_KINDS - 1
    } else if t == FALSE_OKEY {
        okey as usize
    } else {
        t as usize
    }
}

fn remove(p: &mut Packed, t: Kind, okey: Kind) {
    match slot(t, okey) {
        s if s == PACKED_KINDS - 1 => p.jokers -= 1,
        s => p.counts[s] -= 1,
    }
}

/// Oyuncunun göremediği taşlar: kendi eli, ortadaki atıklar ve gösterge dışındakiler.
fn unseen(v: &View) -> [u8; PACKED_KINDS] {
    let mut u = [2u8; PACKED_KINDS];
    let seen = v.hand.iter().chain(v.discards.iter().flatten()).chain(std::iter::once(&v.indicator));
    for &t in seen {
        let s = slot(t, v.okey);
        u[s] = u[s].saturating_sub(1);
    }
    u
}

/// 15 taştan en iyi atış: en küçük uzaklık, sonra en büyük ukeire, sonra okey olmayan.
fn best_discard(v: &View, forbid: Option<Kind>, full: &Packed, dist: &[Option<Distance>; PACKED_KINDS]) -> Kind {
    let mut cands: Vec<(Kind, Packed, Distance)> = Vec::with_capacity(15);
    for (idx, &t) in v.hand.iter().enumerate() {
        if Some(t) == forbid || v.hand[..idx].contains(&t) {
            continue;
        }
        let mut p = *full;
        remove(&mut p, t, v.okey);
        cands.push((t, p, dist[slot(t, v.okey)].expect("eldeki taş")));
    }
    let low = cands.iter().map(|c| c.2.min()).min().expect("atılabilecek taş yok");
    cands.retain(|c| c.2.min() == low);
    if cands.len() == 1 {
        return cands[0].0;
    }
    let u = unseen(v);
    let mut sup = Superset::new(full, v.rules);
    let mut best = (cands[0].0, 0u32);
    let mut first = true;
    for (t, p, d) in &cands {
        let k = ukeire_within(p, v.rules, &u, *d, Some((&mut sup, slot(*t, v.okey))));
        // Eşitlikte okeyi atmamak için okey en sona kalır.
        if first || k > best.1 || (k == best.1 && best.0 == v.okey) {
            best = (*t, k);
            first = false;
        }
    }
    best.0
}

impl Agent for GreedyBot {
    fn take_offer(&mut self, v: &View) -> bool {
        let t = match v.offer {
            Some(t) => t,
            None => return false,
        };
        let mut h = v.hand.to_vec();
        h.push(t);
        let after = distance(&pack(&h, v.okey), v.rules).min();
        if after == 0 {
            let forbid = if v.rules.rediscard_taken { None } else { Some(t) };
            if best_win15(&h, v.okey, v.rules, forbid).is_some() {
                return true;
            }
        }
        // Uzaklık azalıyorsa en iyi atış alınan taş değildir.
        after < distance(&pack(v.hand, v.okey), v.rules).min()
    }

    fn act(&mut self, v: &View) -> Action {
        let full = pack(v.hand, v.okey);
        let dist = discard_distances(&full, v.rules);
        if dist.iter().flatten().any(|d| d.min() == 0) {
            if let Some(w) = best_win15(v.hand, v.okey, v.rules, forbidden(v)) {
                return Action::Win(w.discard);
            }
        }
        Action::Discard(best_discard(v, forbidden(v), &full, &dist))
    }
}
